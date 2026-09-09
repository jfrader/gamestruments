import {
  eventsInRange,
  type MusicEvent,
  type PortableScore,
  type SectionId,
  type TransitionPlan,
} from "../../../packages/runtime/src/index.ts";

const SCHEDULE_INTERVAL_MS = 50;
const LOOKAHEAD_SECONDS = 0.3;
const MIN_GAIN = 0.0001;
const NOTE_TAIL_SECONDS = 0.16;

type NoteEvent = Extract<MusicEvent, { kind: "note" }>;
type SynthVoice = Exclude<
  NoteEvent["voice"],
  "bass" | "epiano" | "organ" | "supersaw" | "triangle" | "chip"
>;

export type SoloMode = "full" | "melody" | "rhythm";
export type TransitionCurve = "linear" | "equalPower";

export function transitionCurveForEvent(event: MusicEvent): TransitionCurve {
  return event.kind === "note" && event.role !== "melody"
    ? "linear"
    : "equalPower";
}

export function percussionBypassesTransition(score: PortableScore): boolean {
  return score.form !== undefined;
}

export function transitionGainAt(
  startLevel: number,
  targetLevel: number,
  progress: number,
  curve: TransitionCurve,
): number {
  const normalizedProgress = clamp(progress, 0, 1);
  if (curve === "linear") {
    return startLevel + (targetLevel - startLevel) * normalizedProgress;
  }
  if (targetLevel >= startLevel) {
    return (
      startLevel +
      (targetLevel - startLevel) *
        Math.sin((normalizedProgress * Math.PI) / 2)
    );
  }
  return (
    targetLevel +
    (startLevel - targetLevel) *
      Math.cos((normalizedProgress * Math.PI) / 2)
  );
}

export function createTransitionCurve(
  startLevel: number,
  targetLevel: number,
  curve: TransitionCurve,
  length = 64,
): Float32Array {
  if (!Number.isInteger(length) || length < 2) {
    throw new RangeError("Transition curves require at least two samples");
  }
  const values = new Float32Array(length);
  for (let index = 0; index < length; index += 1) {
    values[index] = transitionGainAt(
      startLevel,
      targetLevel,
      index / (length - 1),
      curve,
    );
  }
  return values;
}

export function schedulingStartTick(
  currentTick: number,
  scheduledUntil: number | undefined,
): number {
  return Math.max(currentTick, scheduledUntil ?? currentTick);
}

export function sectionSchedulingState(
  currentTick: number,
  scheduledUntil: number | undefined,
  loopOrigin: number,
): { fromTick: number; loopOrigin: number } {
  return {
    fromTick: schedulingStartTick(currentTick, scheduledUntil),
    loopOrigin,
  };
}

export function eventMatchesSolo(event: MusicEvent, mode: SoloMode): boolean {
  if (mode === "full") {
    return true;
  }
  const isMelody = event.kind === "note" && event.role === "melody";
  return mode === "melody" ? isMelody : !isMelody;
}

interface SectionBus {
  input: GainNode;
  melody: GainNode;
  tonal: GainNode;
  percussion: GainNode;
  tonalTransition: GainNode;
  equalPowerTransition: GainNode;
}

interface SynthVoiceSettings {
  primary: OscillatorType;
  secondary: OscillatorType;
  secondaryRatio: number;
  secondaryGain: number;
  detuneCents: number;
  gain: number;
  attack: number;
  decay: number;
  sustain: number;
  release: number;
  cutoffStart: number;
  cutoffEnd: number;
  resonance: number;
  width: number;
  pitchDrop: number;
}

const SYNTH_VOICES: Record<SynthVoice, SynthVoiceSettings> = {
  warm: {
    primary: "sawtooth",
    secondary: "triangle",
    secondaryRatio: 1.002,
    secondaryGain: 0.62,
    detuneCents: 10,
    gain: 0.068,
    attack: 0.048,
    decay: 0.22,
    sustain: 0.74,
    release: 0.2,
    cutoffStart: 1600,
    cutoffEnd: 620,
    resonance: 0.35,
    width: 0.28,
    pitchDrop: 0,
  },
  glass: {
    primary: "sine",
    secondary: "sine",
    secondaryRatio: 2.003,
    secondaryGain: 0.22,
    detuneCents: 6,
    gain: 0.07,
    attack: 0.01,
    decay: 0.18,
    sustain: 0.5,
    release: 0.16,
    cutoffStart: 3800,
    cutoffEnd: 1400,
    resonance: 0.45,
    width: 0.3,
    pitchDrop: 0,
  },
  pulse: {
    primary: "sawtooth",
    secondary: "triangle",
    secondaryRatio: 0.5,
    secondaryGain: 0.38,
    detuneCents: 8,
    gain: 0.05,
    attack: 0.014,
    decay: 0.14,
    sustain: 0.66,
    release: 0.14,
    cutoffStart: 1900,
    cutoffEnd: 780,
    resonance: 0.4,
    width: 0.22,
    pitchDrop: 0,
  },
  pluck: {
    primary: "sawtooth",
    secondary: "triangle",
    secondaryRatio: 2,
    secondaryGain: 0.16,
    detuneCents: 5,
    gain: 0.05,
    attack: 0.004,
    decay: 0.09,
    sustain: 0.22,
    release: 0.08,
    cutoffStart: 3400,
    cutoffEnd: 720,
    resonance: 0.9,
    width: 0.16,
    pitchDrop: 0.004,
  },
};

function midiToFrequency(pitch: number): number {
  return 440 * 2 ** ((pitch - 69) / 12);
}

