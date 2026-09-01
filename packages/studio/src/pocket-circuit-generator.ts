import type { NoteEvent, PortableScore } from "@gamestruments/runtime";
import {
  exportScore,
  type AuthoringScore,
  type AuthoringSection,
} from "./export-score.js";
import { pocketCircuitRules } from "./pocket-circuit-score.js";

export const POCKET_CIRCUIT_GENERATOR_VERSION = "1.10.0" as const;
export const POCKET_CIRCUIT_DNA_SEED_VERSION = "1.1.0" as const;

export const POCKET_CIRCUIT_GENERATOR_DOMAINS = [
  "harmony",
  "motif",
  "rhythm",
  "timbre",
  "arrangement",
  "ornaments",
] as const;

export type PocketCircuitGeneratorDomain =
  (typeof POCKET_CIRCUIT_GENERATOR_DOMAINS)[number];

export type LevelSeed = string | number;
export type PocketCircuitStyle = "fusion" | "neon" | "funk" | "chip";

export interface NormalizedMusicTraits {
  energy: number;
  complexity: number;
  brightness: number;
  syncopation: number;
}

export const DEFAULT_POCKET_CIRCUIT_TRAITS = {
  energy: 0.6,
  complexity: 0.58,
  brightness: 0.52,
  syncopation: 0.64,
} as const satisfies NormalizedMusicTraits;

export interface PocketCircuitGeneratorInput {
  seed: LevelSeed;
  style?: PocketCircuitStyle;
  traits?: Partial<NormalizedMusicTraits>;
}

export type PocketCircuitDomainSeeds = Record<
  PocketCircuitGeneratorDomain,
  number
>;

export type PocketCircuitMode =
  | "natural-minor"
  | "dorian"
  | "mixolydian"
  | "lydian";

export interface PocketCircuitHarmonyDNA {
  key: string;
  rootPitchClass: number;
  mode: PocketCircuitMode;
  scaleIntervals: readonly number[];
  progressionDegrees: readonly number[];
  chordSize: 3 | 4;
}

export interface PocketCircuitMotifDNA {
  degrees: readonly number[];
  anchorLength: number;
}

export interface PocketCircuitRhythmDNA {
  pulse: "eighth-note";
  stepsPerBar: 8;
  pulseTicks: 480;
  melodyOnsets: readonly number[];
  kickOnsets: readonly number[];
  snareOnsets: readonly number[];
}

type HarmonyVoice = Extract<
  NoteEvent["voice"],
  "warm" | "pluck" | "pulse" | "chip" | "organ"
>;
type MelodyVoice = Extract<
  NoteEvent["voice"],
  "epiano" | "glass" | "pulse" | "chip" | "pluck" | "supersaw"
>;
type BassVoice = Extract<NoteEvent["voice"], "bass" | "triangle">;

export interface PocketCircuitTimbreDNA {
  harmonyVoice: HarmonyVoice;
  driveHarmonyVoice: HarmonyVoice;
  melodyVoice: MelodyVoice;
  liftVoice: MelodyVoice;
  bassVoice: BassVoice;
}

export interface PocketCircuitArrangementDNA {
  bpm: number;
  bassApproach: "root-fifth" | "root-octave" | "fifth-root";
  chordGate: number;
  melodyGate: number;
}

export interface PocketCircuitOrnamentDNA {
  barRotations: readonly [number, number, number, number];
  passingOffsets: readonly [number, number, number, number];
  turnaroundStep: number;
}

export interface PocketCircuitMusicalDNA {
  generatorVersion: typeof POCKET_CIRCUIT_GENERATOR_VERSION;
  levelSeed: string;
  style: PocketCircuitStyle;
  traits: NormalizedMusicTraits;
  domainSeeds: PocketCircuitDomainSeeds;
  harmony: PocketCircuitHarmonyDNA;
  motif: PocketCircuitMotifDNA;
  rhythm: PocketCircuitRhythmDNA;
  timbre: PocketCircuitTimbreDNA;
  arrangement: PocketCircuitArrangementDNA;
  ornaments: PocketCircuitOrnamentDNA;
}

export interface GeneratedPocketCircuitLevel {
  generatorVersion: typeof POCKET_CIRCUIT_GENERATOR_VERSION;
  levelSeed: string;
  style: PocketCircuitStyle;
  traits: NormalizedMusicTraits;
  domainSeeds: PocketCircuitDomainSeeds;
  dna: PocketCircuitMusicalDNA;
  authoringScore: AuthoringScore;
  portableScore: PortableScore;
}

const NOTE_NAMES = [
  "c",
  "c#",
  "d",
  "d#",
  "e",
  "f",
  "f#",
  "g",
  "g#",
  "a",
  "a#",
  "b",
] as const;

const KEY_PITCH_CLASSES = [0, 2, 3, 5, 7, 9, 10] as const;

const MODE_INTERVALS: Readonly<Record<PocketCircuitMode, readonly number[]>> = {
  "natural-minor": [0, 2, 3, 5, 7, 8, 10],
  dorian: [0, 2, 3, 5, 7, 9, 10],
  mixolydian: [0, 2, 4, 5, 7, 9, 10],
  lydian: [0, 2, 4, 6, 7, 9, 11],
};

const PROGRESSIONS = [
  [0, 5, 3, 4],
  [0, 3, 5, 4],
  [0, 4, 5, 3],
  [0, 2, 5, 4],
] as const;

const MOTIF_CONTOURS = [
  [0, 2, 4, 3, 1, 5, 4, 0],
  [0, 3, 2, 5, 4, 2, 1, 0],
  [0, 1, 4, 2, 5, 3, 1, 0],
  [0, 4, 3, 1, 2, 5, 4, 0],
] as const;

