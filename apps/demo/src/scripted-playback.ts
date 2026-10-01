import {
  AdaptiveTransport,
  selectSection,
  type SectionId,
  type TransitionRequest,
} from "../../../packages/runtime/src/index.ts";
import { DemoAudioEngine, type SoloMode } from "./audio-engine.ts";
import { playbackSectionOnScore } from "./playback-section.ts";
import { soundingSection, type LabPlayback, type LabScore, type PlaybackFrame } from "./playback.ts";
import type { SignalReadings } from "./recipes.ts";

/** The browser synth: the Lab's TypeScript transport schedules Web Audio
 *  voices for each section. */
export class ScriptedPlayback implements LabPlayback {
  readonly #lab: LabScore;
  #transport: AdaptiveTransport;
  readonly #audio: DemoAudioEngine;
  #manualCue: SectionId | null = null;

  constructor(lab: LabScore, opening: SectionId, held: boolean) {
    this.#lab = lab;
    this.#transport = this.#transportAt(opening, held);
    this.#audio = new DemoAudioEngine(lab.score);
  }

  get running(): boolean {
    return this.#audio.running;
  }

  get volume(): number {
    return this.#audio.volume;
  }

  set volume(value: number) {
    this.#audio.volume = value;
  }

  get soloMode(): SoloMode {
    return this.#audio.soloMode;
  }

  set soloMode(mode: SoloMode) {
    this.#audio.soloMode = mode;
  }

  async start(): Promise<void> {
    await this.#audio.start(
      this.#transport.snapshot().currentSection,
      this.#lab.score.form === undefined ? undefined : this.#transport.advance.bind(this.#transport),
    );
  }

  async stop(): Promise<void> {
    this.#transport = this.#transportAt(this.#sounding(), this.#transport.formHeld);
    this.#manualCue = null;
    await this.#audio.stop();
  }

  frame(): PlaybackFrame {
    const score = this.#lab.score;
    const tick = this.#audio.currentVisualTick();
    if (score.form === undefined) {
      const plan = this.#transport.advance(Math.floor(tick));
      if (plan !== null) this.#audio.applyTransition(plan);
    }
    return {
      score,
      tick,
      snapshot: this.#transport.snapshot(),
      mix: this.#transport.mixAt(Math.floor(tick)),
      formHeld: this.#transport.formHeld,
      nextFormSection: this.#transport.nextFormSection(Math.floor(tick)),
      requestedCue: this.#pendingCue(),
      sectionTick: (section) => this.#audio.sectionVisualTick(section, tick),
    };
  }

  cue(target: SectionId): void {
    if (!this.running) {
      this.#manualCue = null;
      this.#transport = this.#transportAt(target, this.#transport.formHeld);
      return;
    }
    const tick = this.#advance();
    this.#apply(this.#transport.requestSection(target, tick));
    const transition = this.#transport.snapshot().transition;
    if (transition?.to === target && tick < transition.startTick) this.#manualCue = target;
  }

  request(phase: string, readings: SignalReadings): void {
    const state = this.#lab.profile.gameState(phase, readings);
    if (!this.running) {
      this.cue(selectSection(this.#lab.score, state));
      return;
    }
    this.#apply(this.#transport.requestState(state, this.#advance()));
  }

  setHold(held: boolean): void {
    const cancelled = this.#transport.setFormHeld(held, this.#audio.currentTick());
    if (cancelled !== null && this.running) this.#audio.cancelTransition(cancelled);
  }

  advance(): void {
    if (this.#transport.snapshot().transition !== null) return;
    const target = this.#transport.nextFormSection(this.#audio.currentTick());
    if (target !== null) this.cue(target);
  }

  cancelCue(): void {
    if (this.#pendingCue() === null) return;
    const plan = this.#transport.cancelPending(this.#audio.currentTick());
    if (plan !== null) this.#audio.cancelTransition(plan);
    this.#manualCue = null;
  }

  async switchTo(next: LabScore): Promise<LabPlayback> {
    const opening = playbackSectionOnScore(next.score, this.#sounding());
    const playback = new ScriptedPlayback(next, opening, this.#transport.formHeld);
    playback.soloMode = this.soloMode;
    await Promise.all([this.running ? playback.start() : Promise.resolve(), this.#audio.stop()]);
    return playback;
  }

  #transportAt(section: SectionId, held: boolean): AdaptiveTransport {
    const transport = new AdaptiveTransport(this.#lab.score, section);
    transport.setFormHeld(held, 0);
    return transport;
  }

  #sounding(): SectionId {
    return soundingSection(this.#transport.snapshot(), this.#audio.currentTick());
  }

  /** Bring the transport up to the audible tick before a request. */
  #advance(): number {
    const tick = this.#audio.currentTick();
    const automatic = this.#transport.advance(tick);
    if (automatic !== null) this.#audio.applyTransition(automatic);
    return tick;
  }

  #apply(request: TransitionRequest): void {
    if (request.status === "scheduled") {
      if (request.replacedPlan !== undefined) this.#audio.cancelTransition(request.replacedPlan);
      this.#manualCue = request.plan.to;
      this.#audio.applyTransition(request.plan);
    } else if (request.status === "queued") {
      this.#manualCue = request.target;
    } else if (request.status === "cancelled") {
      this.#manualCue = null;
      this.#audio.cancelTransition(request.plan);
    }
  }

  #pendingCue(): SectionId | null {
    const snapshot = this.#transport.snapshot();
    const target =
      snapshot.pendingSection ??
      (snapshot.transition !== null && this.#audio.currentTick() < snapshot.transition.startTick
        ? snapshot.transition.to
        : null);
    if (target !== this.#manualCue) this.#manualCue = null;
    return this.#manualCue;
  }
}
