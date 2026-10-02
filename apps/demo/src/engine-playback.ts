import type { PortableScore, SectionGain, SectionId } from "../../../packages/runtime/src/index.ts";
import { savedVolume, saveVolume, VOLUME_GLIDE_SECONDS } from "./lab-volume.ts";
import { PCM_QUEUE_PROCESSOR, type PcmPlayedReport } from "./pcm-queue.ts";
import pcmQueueWorkletUrl from "./pcm-queue-worklet.ts?worker&url";
import { playbackSectionOnScore } from "./playback-section.ts";
import {
  soundingSection,
  type LabScore,
  type PlaybackFrame,
  type PlaybackTransition,
  type SoloMode,
} from "./playback.ts";
import type { SignalReadings } from "./recipes.ts";
import { createPlayer } from "./wasm-engine.ts";
import { MAX_FILL_FRAMES, type GameUpdate, type WasmPlayer } from "./wasm-player.ts";

/** The rate the engine renders at; the browser resamples to the device. */
const ENGINE_SAMPLE_RATE = 48000;
/** Audio rendered ahead of the speakers, like the Godot player's buffer. */
const QUEUE_SECONDS = 0.15;
const PUMP_INTERVAL_MS = 20;

/** What the engine's live player reports it is sounding. */
interface EngineStatus {
  scoreId: string;
  tick: number;
  currentSection: SectionId;
  pendingSection: SectionId | null;
  pendingScoreId: string | null;
  transition: PlaybackTransition | null;
  formHeld: boolean;
  nextFormSection: SectionId | null;
  mix: (SectionGain & { origin: number })[];
}

interface EngineAudio {
  context: AudioContext;
  node: AudioWorkletNode;
  gain: GainNode;
  timer: number;
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** Plays the Lab through the engine games run: the WASM live player renders
 *  here, and a worklet plays its audio. */
export class EnginePlayback {
  static async create(lab: LabScore, opening: SectionId, held: boolean): Promise<EnginePlayback> {
    const playback = new EnginePlayback(await createPlayer(ENGINE_SAMPLE_RATE), lab);
    playback.#reload(opening, held);
    return playback;
  }

  readonly #player: WasmPlayer;
  /** The newest version; the sounding one may still be an earlier score. */
  #lab: LabScore;
  readonly #scores = new Map<string, PortableScore>();
  #audio: EngineAudio | null = null;
  #volume = savedVolume();
  #manualCue: SectionId | null = null;
  /** Frames sent to the worklet, and how many it has played. */
  #rendered = 0;
  #played = 0;
  #playedAt = 0;
  #soloMode: SoloMode = "full";

  private constructor(player: WasmPlayer, lab: LabScore) {
    this.#player = player;
    this.#lab = lab;
  }

  get running(): boolean {
    return this.#audio?.context.state === "running";
  }

  get volume(): number {
    return this.#volume;
  }