export interface PocketCircuitSectionPlan {
  id: string;
  label: string;
  feeling: string;
  color: string;
  intensity: number;
  development: number;
  lift: number;
  registerShift: number;
  melodyGateScale: number;
  chordGateScale: number;
  bassGateScale: number;
  finalAccent: "hat" | "tom";
}

export const POCKET_CIRCUIT_SECTION_PLANS: readonly PocketCircuitSectionPlan[] = [
  {
    id: "garage",
    label: "Garage",
    feeling: "polished / unhurried",
    color: "#dbc89a",
    intensity: 0.55,
    development: 0,
    lift: 0,
    registerShift: -7,
    melodyGateScale: 1.65,
    chordGateScale: 1.18,
    bassGateScale: 1.22,
    finalAccent: "hat",
  },
  {
    id: "grid",
    label: "Starting Grid",
    feeling: "focus / anticipation",
    color: "#f4b740",
    intensity: 0.7,
    development: 1,
    lift: 0,
    registerShift: -2,
    melodyGateScale: 1.12,
    chordGateScale: 0.82,
    bassGateScale: 0.88,
    finalAccent: "hat",
  },
  {
    id: "cruise",
    label: "Race Flow",
    feeling: "precision / forward motion",
    color: "#73d7c7",
    intensity: 0.82,
    development: 2,
    lift: 0,
    registerShift: 7,
    melodyGateScale: 0.72,
    chordGateScale: 1.12,
    bassGateScale: 0.9,
    finalAccent: "hat",
  },
  {
    id: "attack",
    label: "Position Fight",
    feeling: "pressure / resolve",
    color: "#f06c4f",
    intensity: 0.95,
    development: 3,
    lift: 1,
    registerShift: 7,
    melodyGateScale: 0.52,
    chordGateScale: 1.05,
    bassGateScale: 0.7,
    finalAccent: "hat",
  },
  {
    id: "final-lap",
    label: "Final Lap",
    feeling: "maximum commitment",
    color: "#ef3f50",
    intensity: 1.08,
    development: 4,
    lift: 0,
    registerShift: 12,
    melodyGateScale: 0.55,
    chordGateScale: 1.08,
    bassGateScale: 0.72,
    finalAccent: "tom",
  },
  {
    id: "victory",
    label: "Finish",
    feeling: "release / earned confidence",
    color: "#e7d46a",
    intensity: 0.78,
    development: 5,
    lift: 0,
    registerShift: 0,
    melodyGateScale: 1.48,
    chordGateScale: 1.2,
    bassGateScale: 1.2,
    finalAccent: "tom",
  },
];

class DeterministicRandom {
  #state: number;

  constructor(seed: number) {
    this.#state = seed === 0 ? 0x6d2b79f5 : seed;
  }

  next(): number {
    this.#state = (this.#state + 0x6d2b79f5) >>> 0;
    let value = this.#state;
    value = Math.imul(value ^ (value >>> 15), value | 1);
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61);
    return ((value ^ (value >>> 14)) >>> 0) / 0x1_0000_0000;
  }

  integer(maxExclusive: number): number {
    return Math.floor(this.next() * maxExclusive);
  }

  pick<T>(values: readonly T[]): T {
    const value = values[this.integer(values.length)];
    if (value === undefined) {
      throw new Error("Cannot choose from an empty collection");
    }
    return value;
  }

  shuffle<T>(values: readonly T[]): T[] {
    const shuffled = [...values];
    for (let index = shuffled.length - 1; index > 0; index -= 1) {
      const other = this.integer(index + 1);
      const currentValue = shuffled[index];
      const otherValue = shuffled[other];
      if (currentValue === undefined || otherValue === undefined) {
        throw new Error("Cannot shuffle an incomplete collection");
      }
      shuffled[index] = otherValue;
      shuffled[other] = currentValue;
    }
    return shuffled;
  }
}

function hashText(value: string): number {
  let hash = 0x811c9dc5;
  for (const character of value) {
    hash ^= character.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 0x01000193);
  }
  hash ^= hash >>> 16;
  hash = Math.imul(hash, 0x7feb352d);
  hash ^= hash >>> 15;
  hash = Math.imul(hash, 0x846ca68b);
  return (hash ^ (hash >>> 16)) >>> 0;
}

function canonicalizeSeed(seed: LevelSeed): string {
  if (typeof seed === "number") {
    if (!Number.isSafeInteger(seed)) {
      throw new Error(`Numeric level seed must be a safe integer: ${seed}`);
    }
    return `number:${seed}`;
  }
  return `string:${seed}`;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.max(minimum, Math.min(maximum, value));
}

function clampUnit(value: number, name: keyof NormalizedMusicTraits): number {
  if (!Number.isFinite(value)) {
    throw new Error(`Music trait ${name} must be finite: ${value}`);
  }
  return clamp(value, 0, 1);
}

export function normalizeMusicTraits(
  traits: Partial<NormalizedMusicTraits> = {},
): NormalizedMusicTraits {
  return {
    energy: clampUnit(traits.energy ?? DEFAULT_POCKET_CIRCUIT_TRAITS.energy, "energy"),
    complexity: clampUnit(
      traits.complexity ?? DEFAULT_POCKET_CIRCUIT_TRAITS.complexity,
      "complexity",
    ),
    brightness: clampUnit(
      traits.brightness ?? DEFAULT_POCKET_CIRCUIT_TRAITS.brightness,
      "brightness",
    ),
    syncopation: clampUnit(
      traits.syncopation ?? DEFAULT_POCKET_CIRCUIT_TRAITS.syncopation,
      "syncopation",
    ),
  };
}