function createBitcrushCurve(steps: number): Float32Array {
  const curve = new Float32Array(65536);
  const quantize = Math.max(2, steps);
  for (let index = 0; index < curve.length; index += 1) {
    const x = (index / (curve.length - 1)) * 2 - 1;
    curve[index] = Math.round(x * quantize) / quantize;
  }
  return curve;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value));
}

function deterministicUnit(seed: string): number {
  let hash = 0x811c9dc5;
  for (let index = 0; index < seed.length; index += 1) {
    hash ^= seed.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0) / 0xffffffff;
}

export class DemoAudioEngine {
  readonly #score: PortableScore;
  readonly #secondsPerTick: number;
  #context: AudioContext | null = null;
  #sectionBuses = new Map<SectionId, SectionBus>();
  #activeSections = new Set<SectionId>();
  #sectionReleaseTimers = new Map<SectionId, number>();
  #scheduledUntilBySection = new Map<SectionId, number>();
  #loopOriginBySection = new Map<SectionId, number>();
  #noiseBuffer: AudioBuffer | null = null;
  #echoSend: GainNode | null = null;
  #originTime = 0;
  #timer: number | null = null;
  #soloMode: SoloMode = "full";

  constructor(score: PortableScore) {
    this.#score = score;
    this.#secondsPerTick = 60 / score.bpm / score.ticksPerBeat;
  }

  get running(): boolean {
    return this.#context?.state === "running";
  }

  get soloMode(): SoloMode {
    return this.#soloMode;
  }

  set soloMode(mode: SoloMode) {
    this.#soloMode = mode;
    this.#applySoloMode();
  }