  set volume(value: number) {
    this.#volume = saveVolume(value);
    const audio = this.#audio;
    audio?.gain.gain.setTargetAtTime(this.#volume, audio.context.currentTime, VOLUME_GLIDE_SECONDS);
  }

  get soloMode(): SoloMode {
    return this.#soloMode;
  }

  set soloMode(mode: SoloMode) {
    this.#soloMode = mode;
    this.#command({ solo: typeof mode === "object" ? { voice: mode } : mode });
  }

  async start(): Promise<void> {
    if (this.#audio !== null) return;
    const context = new AudioContext({ sampleRate: ENGINE_SAMPLE_RATE });
    await context.audioWorklet.addModule(pcmQueueWorkletUrl);
    const node = new AudioWorkletNode(context, PCM_QUEUE_PROCESSOR, {
      numberOfInputs: 0,
      outputChannelCount: [2],
    });
    const gain = new GainNode(context, { gain: this.#volume });
    node.connect(gain).connect(context.destination);
    node.port.onmessage = (event: MessageEvent<PcmPlayedReport>) => {
      this.#played = event.data.played;
      this.#playedAt = event.data.time;
    };
    if (context.state === "suspended") await context.resume();
    this.#rendered = 0;
    this.#played = 0;
    this.#playedAt = context.currentTime;
    this.#audio = { context, node, gain, timer: window.setInterval(() => this.#pump(), PUMP_INTERVAL_MS) };
    this.#pump();
  }

  async stop(): Promise<void> {
    const audio = this.#audio;
    if (audio === null) return;
    const status = this.#status();
    const sounding = soundingSection(status, this.#audibleTick(status));
    this.#audio = null;
    window.clearInterval(audio.timer);
    this.#reload(sounding, status.formHeld);
    await audio.context.close();
  }

  frame(): PlaybackFrame {
    const status = this.#status();
    const score = this.#scores.get(status.scoreId) ?? this.#lab.score;
    for (const id of this.#scores.keys()) {
      if (id !== status.scoreId && id !== status.pendingScoreId && id !== this.#lab.score.id) this.#scores.delete(id);
    }
    const tick = this.#audibleTick(status, score);
    return {
      score,
      pendingScore: status.pendingScoreId === null ? null : (this.#scores.get(status.pendingScoreId) ?? null),
      tick,
      snapshot: status,
      mix: status.mix,
      formHeld: status.formHeld,
      nextFormSection: status.nextFormSection,
      requestedCue: this.#pendingCue(status, tick),
      sectionTick: (section) => {
        const origin = status.mix.find((part) => part.section === section)?.origin ?? 0;
        const length = score.sections.find((candidate) => candidate.id === section)?.lengthTicks ?? 1;
        return (((tick - origin) % length) + length) % length;
      },
    };
  }

  cue(target: SectionId): void {
    if (!this.running) {
      this.#manualCue = null;
      this.#reload(target, this.#status().formHeld);
      return;
    }
    this.#command({ cue: { section: target } });
    this.#noteRequested();
  }

  request(phase: string, readings: SignalReadings): void {
    const update = this.#lab.profile.engineUpdate(phase, readings);
    if (!this.running) {
      const section = this.#command({ sectionFor: update }) as SectionId | null;
      if (section !== null) this.cue(section);
      return;
    }
    this.#command({ update } satisfies { update: GameUpdate });
    this.#noteRequested();
  }

  setHold(held: boolean): void {
    this.#command({ hold: { held } });
  }

  advance(): void {
    const status = this.#status();
    if (status.transition === null && status.nextFormSection !== null) this.cue(status.nextFormSection);
  }

  cancelCue(): void {
    const status = this.#status();
    if (this.#pendingCue(status, this.#audibleTick(status)) === null) return;
    this.#command("cancel");
    this.#manualCue = null;
  }

  /** Play `next` from here on, from the section that is sounding. */
  switchTo(next: LabScore): void {
    const status = this.#status();
    const opening = playbackSectionOnScore(next.score, soundingSection(status, this.#audibleTick(status)));
    this.#lab = next;
    this.#manualCue = null;
    if (!this.running) {
      this.#reload(opening, status.formHeld);
      return;
    }
    // New music waits for any running blend and the next bar, then blends
    // in, like a seed change in a game.
    this.#scores.set(next.score.id, next.score);
    this.#load(next, opening);
  }

  /** Hand the engine the playing score again after the Lab edited it. */
  reloadScore(): void {
    this.switchTo(this.#lab);
  }

  /** Start over on the newest version at `section`. */
  #reload(section: SectionId, held: boolean): void {
    this.#player.reset();
    this.#scores.clear();
    this.#scores.set(this.#lab.score.id, this.#lab.score);
    this.#load(this.#lab, section);
    if (held) this.setHold(true);
    if (this.#soloMode !== "full") this.soloMode = this.#soloMode;
  }

  #load(lab: LabScore, opening: SectionId): void {
    this.#command({
      load: {
        score: lab.score,
        seed: lab.seed,
        recipe: lab.profile.id,
        rootPitchClass: lab.rootPitchClass,
        openingSection: opening,
      },
    });
  }

  #command(command: unknown): unknown {
    const response = this.#player.command(encoder.encode(JSON.stringify(command)));
    const text = decoder.decode(response.bytes);
    if (!response.ok) throw new Error(text);
    return JSON.parse(text);
  }

  #status(): EngineStatus {
    const status = this.#command("status") as EngineStatus | null;
    if (status === null) throw new Error("The engine has no score loaded");
    return status;
  }

  /** Keep `QUEUE_SECONDS` of audio queued ahead of the speakers. */
  #pump(): void {
    const audio = this.#audio;
    if (audio === null) return;
    let frames = Math.ceil(QUEUE_SECONDS * this.#player.sampleRate) - (this.#rendered - this.#audibleFrames());
    while (frames > 0) {
      const count = Math.min(frames, MAX_FILL_FRAMES);
      const block = this.#player.fill(count).slice();
      audio.node.port.postMessage(block, [block.buffer]);
      this.#rendered += count;
      frames -= count;
    }
  }

  #audibleFrames(): number {
    const audio = this.#audio;
    if (audio === null) return this.#rendered;
    const elapsed = Math.max(0, audio.context.currentTime - this.#playedAt);
    return Math.min(this.#rendered, this.#played + elapsed * this.#player.sampleRate);
  }

  /** The status tick is where rendering is; the speakers are behind it. */
  #audibleTick(status: EngineStatus, score: PortableScore = this.#lab.score): number {
    const queuedSeconds = (this.#rendered - this.#audibleFrames()) / this.#player.sampleRate;
    const ticksPerSecond = (score.bpm / 60) * score.ticksPerBeat;
    return Math.max(0, status.tick - queuedSeconds * ticksPerSecond);
  }

  /** Remember a cue the listener asked for while it waits to start. */
  #noteRequested(): void {
    const status = this.#status();
    const transition = status.transition;
    this.#manualCue =
      status.pendingSection ?? (transition !== null && status.tick < transition.startTick ? transition.to : null);
  }

  #pendingCue(status: EngineStatus, tick: number): SectionId | null {
    const transition = status.transition;
    const target = status.pendingSection ?? (transition !== null && tick < transition.startTick ? transition.to : null);
    if (target !== this.#manualCue) this.#manualCue = null;
    return this.#manualCue;
  }
}