export function derivePocketCircuitSubSeed(
  levelSeed: LevelSeed,
  domain: PocketCircuitGeneratorDomain,
): number {
  return hashText(
    `${POCKET_CIRCUIT_DNA_SEED_VERSION}\u0000${canonicalizeSeed(levelSeed)}\u0000${domain}`,
  );
}

export function derivePocketCircuitDomainSeeds(
  levelSeed: LevelSeed,
): PocketCircuitDomainSeeds {
  return {
    harmony: derivePocketCircuitSubSeed(levelSeed, "harmony"),
    motif: derivePocketCircuitSubSeed(levelSeed, "motif"),
    rhythm: derivePocketCircuitSubSeed(levelSeed, "rhythm"),
    timbre: derivePocketCircuitSubSeed(levelSeed, "timbre"),
    arrangement: derivePocketCircuitSubSeed(levelSeed, "arrangement"),
    ornaments: derivePocketCircuitSubSeed(levelSeed, "ornaments"),
  };
}

function selectMode(
  random: DeterministicRandom,
  brightness: number,
): PocketCircuitMode {
  if (brightness < 0.34) {
    return random.pick(["natural-minor", "dorian"] as const);
  }
  if (brightness > 0.66) {
    return random.pick(["mixolydian", "lydian"] as const);
  }
  return random.pick(["dorian", "mixolydian"] as const);
}

function createHarmonyDNA(
  seed: number,
  traits: NormalizedMusicTraits,
): PocketCircuitHarmonyDNA {
  const random = new DeterministicRandom(seed);
  const rootPitchClass = random.pick(KEY_PITCH_CLASSES);
  const mode = selectMode(random, traits.brightness);
  return {
    key: NOTE_NAMES[rootPitchClass] ?? "c",
    rootPitchClass,
    mode,
    scaleIntervals: MODE_INTERVALS[mode],
    progressionDegrees: random.pick(PROGRESSIONS),
    chordSize: traits.complexity >= 0.62 ? 4 : 3,
  };
}

function createMotifDNA(seed: number): PocketCircuitMotifDNA {
  const random = new DeterministicRandom(seed);
  const contour: number[] = [...random.pick(MOTIF_CONTOURS)];
  const accentIndex = 4 + random.integer(3);
  const accent = contour[accentIndex];
  if (accent !== undefined) {
    contour[accentIndex] = accent + random.pick([-1, 1] as const);
  }
  contour[0] = 0;
  contour[contour.length - 1] = 0;
  return { degrees: contour, anchorLength: 8 };
}

function createRhythmDNA(
  seed: number,
  traits: NormalizedMusicTraits,
): PocketCircuitRhythmDNA {
  const random = new DeterministicRandom(seed);
  const noteCount = 3 + Math.round(traits.complexity * 4);
  const oddTarget = clamp(Math.round(noteCount * traits.syncopation), 0, 4);
  const evenTarget = clamp(noteCount - oddTarget, 0, 4);
  const remaining = noteCount - oddTarget - evenTarget;
  const oddCount = oddTarget + Math.min(remaining, 4 - oddTarget);
  const evenCount = noteCount - oddCount;
  const odd = random.shuffle([1, 3, 5, 7]).slice(0, oddCount);
  const even = random.shuffle([0, 2, 4, 6]).slice(0, evenCount);
  const melodyOnsets = [...odd, ...even].sort((left, right) => left - right);
  const kickOnsets = [0, random.pick([3, 4, 6] as const)];
  const snareChoices = random
    .shuffle([2, 6, 3, 7, 1, 5])
    .filter((step) => !kickOnsets.includes(step));
  return {
    pulse: "eighth-note",
    stepsPerBar: 8,
    pulseTicks: 480,
    melodyOnsets,
    kickOnsets,
    snareOnsets: [snareChoices[0] ?? 2, snareChoices[1] ?? 6].sort(
      (left, right) => left - right,
    ),
  };
}

const STYLE_KITS = {
  fusion: {
    melody: "epiano",
    brightMelody: "glass",
    harmony: ["warm"],
    drive: ["organ"],
    lift: ["glass", "epiano"],
    bass: "bass",
  },
  neon: {
    melody: "supersaw",
    brightMelody: "glass",
    harmony: ["pulse"],
    drive: ["pulse"],
    lift: ["glass"],
    bass: "bass",
  },
  funk: {
    melody: "pluck",
    brightMelody: "epiano",
    harmony: ["warm", "pluck"],
    drive: ["pluck"],
    lift: ["pluck", "epiano"],
    bass: "bass",
  },
  chip: {
    melody: "chip",
    brightMelody: "chip",
    harmony: ["chip"],
    drive: ["chip"],
    lift: ["chip"],
    bass: "triangle",
  },
} as const satisfies Record<
  PocketCircuitStyle,
  {
    melody: MelodyVoice;
    brightMelody: MelodyVoice;
    harmony: readonly HarmonyVoice[];
    drive: readonly HarmonyVoice[];
    lift: readonly MelodyVoice[];
    bass: BassVoice;
  }
>;

const DRIVE_PHASES: ReadonlySet<PocketCircuitSectionPlan["id"]> = new Set([
  "grid",
  "attack",
  "final-lap",
]);

function createTimbreDNA(
  seed: number,
  traits: NormalizedMusicTraits,
  style: PocketCircuitStyle,
): PocketCircuitTimbreDNA {
  const random = new DeterministicRandom(seed);
  const kit = STYLE_KITS[style];
  const useAlternateLead =
    style === "fusion"
      ? traits.brightness > 0.72
      : style === "neon"
        ? traits.brightness < 0.35
        : style === "funk"
          ? traits.energy < 0.35
          : false;
  return {
    harmonyVoice: random.pick(kit.harmony),
    driveHarmonyVoice: random.pick(kit.drive),
    melodyVoice: useAlternateLead ? kit.brightMelody : kit.melody,
    liftVoice: random.pick(kit.lift),
    bassVoice: kit.bass,
  };
}