  async start(initialSection: SectionId): Promise<void> {
    if (this.#context !== null) {
      await this.#context.resume();
      return;
    }

    const context = new AudioContext({ latencyHint: "interactive" });
    const master = context.createGain();
    const highpass = context.createBiquadFilter();
    const compressor = context.createDynamicsCompressor();
    const limiter = context.createDynamicsCompressor();
    const roomImpulse = this.#createRoomImpulse(context);
    master.gain.value = 0.64;
    highpass.type = "highpass";
    highpass.frequency.value = 28;
    highpass.Q.value = 0.55;
    compressor.threshold.value = -18;
    compressor.knee.value = 12;
    compressor.ratio.value = 3.2;
    compressor.attack.value = 0.007;
    compressor.release.value = 0.16;
    limiter.threshold.value = -3;
    limiter.knee.value = 1;
    limiter.ratio.value = 18;
    limiter.attack.value = 0.001;
    limiter.release.value = 0.075;
    master
      .connect(highpass)
      .connect(compressor)
      .connect(limiter)
      .connect(context.destination);

    const echo = context.createDelay(1.2);
    const echoFeedback = context.createGain();
    const echoSend = context.createGain();
    echo.delayTime.value = (60 / this.#score.bpm) * 0.75;
    echoFeedback.gain.value = 0.3;
    echoSend.gain.value = 0.14;
    echoSend.connect(echo);
    echo.connect(echoFeedback).connect(echo);
    echo.connect(master);
    this.#echoSend = echoSend;

    for (const section of this.#score.sections) {
      const input = context.createGain();
      const melody = context.createGain();
      const tonal = context.createGain();
      const percussion = context.createGain();
      const tonalTransition = context.createGain();
      const equalPowerTransition = context.createGain();
      const dry = context.createGain();
      const roomHighpass = context.createBiquadFilter();
      const roomLowpass = context.createBiquadFilter();
      const room = context.createConvolver();
      const roomReturn = context.createGain();
      dry.gain.value = 0.96;
      roomHighpass.type = "highpass";
      roomHighpass.frequency.value = 190;
      roomLowpass.type = "lowpass";
      roomLowpass.frequency.value = 4800;
      room.buffer = roomImpulse;
      roomReturn.gain.value = 0.14;
      tonalTransition.gain.value = section.id === initialSection ? 1 : 0;
      equalPowerTransition.gain.value = section.id === initialSection ? 1 : 0;
      melody.connect(equalPowerTransition);
      tonal.connect(tonalTransition);
      if (percussionBypassesTransition(this.#score)) {
        percussion.connect(input);
      } else {
        percussion.connect(equalPowerTransition);
      }
      tonalTransition.connect(input);
      equalPowerTransition.connect(input);
      input.connect(dry).connect(master);
      input
        .connect(roomHighpass)
        .connect(roomLowpass)
        .connect(room)
        .connect(roomReturn)
        .connect(master);
      this.#sectionBuses.set(section.id, {
        input,
        melody,
        tonal,
        percussion,
        tonalTransition,
        equalPowerTransition,
      });
    }

    this.#context = context;
    this.#noiseBuffer = this.#createNoiseBuffer(context);
    this.#activeSections.add(initialSection);
    this.#scheduledUntilBySection.set(initialSection, 0);
    this.#loopOriginBySection.set(initialSection, 0);
    this.#applySoloMode();
    if (context.state === "suspended") {
      await context.resume();
    }
    if (this.#context !== context) {
      return;
    }
    this.#originTime = context.currentTime + 0.08;
    this.#schedule();
    this.#timer = window.setInterval(
      () => this.#schedule(),
      SCHEDULE_INTERVAL_MS,
    );
  }

  async stop(): Promise<void> {
    if (this.#timer !== null) {
      window.clearInterval(this.#timer);
      this.#timer = null;
    }
    const context = this.#context;
    this.#context = null;
    for (const timer of this.#sectionReleaseTimers.values()) {
      window.clearTimeout(timer);
    }
    this.#sectionBuses.clear();
    this.#activeSections.clear();
    this.#sectionReleaseTimers.clear();
    this.#scheduledUntilBySection.clear();
    this.#loopOriginBySection.clear();
    this.#noiseBuffer = null;
    this.#echoSend = null;
    if (context !== null && context.state !== "closed") {
      await context.close();
    }
  }

  currentTick(): number {
    return Math.floor(this.currentVisualTick());
  }

  currentVisualTick(): number {
    if (this.#context === null) {
      return 0;
    }
    return Math.max(
      0,
      (this.#context.currentTime - this.#originTime) / this.#secondsPerTick,
    );
  }

  sectionVisualTick(sectionId: SectionId, visualTick = this.currentVisualTick()): number {
    const section = this.#score.sections.find((candidate) => candidate.id === sectionId);
    if (section === undefined) {
      throw new Error(`Unknown section: ${sectionId}`);
    }
    const elapsed = visualTick - (this.#loopOriginBySection.get(sectionId) ?? 0);
    return ((elapsed % section.lengthTicks) + section.lengthTicks) % section.lengthTicks;
  }

  applyTransition(plan: TransitionPlan): void {
    const context = this.#context;
    const from = this.#sectionBuses.get(plan.from);
    const to = this.#sectionBuses.get(plan.to);
    if (
      context === null ||
      from === undefined ||
      to === undefined ||
      from === to
    ) {
      return;
    }

    const now = context.currentTime;
    const start = Math.max(
      now + 0.005,
      this.#tickToTime(plan.startTick),
    );
    const duration = Math.max(
      0.01,
      (plan.endTick - plan.startTick) * this.#secondsPerTick,
    );
    this.#activateSection(plan.from);
    this.#activateSection(plan.to, plan.startTick);
    this.#holdTransitionGains(from, now);
    this.#holdTransitionGains(to, now);
    this.#fadeSection(from, 0, start, duration);
    this.#fadeSection(to, 1, start, duration);
    this.#releaseSectionAfter(plan.from, start + duration);
  }

  cancelTransition(plan: TransitionPlan): void {
    const context = this.#context;
    const from = this.#sectionBuses.get(plan.from);
    const to = this.#sectionBuses.get(plan.to);
    if (context === null || from === undefined || to === undefined) {
      return;
    }
    const now = context.currentTime;
    this.#holdTransitionGains(from, now);
    this.#holdTransitionGains(to, now);
    this.#rampTransitionGains(from, 1, now + 0.03);
    this.#rampTransitionGains(to, 0, now + 0.03);
    this.#releaseSectionAfter(plan.to, now + 0.03);
  }

  jumpSection(target: SectionId, atTick: number): void {
    if (!this.#score.sections.some((section) => section.id === target)) {
      throw new Error(`Unknown target section: ${target}`);
    }
    const context = this.#context;
    if (context === null) {
      return;
    }
    const now = context.currentTime;
    for (const timer of this.#sectionReleaseTimers.values()) {
      window.clearTimeout(timer);
    }
    this.#sectionReleaseTimers.clear();
    this.#activeSections.clear();
    this.#loopOriginBySection.clear();
    this.#loopOriginBySection.set(target, atTick);
    this.#activeSections.add(target);
    this.#scheduledUntilBySection.set(
      target,
      schedulingStartTick(atTick, this.#scheduledUntilBySection.get(target)),
    );
    for (const [section, bus] of this.#sectionBuses) {
      this.#holdTransitionGains(bus, now);
      this.#rampTransitionGains(bus, section === target ? 1 : 0, now + 0.03);
    }
    this.#schedule(atTick);
  }

  #activateSection(section: SectionId, loopOrigin?: number): void {
    if (loopOrigin !== undefined) {
      this.#loopOriginBySection.set(section, loopOrigin);
    }
    const releaseTimer = this.#sectionReleaseTimers.get(section);
    if (releaseTimer !== undefined) {
      window.clearTimeout(releaseTimer);
      this.#sectionReleaseTimers.delete(section);
    }
    if (!this.#activeSections.has(section)) {
      this.#activeSections.add(section);
      this.#scheduledUntilBySection.set(
        section,
        schedulingStartTick(
          this.currentTick(),
          this.#scheduledUntilBySection.get(section),
        ),
      );
    }
  }

  #releaseSectionAfter(section: SectionId, atTime: number): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const existing = this.#sectionReleaseTimers.get(section);
    if (existing !== undefined) {
      window.clearTimeout(existing);
    }
    const delay = Math.max(0, (atTime - context.currentTime) * 1000) + 40;
    const timer = window.setTimeout(() => {
      this.#activeSections.delete(section);
      this.#sectionReleaseTimers.delete(section);
    }, delay);
    this.#sectionReleaseTimers.set(section, timer);
  }

  #applySoloMode(): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const now = context.currentTime;
    const melodyGain = this.#soloMode === "rhythm" ? 0 : 1;
    const rhythmGain = this.#soloMode === "melody" ? 0 : 1;
    for (const bus of this.#sectionBuses.values()) {
      this.#rampGain(bus.melody.gain, melodyGain, now, now + 0.018);
      this.#rampGain(bus.tonal.gain, rhythmGain, now, now + 0.018);
      this.#rampGain(bus.percussion.gain, rhythmGain, now, now + 0.018);
    }
  }

  #transitionGains(
    bus: SectionBus,
  ): readonly [AudioParam, TransitionCurve][] {
    return [
      [bus.tonalTransition.gain, "linear"],
      [bus.equalPowerTransition.gain, "equalPower"],
    ];
  }

  #holdTransitionGains(bus: SectionBus, atTime: number): void {
    for (const [gain] of this.#transitionGains(bus)) {
      this.#cancelAndHold(gain, atTime);
    }
  }

  #fadeSection(
    bus: SectionBus,
    target: number,
    start: number,
    duration: number,
  ): void {
    for (const [gain, curve] of this.#transitionGains(bus)) {
      const startLevel = clamp(gain.value, 0, 1);
      gain.setValueAtTime(startLevel, start);
      gain.setValueCurveAtTime(
        createTransitionCurve(startLevel, target, curve),
        start,
        duration,
      );
    }
  }

  #rampTransitionGains(
    bus: SectionBus,
    target: number,
    end: number,
  ): void {
    for (const [gain] of this.#transitionGains(bus)) {
      gain.linearRampToValueAtTime(target, end);
    }
  }

  #rampGain(
    parameter: AudioParam,
    target: number,
    start: number,
    end: number,
  ): void {
    parameter.cancelScheduledValues(start);
    parameter.setValueAtTime(parameter.value, start);
    parameter.linearRampToValueAtTime(target, end);
  }

  #cancelAndHold(parameter: AudioParam, atTime: number): void {
    const compatible = parameter as AudioParam & {
      cancelAndHoldAtTime?: (cancelTime: number) => AudioParam;
    };
    if (typeof compatible.cancelAndHoldAtTime === "function") {
      compatible.cancelAndHoldAtTime(atTime);
      return;
    }
    const currentValue = parameter.value;
    parameter.cancelScheduledValues(atTime);
    parameter.setValueAtTime(currentValue, atTime);
  }

  #schedule(currentTick = this.currentTick()): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const lookaheadTicks = Math.ceil(LOOKAHEAD_SECONDS / this.#secondsPerTick);
    const toTick = currentTick + lookaheadTicks;

    for (const section of this.#score.sections) {
      if (!this.#activeSections.has(section.id)) {
        continue;
      }
      const scheduling = sectionSchedulingState(
        currentTick,
        this.#scheduledUntilBySection.get(section.id),
        this.#loopOriginBySection.get(section.id) ?? 0,
      );
      if (toTick <= scheduling.fromTick) {
        continue;
      }
      for (const event of eventsInRange(
        section,
        scheduling.fromTick,
        toTick,
        scheduling.loopOrigin,
      )) {
        this.#scheduleEvent(event);
      }
      this.#scheduledUntilBySection.set(section.id, toTick);
    }
  }

  #scheduleEvent(event: MusicEvent): void {
    if (event.kind === "stem") {
      return;
    }
    const context = this.#context;
    const bus = this.#sectionBuses.get(event.section);
    if (context === null || bus === undefined) {
      return;
    }
    const start = Math.max(context.currentTime + 0.004, this.#tickToTime(event.startTick));
    if (event.kind === "note") {
      this.#scheduleNote(
        event,
        start,
        transitionCurveForEvent(event) === "equalPower"
          ? bus.melody
          : bus.tonal,
      );
      return;
    }
    this.#schedulePercussion(event, start, bus.percussion);
  }

  #scheduleNote(
    event: NoteEvent,
    start: number,
    destination: AudioNode,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const duration = Math.max(0.04, event.durationTicks * this.#secondsPerTick);
    const dedicated = {
      epiano: () =>
        this.#scheduleElectricPiano(event, start, duration, destination),
      bass: () => this.#scheduleBass(event, start, duration, destination),
      organ: () => this.#scheduleOrgan(event, start, duration, destination),
      supersaw: () =>
        this.#scheduleSupersaw(event, start, duration, destination),
      triangle: () =>
        this.#scheduleTriangleBass(event, start, duration, destination),
      chip: () => this.#scheduleChip(event, start, duration, destination),
    } as const;
    const scheduleDedicated = dedicated[event.voice as keyof typeof dedicated];
    if (scheduleDedicated !== undefined) {
      scheduleDedicated();
      return;
    }
    this.#scheduleSynthVoice(
      event,
      event.voice as SynthVoice,
      start,
      duration,
      destination,
    );
  }

  #scheduleSynthVoice(
    event: NoteEvent,
    voice: SynthVoice,
    start: number,
    duration: number,
    destination: AudioNode,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }

    const settings = SYNTH_VOICES[voice];
    const velocity = clamp(event.velocity, 0, 1);
    const frequency = midiToFrequency(event.pitch);
    const end = start + duration;
    const stop = end + NOTE_TAIL_SECONDS;
    const primary = context.createOscillator();
    const secondary = context.createOscillator();
    const primaryGain = context.createGain();
    const secondaryGain = context.createGain();
    const primaryPan = context.createStereoPanner();
    const secondaryPan = context.createStereoPanner();
    const filter = context.createBiquadFilter();
    const envelope = context.createGain();

    primary.type = settings.primary;
    secondary.type = settings.secondary;
    primary.detune.value = -settings.detuneCents;
    secondary.detune.value = settings.detuneCents;
    this.#schedulePitch(primary.frequency, frequency, settings.pitchDrop, start);
    this.#schedulePitch(
      secondary.frequency,
      frequency * settings.secondaryRatio,
      settings.pitchDrop * 0.5,
      start,
    );
    primaryGain.gain.value = 1;
    secondaryGain.gain.value = settings.secondaryGain;
    primaryPan.pan.value = -settings.width;
    secondaryPan.pan.value = settings.width;
    filter.type = "lowpass";
    filter.Q.value = settings.resonance + (event.role === "melody" ? 0.25 : 0);
    const nyquistSafe = context.sampleRate * 0.44;
    const brightness = 0.72 + velocity * 0.48;
    filter.frequency.setValueAtTime(
      Math.min(nyquistSafe, settings.cutoffStart * brightness),
      start,
    );
    filter.frequency.exponentialRampToValueAtTime(
      Math.max(120, settings.cutoffEnd * (0.82 + velocity * 0.28)),
      Math.min(end, start + settings.decay),
    );
    const peak =
      settings.gain *
      Math.pow(Math.max(0.02, velocity), 0.82) *
      (event.role === "melody" ? 1.18 : 1);
    this.#scheduleEnvelope(
      envelope.gain,
      start,
      duration,
      peak,
      settings.sustain,
      settings.attack,
      settings.decay,
      settings.release,
    );

    primary.connect(primaryGain).connect(primaryPan).connect(filter);
    secondary.connect(secondaryGain).connect(secondaryPan).connect(filter);
    filter.connect(envelope).connect(destination);
    primary.start(start);
    secondary.start(start);
    primary.stop(stop);
    secondary.stop(stop);
    this.#cleanupAfter(primary, [
      primary,
      secondary,
      primaryGain,
      secondaryGain,
      primaryPan,
      secondaryPan,
      filter,
      envelope,
    ]);
  }

  #scheduleBass(
    event: NoteEvent,
    start: number,
    duration: number,
    destination: AudioNode,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }

    const velocity = clamp(event.velocity, 0, 1);
    const frequency = midiToFrequency(event.pitch);
    const end = start + duration;
    const stop = end + NOTE_TAIL_SECONDS;
    const body = context.createOscillator();
    const sub = context.createOscillator();
    const bodyGain = context.createGain();
    const subGain = context.createGain();
    const filter = context.createBiquadFilter();
    const envelope = context.createGain();

    body.type = "triangle";
    sub.type = "sine";
    this.#schedulePitch(body.frequency, frequency, 0.004, start);
    this.#schedulePitch(sub.frequency, frequency * 0.5, 0, start);
    bodyGain.gain.value = 0.42 + velocity * 0.1;
    subGain.gain.value = 0.95;
    filter.type = "lowpass";
    filter.Q.value = 0.7 + velocity * 0.35;
    filter.frequency.setValueAtTime(520 + velocity * 680, start);
    filter.frequency.exponentialRampToValueAtTime(
      Math.max(180, Math.min(420, frequency * 2.4)),
      Math.min(end, start + 0.16),
    );
    this.#scheduleEnvelope(
      envelope.gain,
      start,
      duration,
      0.12 * Math.pow(Math.max(0.02, velocity), 0.78),
      0.62,
      0.008,
      0.12,
      0.11,
    );

    body.connect(bodyGain).connect(filter);
    sub.connect(subGain).connect(filter);
    filter.connect(envelope).connect(destination);
    body.start(start);
    sub.start(start);
    body.stop(stop);
    sub.stop(stop);
    this.#cleanupAfter(body, [body, sub, bodyGain, subGain, filter, envelope]);
  }

  #scheduleElectricPiano(
    event: NoteEvent,
    start: number,
    duration: number,
    destination: AudioNode,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }

    const velocity = clamp(event.velocity, 0, 1);
    const frequency = midiToFrequency(event.pitch);
    const end = start + duration;
    const stop = end + NOTE_TAIL_SECONDS;
    const bodyLeft = context.createOscillator();
    const bodyRight = context.createOscillator();
    const tine = context.createOscillator();
    const bodyLeftGain = context.createGain();
    const bodyRightGain = context.createGain();
    const bodyLeftPan = context.createStereoPanner();
    const bodyRightPan = context.createStereoPanner();
    const tineGain = context.createGain();
    const filter = context.createBiquadFilter();
    const envelope = context.createGain();
    const tremolo = context.createGain();
    const tremoloOscillator = context.createOscillator();
    const tremoloDepth = context.createGain();
    const tinePeak = 0.11 + Math.pow(velocity, 1.7) * 0.38;
    const tineDecay = Math.min(end, start + 0.09 + (1 - velocity) * 0.08);

    bodyLeft.type = "sine";
    bodyRight.type = "sine";
    bodyLeft.frequency.value = frequency;
    bodyRight.frequency.value = frequency;
    bodyLeft.detune.value = -7;
    bodyRight.detune.value = 7;
    tine.type = "sine";
    tine.frequency.value = frequency * (2.001 + velocity * 0.003);
    tine.detune.value = 3;
    bodyLeftGain.gain.value = 0.62;
    bodyRightGain.gain.value = 0.62;
    bodyLeftPan.pan.value = -0.34;
    bodyRightPan.pan.value = 0.34;
    tineGain.gain.setValueAtTime(MIN_GAIN, start);
    tineGain.gain.exponentialRampToValueAtTime(tinePeak * 0.7, start + 0.004);
    tineGain.gain.exponentialRampToValueAtTime(0.012, tineDecay);
    tineGain.gain.setValueAtTime(0.012, end);
    filter.type = "lowpass";
    filter.Q.value = 0.45 + velocity * 0.35;
    filter.frequency.setValueAtTime(1100 + Math.pow(velocity, 1.4) * 2200, start);
    filter.frequency.exponentialRampToValueAtTime(
      780 + velocity * 420,
      Math.min(end, start + 0.28),
    );
    this.#scheduleEnvelope(
      envelope.gain,
      start,
      duration,
      0.12 * Math.pow(Math.max(0.02, velocity), 0.78),
      0.48 + (1 - velocity) * 0.12,
      0.012,
      0.18 + (1 - velocity) * 0.08,
      0.16,
    );
    tremolo.gain.value = 0.975;
    tremoloOscillator.type = "sine";
    tremoloOscillator.frequency.value = 4.65 + (event.pitch % 5) * 0.07;
    tremoloDepth.gain.value = 0.018 + velocity * 0.008;
    tremoloOscillator.connect(tremoloDepth).connect(tremolo.gain);

    bodyLeft.connect(bodyLeftGain).connect(bodyLeftPan).connect(filter);
    bodyRight.connect(bodyRightGain).connect(bodyRightPan).connect(filter);
    tine.connect(tineGain).connect(filter);
    filter.connect(envelope).connect(tremolo).connect(destination);
    bodyLeft.start(start);
    bodyRight.start(start);
    tine.start(start);
    tremoloOscillator.start(start);
    bodyLeft.stop(stop);
    bodyRight.stop(stop);
    tine.stop(stop);
    tremoloOscillator.stop(stop);
    this.#cleanupAfter(bodyLeft, [
      bodyLeft,
      bodyRight,
      tine,
      tremoloOscillator,
      bodyLeftGain,
      bodyRightGain,
      bodyLeftPan,
      bodyRightPan,
      tineGain,
      filter,
      envelope,
      tremolo,
      tremoloDepth,
    ]);
  }

  #scheduleOrgan(
    event: NoteEvent,
    start: number,
    duration: number,
    destination: AudioNode,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const velocity = clamp(event.velocity, 0, 1);
    const frequency = midiToFrequency(event.pitch);
    const stop = start + duration + NOTE_TAIL_SECONDS;
    const draws = [1, 2, 3] as const;
    const gains = [0.42, 0.28, 0.16] as const;
    const mix = context.createGain();
    const envelope = context.createGain();
    const tremolo = context.createGain();
    const tremoloOscillator = context.createOscillator();
    const tremoloDepth = context.createGain();
    const oscillators = draws.map((ratio, index) => {
      const oscillator = context.createOscillator();
      const gain = context.createGain();
      oscillator.type = "sine";
      oscillator.frequency.value = frequency * ratio;
      gain.gain.value = gains[index] ?? 0.2;
      oscillator.connect(gain).connect(mix);
      oscillator.start(start);
      oscillator.stop(stop);
      return { oscillator, gain };
    });
    this.#scheduleEnvelope(
      envelope.gain,
      start,
      duration,
      (event.role === "melody" ? 0.09 : 0.034) *
        Math.pow(Math.max(0.02, velocity), 0.8),
      0.7,
      0.03,
      0.18,
      0.2,
    );
    tremolo.gain.value = 0.92;
    tremoloOscillator.type = "sine";
    tremoloOscillator.frequency.value = 5.4;
    tremoloDepth.gain.value = 0.08;
    tremoloOscillator.connect(tremoloDepth).connect(tremolo.gain);
    mix.connect(envelope).connect(tremolo).connect(destination);
    tremoloOscillator.start(start);
    tremoloOscillator.stop(stop);
    this.#cleanupAfter(oscillators[0]?.oscillator ?? tremoloOscillator, [
      ...oscillators.flatMap(({ oscillator, gain }) => [oscillator, gain]),
      mix,
      envelope,
      tremolo,
      tremoloOscillator,
      tremoloDepth,
    ]);
  }

  #scheduleSupersaw(
    event: NoteEvent,
    start: number,
    duration: number,
    destination: AudioNode,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const velocity = clamp(event.velocity, 0, 1);
    const frequency = midiToFrequency(event.pitch);
    const stop = start + duration + NOTE_TAIL_SECONDS;
    const detunes = [-11, 0, 13] as const;
    const filter = context.createBiquadFilter();
    const envelope = context.createGain();
    const mix = context.createGain();
    const oscillators = detunes.map((cents) => {
      const oscillator = context.createOscillator();
      oscillator.type = "sawtooth";
      oscillator.frequency.value = frequency;
      oscillator.detune.value = cents;
      oscillator.connect(mix);
      oscillator.start(start);
      oscillator.stop(stop);
      return oscillator;
    });
    filter.type = "lowpass";
    filter.Q.value = 0.4;
    filter.frequency.setValueAtTime(2400 + velocity * 900, start);
    filter.frequency.exponentialRampToValueAtTime(
      1100,
      Math.min(start + duration, start + 0.22),
    );
    this.#scheduleEnvelope(
      envelope.gain,
      start,
      duration,
      (event.role === "melody" ? 0.034 : 0.016) *
        Math.pow(Math.max(0.02, velocity), 0.8),
      0.62,
      0.02,
      0.14,
      0.16,
    );
    mix.connect(filter).connect(envelope).connect(destination);
    if (this.#echoSend !== null) {
      envelope.connect(this.#echoSend);
    }
    const lead = oscillators[0];
    if (lead === undefined) {
      return;
    }
    this.#cleanupAfter(lead, [...oscillators, mix, filter, envelope]);
  }

  #scheduleTriangleBass(
    event: NoteEvent,
    start: number,
    duration: number,
    destination: AudioNode,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const velocity = clamp(event.velocity, 0, 1);
    const frequency = midiToFrequency(event.pitch);
    const stop = start + duration + NOTE_TAIL_SECONDS;
    const body = context.createOscillator();
    const envelope = context.createGain();
    body.type = "triangle";
    body.frequency.value = frequency;
    this.#scheduleEnvelope(
      envelope.gain,
      start,
      duration,
      0.1 * Math.pow(Math.max(0.02, velocity), 0.75),
      0.7,
      0.004,
      0.05,
      0.04,
    );
    body.connect(envelope).connect(destination);
    body.start(start);
    body.stop(stop);
    this.#cleanupAfter(body, [body, envelope]);
  }

  #scheduleChip(
    event: NoteEvent,
    start: number,
    duration: number,
    destination: AudioNode,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const velocity = clamp(event.velocity, 0, 1);
    const frequency = midiToFrequency(event.pitch);
    const stop = start + duration + NOTE_TAIL_SECONDS;
    const primary = context.createOscillator();
    const octave = context.createOscillator();
    const octaveGain = context.createGain();
    const crush = context.createWaveShaper();
    const envelope = context.createGain();
    const vibrato = context.createOscillator();
    const vibratoDepth = context.createGain();
    primary.type = "square";
    octave.type = "square";
    primary.frequency.value = frequency;
    octave.frequency.value = frequency * 2;
    const isLead = event.role === "melody";
    octaveGain.gain.value = isLead ? 0.12 : 0.04;
    crush.curve = createBitcrushCurve(isLead ? 28 : 48) as Float32Array<ArrayBuffer>;
    crush.oversample = "none";
    vibrato.type = "sine";
    vibrato.frequency.value = isLead ? 5.7 : 0.8;
    vibratoDepth.gain.value = isLead ? 16 : 4;
    vibrato.connect(vibratoDepth).connect(primary.detune);
    this.#scheduleEnvelope(
      envelope.gain,
      start,
      duration,
      (isLead ? 0.032 : 0.011) * Math.pow(Math.max(0.02, velocity), 0.82),
      0.55,
      0.003,
      0.04,
      0.03,
    );
    primary.connect(crush);
    octave.connect(octaveGain).connect(crush);
    crush.connect(envelope).connect(destination);
    primary.start(start);
    octave.start(start);
    vibrato.start(start);
    primary.stop(stop);
    octave.stop(stop);
    vibrato.stop(stop);
    this.#cleanupAfter(primary, [
      primary,
      octave,
      octaveGain,
      crush,
      envelope,
      vibrato,
      vibratoDepth,
    ]);
  }

  #schedulePercussion(
    event: Extract<MusicEvent, { kind: "percussion" }>,
    start: number,
    destination: AudioNode,
  ): void {
    const schedulers = {
      kick: () =>
        this.#scheduleKick(event.velocity, start, destination, event.id),
      snare: () =>
        this.#scheduleSnare(event.velocity, start, destination, event.id),
      hat: () =>
        this.#scheduleHat(event.velocity, start, destination, event.id),
      tom: () =>
        this.#scheduleTom(event.velocity, start, destination, event.id),
    };
    schedulers[event.voice]();
  }

  #scheduleKick(
    eventVelocity: number,
    start: number,
    destination: AudioNode,
    seed: string,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const velocity = clamp(eventVelocity, 0, 1);
    const oscillator = context.createOscillator();
    const envelope = context.createGain();
    oscillator.type = "sine";
    oscillator.frequency.setValueAtTime(162 + velocity * 18, start);
    oscillator.frequency.exponentialRampToValueAtTime(49, start + 0.12);
    envelope.gain.setValueAtTime(Math.max(MIN_GAIN, 0.27 * velocity), start);
    envelope.gain.exponentialRampToValueAtTime(MIN_GAIN, start + 0.22);
    oscillator.connect(envelope).connect(destination);
    oscillator.start(start);
    oscillator.stop(start + 0.225);
    this.#scheduleNoise(
      0.045 * velocity,
      start,
      0.018,
      "highpass",
      4800,
      0.65,
      destination,
      `${seed}:click`,
    );
    this.#cleanupAfter(oscillator, [oscillator, envelope]);
  }

  #scheduleSnare(
    eventVelocity: number,
    start: number,
    destination: AudioNode,
    seed: string,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const velocity = clamp(eventVelocity, 0, 1);
    const tone = context.createOscillator();
    const toneEnvelope = context.createGain();
    tone.type = "triangle";
    tone.frequency.setValueAtTime(205, start);
    tone.frequency.exponentialRampToValueAtTime(142, start + 0.085);
    toneEnvelope.gain.setValueAtTime(
      Math.max(MIN_GAIN, 0.105 * velocity),
      start,
    );
    toneEnvelope.gain.exponentialRampToValueAtTime(MIN_GAIN, start + 0.12);
    tone.connect(toneEnvelope).connect(destination);
    tone.start(start);
    tone.stop(start + 0.125);
    this.#scheduleNoise(
      0.205 * velocity,
      start,
      0.16,
      "bandpass",
      2350,
      0.72,
      destination,
      `${seed}:snare`,
    );
    this.#cleanupAfter(tone, [tone, toneEnvelope]);
  }

  #scheduleHat(
    eventVelocity: number,
    start: number,
    destination: AudioNode,
    seed: string,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const velocity = clamp(eventVelocity, 0, 1);
    this.#scheduleNoise(
      0.12 * velocity,
      start,
      0.08,
      "highpass",
      6200,
      0.35,
      destination,
      `${seed}:hat`,
    );
  }

  #scheduleTom(
    eventVelocity: number,
    start: number,
    destination: AudioNode,
    seed: string,
  ): void {
    const context = this.#context;
    if (context === null) {
      return;
    }
    const velocity = clamp(eventVelocity, 0, 1);
    const startFrequency = 155 + deterministicUnit(seed) * 58;
    const body = context.createOscillator();
    const overtone = context.createOscillator();
    const overtoneGain = context.createGain();
    const envelope = context.createGain();
    body.type = "sine";
    overtone.type = "triangle";
    body.frequency.setValueAtTime(startFrequency, start);
    body.frequency.exponentialRampToValueAtTime(startFrequency * 0.58, start + 0.15);
    overtone.frequency.setValueAtTime(startFrequency * 1.63, start);
    overtone.frequency.exponentialRampToValueAtTime(
      startFrequency * 0.92,
      start + 0.11,
    );
    overtoneGain.gain.value = 0.23;
    envelope.gain.setValueAtTime(Math.max(MIN_GAIN, 0.17 * velocity), start);
    envelope.gain.exponentialRampToValueAtTime(MIN_GAIN, start + 0.19);
    body.connect(envelope);
    overtone.connect(overtoneGain).connect(envelope);
    envelope.connect(destination);
    body.start(start);
    overtone.start(start);
    body.stop(start + 0.195);
    overtone.stop(start + 0.195);
    this.#scheduleNoise(
      0.032 * velocity,
      start,
      0.026,
      "bandpass",
      1750,
      0.8,
      destination,
      `${seed}:tom`,
    );
    this.#cleanupAfter(body, [body, overtone, overtoneGain, envelope]);
  }

  #scheduleNoise(
    gain: number,
    start: number,
    duration: number,
    filterType: BiquadFilterType,
    frequency: number,
    resonance: number,
    destination: AudioNode,
    seed: string,
  ): void {
    const context = this.#context;
    if (context === null || this.#noiseBuffer === null) {
      return;
    }
    const source = context.createBufferSource();
    const filter = context.createBiquadFilter();
    const envelope = context.createGain();
    source.buffer = this.#noiseBuffer;
    filter.type = filterType;
    filter.frequency.value = frequency;
    filter.Q.value = resonance;
    envelope.gain.setValueAtTime(Math.max(MIN_GAIN, gain), start);
    envelope.gain.exponentialRampToValueAtTime(MIN_GAIN, start + duration);
    source.connect(filter).connect(envelope).connect(destination);
    const availableOffset = Math.max(
      0,
      this.#noiseBuffer.duration - duration - 0.005,
    );
    source.start(start, deterministicUnit(seed) * availableOffset);
    source.stop(start + duration);
    this.#cleanupAfter(source, [source, filter, envelope]);
  }

  #schedulePitch(
    parameter: AudioParam,
    frequency: number,
    pitchDrop: number,
    start: number,
  ): void {
    parameter.setValueAtTime(frequency * (1 + pitchDrop), start);
    if (pitchDrop > 0) {
      parameter.exponentialRampToValueAtTime(frequency, start + 0.022);
    }
  }

  #scheduleEnvelope(
    parameter: AudioParam,
    start: number,
    duration: number,
    peak: number,
    sustain: number,
    attack: number,
    decay: number,
    release: number,
  ): void {
    const end = start + duration;
    const attackEnd = start + clamp(attack, 0.001, duration * 0.24);
    const decayEnd = Math.min(
      start + duration * 0.7,
      attackEnd + clamp(decay, 0.001, duration * 0.46),
    );
    const releaseTime = clamp(release, 0.02, 0.24);
    const safePeak = Math.max(MIN_GAIN, peak);
    const sustainGain = Math.max(MIN_GAIN, safePeak * sustain);
    parameter.setValueAtTime(MIN_GAIN, start);
    parameter.exponentialRampToValueAtTime(safePeak, attackEnd);
    parameter.exponentialRampToValueAtTime(sustainGain, decayEnd);
    parameter.setValueAtTime(sustainGain, end);
    parameter.exponentialRampToValueAtTime(MIN_GAIN, end + releaseTime);
  }

  #cleanupAfter(
    source: AudioScheduledSourceNode,
    nodes: readonly AudioNode[],
  ): void {
    source.addEventListener(
      "ended",
      () => {
        for (const node of nodes) {
          node.disconnect();
        }
      },
      { once: true },
    );
  }

  #createNoiseBuffer(context: AudioContext): AudioBuffer {
    const buffer = context.createBuffer(1, context.sampleRate * 0.75, context.sampleRate);
    const data = buffer.getChannelData(0);
    let state = 0x12345678;
    for (let index = 0; index < data.length; index += 1) {
      state ^= state << 13;
      state ^= state >>> 17;
      state ^= state << 5;
      data[index] = ((state >>> 0) / 0xffffffff) * 2 - 1;
    }
    return buffer;
  }

  #createRoomImpulse(context: AudioContext): AudioBuffer {
    const length = Math.floor(context.sampleRate * 0.32);
    const impulse = context.createBuffer(2, length, context.sampleRate);
    for (let channel = 0; channel < impulse.numberOfChannels; channel += 1) {
      const data = impulse.getChannelData(channel);
      let state = 0x6d2b79f5 ^ (channel * 0x9e3779b9);
      for (let index = 0; index < data.length; index += 1) {
        state ^= state << 13;
        state ^= state >>> 17;
        state ^= state << 5;
        const noise = ((state >>> 0) / 0xffffffff) * 2 - 1;
        const progress = index / data.length;
        data[index] = noise * 0.28 * (1 - progress) ** 2.7;
      }
      const firstReflection = Math.floor(
        context.sampleRate * (0.011 + channel * 0.002),
      );
      const secondReflection = Math.floor(
        context.sampleRate * (0.027 - channel * 0.003),
      );
      data[firstReflection] = (data[firstReflection] ?? 0) + 0.65;
      data[secondReflection] = (data[secondReflection] ?? 0) - 0.38;
    }
    return impulse;
  }

  #tickToTime(tick: number): number {
    return this.#originTime + tick * this.#secondsPerTick;
  }
}