export function arrangementGroove(
  style: PocketCircuitStyle,
  plan: PocketCircuitSectionPlan,
): PocketCircuitSectionPlan {
  if (style !== "funk") {
    return plan;
  }
  if (plan.id === "grid") {
    return (
      POCKET_CIRCUIT_SECTION_PLANS.find((candidate) => candidate.id === "cruise") ??
      plan
    );
  }
  if (plan.id === "cruise") {
    return (
      POCKET_CIRCUIT_SECTION_PLANS.find((candidate) => candidate.id === "grid") ??
      plan
    );
  }
  return plan;
}

function createArrangementDNA(
  seed: number,
  traits: NormalizedMusicTraits,
): PocketCircuitArrangementDNA {
  const random = new DeterministicRandom(seed);
  const bpmJitter = random.integer(5) - 2;
  return {
    bpm: clamp(112 + Math.round(traits.energy * 36) + bpmJitter, 112, 150),
    bassApproach: random.pick(
      ["root-fifth", "root-octave", "fifth-root"] as const,
    ),
    chordGate: 0.68 + traits.complexity * 0.22,
    melodyGate: 0.42 + (1 - traits.syncopation) * 0.3,
  };
}

function createOrnamentDNA(
  seed: number,
  traits: NormalizedMusicTraits,
): PocketCircuitOrnamentDNA {
  const random = new DeterministicRandom(seed);
  const ornamentStrength = traits.complexity >= 0.5 ? 1 : 0;
  return {
    barRotations: [
      0,
      1 + random.integer(3),
      3 + random.integer(3),
      1 + random.integer(5),
    ],
    passingOffsets: [
      0,
      random.pick([-1, 1] as const) * ornamentStrength,
      random.pick([-1, 1] as const) * ornamentStrength,
      0,
    ],
    turnaroundStep: random.integer(7),
  };
}

function createDNA(input: PocketCircuitGeneratorInput): PocketCircuitMusicalDNA {
  const traits = normalizeMusicTraits(input.traits);
  const style = input.style ?? "fusion";
  const levelSeed = canonicalizeSeed(input.seed);
  const domainSeeds = derivePocketCircuitDomainSeeds(input.seed);
  const arrangement = createArrangementDNA(domainSeeds.arrangement, traits);
  return {
    generatorVersion: POCKET_CIRCUIT_GENERATOR_VERSION,
    levelSeed,
    style,
    traits,
    domainSeeds,
    harmony: createHarmonyDNA(domainSeeds.harmony, traits),
    motif: createMotifDNA(domainSeeds.motif),
    rhythm: createRhythmDNA(domainSeeds.rhythm, traits),
    timbre: createTimbreDNA(domainSeeds.timbre, traits, style),
    arrangement:
      style === "fusion"
        ? { ...arrangement, bpm: clamp(arrangement.bpm + 10, 112, 150) }
        : arrangement,
    ornaments: createOrnamentDNA(domainSeeds.ornaments, traits),
  };
}

export function createPocketCircuitMusicalDNA(
  input: PocketCircuitGeneratorInput,
): PocketCircuitMusicalDNA {
  return createDNA(input);
}

function scalePitch(
  rootMidi: number,
  degree: number,
  scaleIntervals: readonly number[],
): number {
  const scaleLength = scaleIntervals.length;
  const scaleIndex = ((degree % scaleLength) + scaleLength) % scaleLength;
  const octave = Math.floor(degree / scaleLength);
  const interval = scaleIntervals[scaleIndex];
  if (interval === undefined) {
    throw new Error(`Missing scale degree: ${degree}`);
  }
  return rootMidi + octave * 12 + interval;
}

function midiToNote(midi: number): string {
  const pitchClass = ((midi % 12) + 12) % 12;
  const name = NOTE_NAMES[pitchClass];
  if (name === undefined || midi < 0 || midi > 127) {
    throw new Error(`Cannot encode MIDI note: ${midi}`);
  }
  return `${name}${Math.floor(midi / 12) - 1}`;
}

export type PocketCircuitHarmonyTreatment =
  | "shared"
  | "pressure"
  | "dominant"
  | "parallel-major";

export interface ResolvedSectionHarmony {
  scaleIntervals: readonly number[];
  progressionDegrees: readonly number[];
  chordSize: 3 | 4;
}

const MAJOR_INTERVALS = [0, 2, 4, 5, 7, 9, 11] as const;

const SECTION_HARMONY_TREATMENTS: Readonly<
  Record<string, PocketCircuitHarmonyTreatment>
> = {
  garage: "shared",
  grid: "shared",
  cruise: "shared",
  attack: "pressure",
  "final-lap": "dominant",
  victory: "parallel-major",
};

function raiseSeventh(intervals: readonly number[]): readonly number[] {
  const raised = [...intervals];
  const seventh = raised[6];
  if (seventh !== undefined) {
    raised[6] = Math.min(11, seventh + 1);
  }
  return raised;
}

function resolveSectionHarmony(
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
): ResolvedSectionHarmony {
  const treatment = SECTION_HARMONY_TREATMENTS[plan.id] ?? "shared";
  const base = dna.harmony;
  switch (treatment) {
    case "shared":
      return {
        scaleIntervals: base.scaleIntervals,
        progressionDegrees: base.progressionDegrees,
        chordSize: base.chordSize,
      };
    case "pressure":
      return {
        scaleIntervals: raiseSeventh(base.scaleIntervals),
        progressionDegrees: base.progressionDegrees,
        chordSize: base.chordSize,
      };
    case "dominant": {
      const progression = [...base.progressionDegrees];
      progression[3] = 4;
      return {
        scaleIntervals: raiseSeventh(base.scaleIntervals),
        progressionDegrees: progression,
        chordSize: base.chordSize,
      };
    }
    case "parallel-major": {
      const progression = [...base.progressionDegrees];
      progression[3] = 0;
      return {
        scaleIntervals: MAJOR_INTERVALS,
        progressionDegrees: progression,
        chordSize: base.chordSize,
      };
    }
  }
}

export function resolvePocketCircuitSectionHarmony(
  dna: PocketCircuitMusicalDNA,
  sectionId: string,
): ResolvedSectionHarmony {
  const plan = POCKET_CIRCUIT_SECTION_PLANS.find(
    (candidate) => candidate.id === sectionId,
  );
  if (plan === undefined) {
    throw new Error(`Unknown Pocket Circuit section: ${sectionId}`);
  }
  return resolveSectionHarmony(dna, plan);
}

type BarMask = readonly number[] | "whole";

const CHORD_ONSET_MASKS: Readonly<Record<string, readonly BarMask[]>> = {
  garage: ["whole", "whole", "whole", "whole"],
  grid: [[0, 4], [0, 3, 5], [0, 2, 4, 6], [0, 1, 3, 5, 7]],
  cruise: ["whole", "whole", "whole", "whole"],
  attack: ["whole", "whole", "whole", "whole"],
  "final-lap": ["whole", "whole", "whole", "whole"],
  victory: ["whole", "whole", "whole", "whole"],
};

const BASS_ONSET_MASKS: Readonly<Record<string, readonly BarMask[]>> = {
  garage: ["whole", "whole", "whole", "whole"],
  grid: [[0, 4], [0, 3, 6], [0, 2, 4, 6], [0, 1, 3, 5, 7]],
  cruise: [[0, 4], [0, 4], [0, 4], [0, 4]],
  attack: [[0, 4], [0, 4], [0, 4], [0, 4]],
  "final-lap": [[0, 4], [0, 4], [0, 4], [0, 4]],
  victory: ["whole", "whole", "whole", "whole"],
};

function uniqueSteps(steps: readonly number[]): number[] {
  return [...new Set(steps.map((step) => ((step % 8) + 8) % 8))].sort(
    (left, right) => left - right,
  );
}

function fillSteps(
  initial: readonly number[],
  target: number,
  candidates: readonly number[],
): number[] {
  const result = uniqueSteps(initial);
  for (const candidate of candidates) {
    if (result.length >= target) {
      break;
    }
    if (!result.includes(candidate)) {
      result.push(candidate);
    }
  }
  return uniqueSteps(result).slice(0, target);
}

function distributedSteps(steps: readonly number[], count: number): number[] {
  if (count >= steps.length) {
    return [...steps];
  }
  return Array.from({ length: count }, (_, index) => {
    const sourceIndex = Math.floor((index * steps.length) / count);
    const step = steps[sourceIndex];
    if (step === undefined) {
      throw new Error(`Missing distributed onset: ${sourceIndex}`);
    }
    return step;
  });
}

function barMask(
  masks: Readonly<Record<string, readonly BarMask[]>>,
  plan: PocketCircuitSectionPlan,
  barIndex: number,
): BarMask {
  const mask = masks[plan.id]?.[barIndex];
  if (mask === undefined) {
    throw new Error(`Missing ${plan.id} bar mask: ${barIndex}`);
  }
  return mask;
}

function transformStyleMask(
  mask: BarMask,
  style: PocketCircuitStyle,
  plan: PocketCircuitSectionPlan,
  _lane: "harmony" | "bass",
  preserveDownbeat: boolean,
): BarMask {
  if (
    mask === "whole" ||
    style === "fusion" ||
    plan.id === "cruise" ||
    plan.id === "attack" ||
    plan.id === "final-lap"
  ) {
    return mask;
  }
  const downbeat = preserveDownbeat && mask.includes(0) ? [0] : [];
  const movable = mask.filter((step) => !downbeat.includes(step));
  if (style === "neon") {
    const shift = plan.id === "grid" ? 1 : 2;
    return uniqueSteps([...downbeat, ...movable.map((step) => step + shift)]);
  }
  if (style === "funk") {
    return uniqueSteps([
      ...downbeat,
      ...movable.map((step) => (step % 2 === 0 ? step + 1 : step)),
    ]);
  }
  const echoDistance = plan.id === "attack" ? 2 : 1;
  return uniqueSteps([
    ...mask,
    ...movable.map((step) => step + echoDistance),
  ]);
}

function formatBarPattern(value: string, mask: BarMask): string {
  if (mask === "whole") {
    return value;
  }
  const onsets = new Set(mask);
  const tokens = Array.from({ length: 8 }, (_, step) =>
    onsets.has(step) ? value : "~",
  );
  return `[${tokens.join(" ")}]`;
}

function createChordPattern(
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
  resolved: ResolvedSectionHarmony,
): string {
  const rootMidi = 48 + dna.harmony.rootPitchClass;
  const useSeventh =
    resolved.chordSize === 4 ||
    plan.development >= 3 ||
    (dna.style === "fusion" && plan.development >= 1);
  const chordOffsets = useSeventh ? [0, 2, 4, 6] : [0, 2, 4];
  const bars = resolved.progressionDegrees.map((progressionDegree, barIndex) => {
    const notes = chordOffsets.map((offset) =>
      midiToNote(
        scalePitch(
          rootMidi,
          progressionDegree + offset,
          resolved.scaleIntervals,
        ),
      ),
    );
    const mask = transformStyleMask(
      barMask(CHORD_ONSET_MASKS, plan, barIndex),
      dna.style,
      plan,
      "harmony",
      true,
    );
    return formatBarPattern(`[${notes.join(",")}]`, mask);
  });
  return `<${bars.join(" ")}>`;
}

function createBassPattern(
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
  resolved: ResolvedSectionHarmony,
): string {
  const rootMidi = 36 + dna.harmony.rootPitchClass;
  const bars = resolved.progressionDegrees.map((degree, barIndex) => {
    const root = scalePitch(rootMidi, degree, resolved.scaleIntervals);
    const fifth = scalePitch(rootMidi, degree + 4, resolved.scaleIntervals);
    const alternate =
      dna.arrangement.bassApproach === "root-octave"
        ? root + 12
        : dna.arrangement.bassApproach === "fifth-root"
          ? scalePitch(rootMidi, degree - 3, resolved.scaleIntervals)
          : fifth;
    const mask = transformStyleMask(
      barMask(BASS_ONSET_MASKS, plan, barIndex),
      dna.style,
      plan,
      "bass",
      true,
    );
    if (mask === "whole") {
      return midiToNote(root);
    }
    const tokens = Array.from({ length: 8 }, () => "~");
    const sequence = [root, alternate, root + 12, fifth];
    mask.forEach((step, noteIndex) => {
      tokens[step] = midiToNote(sequence[noteIndex % sequence.length] ?? root);
    });
    return `[${tokens.join(" ")}]`;
  });
  return `<${bars.join(" ")}>`;
}

function breakConsecutiveRuns(steps: readonly number[]): number[] {
  const result = [...steps];
  while (true) {
    const runStart = result.findIndex(
      (step, index) =>
        result[index + 1] === step + 1 && result[index + 2] === step + 2,
    );
    if (runStart === -1) {
      return result;
    }
    result.splice(runStart + 1, 1);
  }
}

function phaseMelodyOnsets(
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
  barIndex: number,
): number[] {
  const base = dna.rhythm.melodyOnsets;
  let onsets: number[];
  if (plan.id === "garage") {
    onsets = distributedSteps(base, Math.min(2, base.length));
  } else if (plan.id === "grid") {
    onsets = fillSteps(
      [],
      2 + Math.round(dna.traits.complexity * 3) + barIndex,
      [0, 4, 6, 2, 5, 7, 1, 3],
    );
  } else if (plan.id === "cruise") {
    onsets = [2, 5];
  } else if (plan.id === "attack") {
    onsets = [2, 6];
  } else if (plan.id === "final-lap") {
    onsets = barIndex === 3 ? [2, 5, 7] : [2, 5];
  } else {
    onsets = distributedSteps(base, Math.min(2, base.length));
  }

  if (
    plan.id === "cruise" ||
    plan.id === "attack" ||
    plan.id === "final-lap"
  ) {
    return uniqueSteps(onsets);
  }

  if (dna.style === "neon") {
    const shift = plan.id === "grid" ? 1 : 2;
    const shifted = uniqueSteps(onsets.map((step) => step + shift));
    return plan.id === "grid" ? shifted : breakConsecutiveRuns(shifted);
  }
  if (dna.style === "funk") {
    if (plan.id === "final-lap") {
      return fillSteps(onsets, Math.min(7, onsets.length), [1, 3, 5, 7, 0, 2, 4, 6]);
    }
    const target = Math.min(4, onsets.length);
    return breakConsecutiveRuns(
      fillSteps(
        onsets.map((step) => (step % 2 === 0 ? step + 1 : step)),
        target,
        [1, 3, 5, 7],
      ),
    );
  }
  if (dna.style === "chip" && plan.id !== "victory" && plan.id !== "garage") {
    if (plan.id === "grid") {
      return uniqueSteps([...onsets, ...onsets.map((step) => step + 1)]);
    }
    const maximum = plan.id === "garage" ? 3 : plan.id === "cruise" ? 7 : 8;
    const target =
      plan.id === "final-lap"
        ? 8
        : Math.min(maximum, onsets.length + 1);
    const echoDistance = plan.id === "attack" ? 2 : 1;
    return fillSteps(
      [...onsets, ...onsets.map((step) => step + echoDistance)],
      target,
      plan.id === "attack" ? [1, 3, 5, 7] : [0, 2, 4, 6, 1, 3, 5, 7],
    );
  }
  return uniqueSteps(onsets);
}

export function melodyDegreesForBar(
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
  barIndex: number,
): number[] {
  const rotation = dna.ornaments.barRotations[barIndex];
  const passingOffset = dna.ornaments.passingOffsets[barIndex];
  if (rotation === undefined || passingOffset === undefined) {
    throw new Error(`Missing bar development: ${barIndex}`);
  }

  const degrees: number[] = [];
  const onsets = phaseMelodyOnsets(dna, plan, barIndex);
  onsets.forEach((step, noteIndex) => {
    const motifIndex =
      (noteIndex + rotation + (barIndex === 0 ? 0 : plan.development)) %
      dna.motif.degrees.length;
    const motifDegree = dna.motif.degrees[motifIndex];
    if (motifDegree === undefined) {
      throw new Error(`Missing motif degree: ${motifIndex}`);
    }
    const isTurnaround =
      barIndex === 3 && step === dna.ornaments.turnaroundStep;
    const developedDegree = isTurnaround
      ? 0
      : motifDegree + (barIndex === 0 ? 0 : plan.lift + passingOffset);
    const previousDegree = degrees.at(-1);
    degrees.push(
      previousDegree === developedDegree ? developedDegree + 2 : developedDegree,
    );
  });
  return degrees;
}

function createMelodyPattern(
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
  resolved: ResolvedSectionHarmony,
): string {
  const brightnessOctave = dna.traits.brightness >= 0.67 ? 12 : 0;
  const rootMidi = 60 + dna.harmony.rootPitchClass + brightnessOctave;
  const registerShift =
    plan.registerShift + (dna.style === "fusion" && plan.id === "victory" ? 7 : 0);
  const bars = Array.from({ length: 4 }, (_, barIndex) => {
    const tokens = Array.from({ length: dna.rhythm.stepsPerBar }, () => "~");
    const onsets = phaseMelodyOnsets(dna, plan, barIndex);
    const degrees = melodyDegreesForBar(dna, plan, barIndex);
    onsets.forEach((step, noteIndex) => {
      const developedDegree = degrees[noteIndex];
      if (developedDegree === undefined) {
        throw new Error(`Missing developed melody degree: ${noteIndex}`);
      }
      tokens[step] = midiToNote(
        scalePitch(
          rootMidi,
          developedDegree + registerShift,
          resolved.scaleIntervals,
        ),
      );
    });
    return `[${tokens.join(" ")}]`;
  });
  return `<${bars.join(" ")}>`;
}

function createVictorySparklePattern(
  dna: PocketCircuitMusicalDNA,
  resolved: ResolvedSectionHarmony,
): string {
  const rootMidi = 72 + dna.harmony.rootPitchClass;
  const offsets = [0, 2, 4, 7] as const;
  const onsets = [0, 2, 4, 6] as const;
  const bars = Array.from({ length: 4 }, () => {
    const tokens = Array.from({ length: dna.rhythm.stepsPerBar }, () => "~");
    onsets.forEach((step, noteIndex) => {
      tokens[step] = midiToNote(
        scalePitch(
          rootMidi,
          offsets[noteIndex] ?? 0,
          resolved.scaleIntervals,
        ),
      );
    });
    return `[${tokens.join(" ")}]`;
  });
  return `<${bars.join(" ")}>`;
}

const LIFT_DEGREE: Readonly<Record<PocketCircuitStyle, number>> = {
  fusion: 4,
  neon: 7,
  funk: 2,
  chip: 0,
};

function createLiftPattern(
  dna: PocketCircuitMusicalDNA,
  resolved: ResolvedSectionHarmony,
): string {
  const rootMidi = 72 + dna.harmony.rootPitchClass;
  const extra = LIFT_DEGREE[dna.style];
  const bars = resolved.progressionDegrees.map((degree) =>
    midiToNote(
      scalePitch(rootMidi, degree + extra, resolved.scaleIntervals),
    ),
  );
  return `<${bars.join(" ")}>`;
}

function percussionOnsets(
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
  barIndex: number,
): { kick: number[]; snare: number[]; hat: number[] } {
  let kick: number[];
  let snare: number[];
  let hat: number[];
  if (plan.id === "garage") {
    kick = [dna.rhythm.kickOnsets[0] ?? 0];
    snare = [];
    hat = [4];
  } else if (plan.id === "grid") {
    const extraKick = dna.rhythm.kickOnsets[1];
    kick =
      barIndex >= 2
        ? [0, extraKick === undefined || extraKick === 6 ? 4 : extraKick]
        : [0];
    snare = [6, ...(barIndex >= 1 ? [2] : [])];
    hat = [2, 4, ...(barIndex >= 2 ? [1] : []), ...(barIndex === 3 ? [7] : [])];
  } else if (plan.id === "cruise") {
    kick = [0, 4];
    snare = [2, 6];
    hat = [1, 3, 5, 7];
  } else if (plan.id === "attack") {
    kick = [0, 4, 5];
    snare = [2, 6];
    hat = [1, 3, 7];
  } else if (plan.id === "final-lap") {
    kick = [0, 4, 5];
    snare = [2, 6, 7];
    hat = [1, 3];
  } else {
    kick = [0];
    snare = [4];
    hat = barIndex === 3 ? [] : [6];
  }

  if (plan.id === "garage" || plan.id === "grid" || plan.id === "victory") {
    if (dna.style === "neon") {
      hat = hat.map((step) => step + 2);
    } else if (dna.style === "funk") {
      kick = kick.map((step, index) =>
        index === 0 ? step : step % 2 === 0 ? step + 1 : step,
      );
      hat = [...hat.filter((step) => step % 2 === 1), ...(plan.id === "garage" ? [] : [7])];
    } else if (dna.style === "chip") {
      const echoes = kick.map((step) => step + 1);
      hat = [...hat, ...echoes];
    }
  }
  return {
    kick: uniqueSteps(kick),
    snare: uniqueSteps(snare),
    hat: uniqueSteps(hat),
  };
}

function createPercussionPattern(
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
): string {
  const bars = Array.from({ length: 4 }, (_, barIndex) => {
    const onsets = percussionOnsets(dna, plan, barIndex);
    const kick = new Set(onsets.kick);
    const snare = new Set(onsets.snare);
    const hat = new Set(onsets.hat);
    const tokens: string[] = Array.from(
      { length: dna.rhythm.stepsPerBar },
      (_, step) => {
      if (kick.has(step)) {
        return "kick";
      }
      if (snare.has(step)) {
        return "snare";
      }
      return hat.has(step) ? "hat" : "~";
      },
    );
    if (barIndex === 3 && plan.finalAccent === "tom") {
      tokens[7] = "tom";
    }
    return `[${tokens.join(" ")}]`;
  });
  return `<${bars.join(" ")}>`;
}

function velocity(
  base: number,
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
): number {
  return clamp(base + dna.traits.energy * 0.22 + plan.intensity * 0.24, 0.1, 0.96);
}

const STYLE_GATE_SCALES: Readonly<Record<PocketCircuitStyle, number>> = {
  fusion: 1,
  neon: 0.88,
  funk: 0.72,
  chip: 0.58,
};

const LIFT_GATES: Readonly<Record<PocketCircuitStyle, number>> = {
  fusion: 0.92,
  neon: 0.88,
  funk: 0.8,
  chip: 0.7,
};

function phaseGate(
  base: number,
  phaseScale: number,
  style: PocketCircuitStyle,
): number {
  return clamp(base * phaseScale * STYLE_GATE_SCALES[style], 0.16, 0.98);
}

function createSection(
  dna: PocketCircuitMusicalDNA,
  plan: PocketCircuitSectionPlan,
): AuthoringSection {
  const groove = arrangementGroove(dna.style, plan);
  const resolved = resolveSectionHarmony(dna, groove);
  const bassGate = 0.58 + dna.traits.energy * 0.24;
  const harmonyVoice = DRIVE_PHASES.has(groove.id)
    ? dna.timbre.driveHarmonyVoice
    : dna.timbre.harmonyVoice;
  const melodyVoice = dna.timbre.melodyVoice;
  const lanes = [
    {
      kind: "note" as const,
      id: `${plan.id}-harmony`,
      pattern: createChordPattern(dna, groove, resolved),
      voice: harmonyVoice,
      velocity: velocity(dna.style === "chip" ? 0.1 : 0.18, dna, groove),
      gate: phaseGate(
        dna.arrangement.chordGate,
        groove.chordGateScale,
        dna.style,
      ),
    },
    {
      kind: "note" as const,
      id: `${plan.id}-melody`,
      pattern: createMelodyPattern(dna, groove, resolved),
      voice: melodyVoice,
      role: "melody" as const,
      velocity: velocity(0.24, dna, groove),
      gate: phaseGate(
        dna.arrangement.melodyGate,
        groove.melodyGateScale,
        dna.style,
      ),
    },
    {
      kind: "note" as const,
      id: `${plan.id}-bass`,
      pattern: createBassPattern(dna, groove, resolved),
      voice: dna.timbre.bassVoice,
      velocity: velocity(0.3, dna, groove),
      gate: phaseGate(bassGate, groove.bassGateScale, dna.style),
    },
    {
      kind: "percussion" as const,
      id: `${plan.id}-kit`,
      pattern: createPercussionPattern(dna, groove),
      velocity: velocity(0.2, dna, groove),
    },
    ...(plan.id === "final-lap"
      ? [
          {
            kind: "note" as const,
            id: `${plan.id}-lift`,
            pattern: createLiftPattern(dna, resolved),
            voice: dna.timbre.liftVoice,
            velocity: velocity(0.28, dna, plan),
            gate: LIFT_GATES[dna.style],
          },
        ]
      : []),
    ...(dna.style === "fusion" && plan.id === "victory"
      ? [
          {
            kind: "note" as const,
            id: `${plan.id}-sparkle`,
            pattern: createVictorySparklePattern(dna, resolved),
            voice: "glass" as const,
            velocity: velocity(0.2, dna, plan),
            gate: 0.5,
          },
        ]
      : []),
  ];

  return {
    id: plan.id,
    label: plan.label,
    feeling: plan.feeling,
    color: plan.color,
    bars: 4,
    lanes,
  };
}

function scoreId(dna: PocketCircuitMusicalDNA): string {
  const version = POCKET_CIRCUIT_GENERATOR_VERSION.replaceAll(".", "-");
  const identity = hashText(
    `${dna.levelSeed}:${dna.style}:${JSON.stringify(dna.traits)}`,
  ).toString(16).padStart(8, "0");
  return `pocket-circuit-generated-v${version}-${identity}`;
}

function createAuthoringScore(dna: PocketCircuitMusicalDNA): AuthoringScore {
  const styleNames = {
    fusion: "Fusion",
    neon: "Neon",
    funk: "Pocket Funk",
    chip: "Micro Motor",
  } satisfies Record<PocketCircuitStyle, string>;
  return {
    id: scoreId(dna),
    title: `${styleNames[dna.style]} ${dna.harmony.key.toUpperCase()} Run`,
    bpm: dna.arrangement.bpm,
    beatsPerBar: 4,
    ticksPerBeat: 960,
    crossfadeBars: 2,
    defaultSection: "garage",
    sections: POCKET_CIRCUIT_SECTION_PLANS.map((plan) =>
      createSection(dna, plan),
    ),
    rules: pocketCircuitRules,
  };
}

export function generatePocketCircuitAuthoringScore(
  input: PocketCircuitGeneratorInput,
): AuthoringScore {
  return createAuthoringScore(createDNA(input));
}

export function generatePocketCircuitLevel(
  input: PocketCircuitGeneratorInput,
): GeneratedPocketCircuitLevel {
  const dna = createDNA(input);
  const authoringScore = createAuthoringScore(dna);
  const portableScore = exportScore(
    authoringScore,
    `${POCKET_CIRCUIT_GENERATOR_VERSION}:${dna.levelSeed}`,
  );
  return {
    generatorVersion: POCKET_CIRCUIT_GENERATOR_VERSION,
    levelSeed: dna.levelSeed,
    style: dna.style,
    traits: dna.traits,
    domainSeeds: dna.domainSeeds,
    dna,
    authoringScore,
    portableScore,
  };
}
