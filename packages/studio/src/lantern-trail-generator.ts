import type { NoteEvent, PortableScore } from "@gamestruments/runtime";
import {
  exportScore,
  type AuthoringScore,
  type AuthoringSection,
} from "./export-score.js";
import { lanternTrailRules } from "./lantern-trail-score.js";

export const LANTERN_TRAIL_GENERATOR_VERSION = "1.1.0" as const;

export const LANTERN_TRAIL_GENERATOR_DOMAINS = [
  "harmony",
  "motif",
  "rhythm",
  "timbre",
  "arrangement",
  "ornaments",
] as const;

export type LanternTrailGeneratorDomain =
  (typeof LANTERN_TRAIL_GENERATOR_DOMAINS)[number];

export type LanternTrailSeed = string | number;

export interface NormalizedTrailTraits {
  wonder: number;
  danger: number;
  mystery: number;
  motion: number;
}

export const DEFAULT_LANTERN_TRAIL_TRAITS = {
  wonder: 0.55,
  danger: 0.5,
  mystery: 0.6,
  motion: 0.58,
} as const satisfies NormalizedTrailTraits;

export interface LanternTrailGeneratorInput {
  seed: LanternTrailSeed;
  traits?: Partial<NormalizedTrailTraits>;
}

export type LanternTrailDomainSeeds = Record<
  LanternTrailGeneratorDomain,
  number
>;

export type LanternTrailMode =
  | "aeolian"
  | "dorian"
  | "mixolydian"
  | "lydian"
  | "phrygian";

export interface LanternTrailHarmonyDNA {
  key: string;
  rootPitchClass: number;
  mode: LanternTrailMode;
  scaleIntervals: readonly number[];
  progressionDegrees: readonly number[];
  chordSize: 3 | 4;
}

export interface LanternTrailMotifDNA {
  degrees: readonly number[];
  anchorLength: number;
}

export interface LanternTrailRhythmDNA {
  pulse: "eighth-note";
  stepsPerBar: 8;
  pulseTicks: 480;
  melodyOnsets: readonly number[];
  kickOnsets: readonly number[];
  snareOnsets: readonly number[];
}

type HarmonyVoice = Extract<NoteEvent["voice"], "warm" | "pulse">;
type MelodyVoice = Extract<NoteEvent["voice"], "glass" | "pluck" | "epiano">;

export interface LanternTrailTimbreDNA {
  harmonyVoice: HarmonyVoice;
  melodyVoice: MelodyVoice;
  bassVoice: "bass";
}

export interface LanternTrailArrangementDNA {
  bpm: number;
  bassApproach: "root-fifth" | "root-octave" | "fifth-root";
  chordGate: number;
  melodyGate: number;
}

export interface LanternTrailOrnamentDNA {
  barRotations: readonly [number, number, number, number];
  passingOffsets: readonly [number, number, number, number];
  discoveryStep: number;
}

export interface LanternTrailMusicalDNA {
  generatorVersion: typeof LANTERN_TRAIL_GENERATOR_VERSION;
  levelSeed: string;
  traits: NormalizedTrailTraits;
  domainSeeds: LanternTrailDomainSeeds;
  harmony: LanternTrailHarmonyDNA;
  motif: LanternTrailMotifDNA;
  rhythm: LanternTrailRhythmDNA;
  timbre: LanternTrailTimbreDNA;
  arrangement: LanternTrailArrangementDNA;
  ornaments: LanternTrailOrnamentDNA;
}

export interface GeneratedLanternTrailAdventure {
  generatorVersion: typeof LANTERN_TRAIL_GENERATOR_VERSION;
  levelSeed: string;
  traits: NormalizedTrailTraits;
  domainSeeds: LanternTrailDomainSeeds;
  dna: LanternTrailMusicalDNA;
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

const MODE_INTERVALS: Readonly<Record<LanternTrailMode, readonly number[]>> = {
  aeolian: [0, 2, 3, 5, 7, 8, 10],
  dorian: [0, 2, 3, 5, 7, 9, 10],
  mixolydian: [0, 2, 4, 5, 7, 9, 10],
  lydian: [0, 2, 4, 6, 7, 9, 11],
  phrygian: [0, 1, 3, 5, 7, 8, 10],
};

const PROGRESSIONS = [
  [0, 3, 5, 4],
  [0, 5, 3, 4],
  [0, 2, 3, 4],
  [0, 4, 3, 5],
] as const;

const MOTIF_CONTOURS = [
  [0, 2, 4, 2, 5, 3, 1, 0],
  [0, 3, 1, 4, 2, 5, 3, 0],
  [0, 1, 3, 5, 3, 2, 4, 0],
  [0, 4, 2, 3, 5, 3, 2, 0],
] as const;

interface SectionPlan {
  id: string;
  label: string;
  feeling: string;
  color: string;
  intensity: number;
  development: number;
  lift: number;
  drumDensity: number;
  finalAccent: "hat" | "tom";
}

const SECTION_PLANS: readonly SectionPlan[] = [
  {
    id: "camp",
    label: "Trailhead Camp",
    feeling: "warmth / still anticipation",
    color: "#e8c67a",
    intensity: 0.5,
    development: 0,
    lift: 0,
    drumDensity: 0.28,
    finalAccent: "hat",
  },
  {
    id: "explore",
    label: "The Old Forest",
    feeling: "curiosity / forward motion",
    color: "#8fbf8f",
    intensity: 0.66,
    development: 1,
    lift: 0,
    drumDensity: 0.45,
    finalAccent: "hat",
  },
  {
    id: "clue",
    label: "A Lantern Found",
    feeling: "mystery / dawning discovery",
    color: "#7fd4d8",
    intensity: 0.72,
    development: 2,
    lift: 0,
    drumDensity: 0.55,
    finalAccent: "hat",
  },
  {
    id: "danger",
    label: "The Darkening Path",
    feeling: "menace / resolve",
    color: "#c94f4f",
    intensity: 0.9,
    development: 3,
    lift: 1,
    drumDensity: 0.8,
    finalAccent: "hat",
  },
  {
    id: "sanctuary",
    label: "The Hidden Glade",
    feeling: "awe / radiant arrival",
    color: "#a6d7a0",
    intensity: 0.92,
    development: 4,
    lift: 6,
    drumDensity: 0.85,
    finalAccent: "tom",
  },
  {
    id: "quest-complete",
    label: "Lantern Lit",
    feeling: "release / earned wonder",
    color: "#f2e39a",
    intensity: 0.72,
    development: 5,
    lift: 2,
    drumDensity: 0.55,
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

function canonicalizeSeed(seed: LanternTrailSeed): string {
  if (typeof seed === "number") {
    if (!Number.isSafeInteger(seed)) {
      throw new Error(`Numeric trail seed must be a safe integer: ${seed}`);
    }
    return `number:${seed}`;
  }
  return `string:${seed}`;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.max(minimum, Math.min(maximum, value));
}

function clampUnit(value: number, name: keyof NormalizedTrailTraits): number {
  if (!Number.isFinite(value)) {
    throw new Error(`Trail trait ${name} must be finite: ${value}`);
  }
  return clamp(value, 0, 1);
}

export function normalizeTrailTraits(
  traits: Partial<NormalizedTrailTraits> = {},
): NormalizedTrailTraits {
  return {
    wonder: clampUnit(traits.wonder ?? DEFAULT_LANTERN_TRAIL_TRAITS.wonder, "wonder"),
    danger: clampUnit(traits.danger ?? DEFAULT_LANTERN_TRAIL_TRAITS.danger, "danger"),
    mystery: clampUnit(
      traits.mystery ?? DEFAULT_LANTERN_TRAIL_TRAITS.mystery,
      "mystery",
    ),
    motion: clampUnit(traits.motion ?? DEFAULT_LANTERN_TRAIL_TRAITS.motion, "motion"),
  };
}

export function deriveLanternTrailSubSeed(
  trailSeed: LanternTrailSeed,
  domain: LanternTrailGeneratorDomain,
): number {
  return hashText(
    `${LANTERN_TRAIL_GENERATOR_VERSION}\u0000${canonicalizeSeed(trailSeed)}\u0000${domain}`,
  );
}

export function deriveLanternTrailDomainSeeds(
  trailSeed: LanternTrailSeed,
): LanternTrailDomainSeeds {
  return {
    harmony: deriveLanternTrailSubSeed(trailSeed, "harmony"),
    motif: deriveLanternTrailSubSeed(trailSeed, "motif"),
    rhythm: deriveLanternTrailSubSeed(trailSeed, "rhythm"),
    timbre: deriveLanternTrailSubSeed(trailSeed, "timbre"),
    arrangement: deriveLanternTrailSubSeed(trailSeed, "arrangement"),
    ornaments: deriveLanternTrailSubSeed(trailSeed, "ornaments"),
  };
}

function selectMode(
  random: DeterministicRandom,
  traits: NormalizedTrailTraits,
): LanternTrailMode {
  if (traits.mystery >= 0.66) {
    return random.pick(
      traits.danger >= 0.5
        ? (["phrygian", "aeolian"] as const)
        : (["phrygian", "dorian"] as const),
    );
  }
  if (traits.wonder >= 0.66) {
    return random.pick(["lydian", "mixolydian"] as const);
  }
  if (traits.danger >= 0.6) {
    return random.pick(["aeolian", "dorian"] as const);
  }
  return random.pick(["dorian", "mixolydian"] as const);
}

function createHarmonyDNA(
  seed: number,
  traits: NormalizedTrailTraits,
): LanternTrailHarmonyDNA {
  const random = new DeterministicRandom(seed);
  const rootPitchClass = random.pick(KEY_PITCH_CLASSES);
  const mode = selectMode(random, traits);
  return {
    key: NOTE_NAMES[rootPitchClass] ?? "c",
    rootPitchClass,
    mode,
    scaleIntervals: MODE_INTERVALS[mode],
    progressionDegrees: random.pick(PROGRESSIONS),
    chordSize: traits.mystery >= 0.6 ? 4 : 3,
  };
}

function createMotifDNA(seed: number): LanternTrailMotifDNA {
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
  traits: NormalizedTrailTraits,
): LanternTrailRhythmDNA {
  const random = new DeterministicRandom(seed);
  const noteCount = 3 + Math.round(traits.motion * 4);
  const oddTarget = clamp(Math.round(noteCount * traits.motion), 0, 4);
  const evenTarget = clamp(noteCount - oddTarget, 0, 4);
  const remaining = noteCount - oddTarget - evenTarget;
  const oddCount = oddTarget + Math.min(remaining, 4 - oddTarget);
  const evenCount = noteCount - oddCount;
  const odd = random.shuffle([1, 3, 5, 7]).slice(0, oddCount);
  const even = random.shuffle([0, 2, 4, 6]).slice(0, evenCount);
  const melodyOnsets = [...odd, ...even].sort((left, right) => left - right);
  const kickChoices = random.shuffle([0, 3, 4, 6]);
  const kickOnsets = [0, kickChoices[1] ?? 4].sort(
    (left, right) => left - right,
  );
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

function createTimbreDNA(
  seed: number,
  traits: NormalizedTrailTraits,
): LanternTrailTimbreDNA {
  const random = new DeterministicRandom(seed);
  let melodyVoice: MelodyVoice;
  if (traits.wonder >= 0.66) {
    melodyVoice = "glass";
  } else if (traits.mystery >= 0.66) {
    melodyVoice = "pluck";
  } else {
    melodyVoice = random.pick(["epiano", "glass"] as const);
  }
  const harmonyVoice: HarmonyVoice =
    traits.danger >= 0.6 ? "pulse" : random.pick(["warm", "pulse"] as const);
  return { harmonyVoice, melodyVoice, bassVoice: "bass" };
}

function createArrangementDNA(
  seed: number,
  traits: NormalizedTrailTraits,
): LanternTrailArrangementDNA {
  const random = new DeterministicRandom(seed);
  const bpmJitter = random.integer(5) - 2;
  return {
    bpm: 84 + Math.round(traits.danger * 30) + Math.round(traits.motion * 14) + bpmJitter,
    bassApproach: random.pick(
      ["root-fifth", "root-octave", "fifth-root"] as const,
    ),
    chordGate: 0.68 + traits.mystery * 0.2,
    melodyGate: 0.5 + (1 - traits.mystery) * 0.3,
  };
}

function createOrnamentDNA(
  seed: number,
  traits: NormalizedTrailTraits,
): LanternTrailOrnamentDNA {
  const random = new DeterministicRandom(seed);
  const ornamentStrength = traits.mystery >= 0.5 ? 1 : 0;
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
    discoveryStep: random.integer(7),
  };
}

function createDNA(input: LanternTrailGeneratorInput): LanternTrailMusicalDNA {
  const traits = normalizeTrailTraits(input.traits);
  const trailSeed = canonicalizeSeed(input.seed);
  const domainSeeds = deriveLanternTrailDomainSeeds(input.seed);
  return {
    generatorVersion: LANTERN_TRAIL_GENERATOR_VERSION,
    levelSeed: trailSeed,
    traits,
    domainSeeds,
    harmony: createHarmonyDNA(domainSeeds.harmony, traits),
    motif: createMotifDNA(domainSeeds.motif),
    rhythm: createRhythmDNA(domainSeeds.rhythm, traits),
    timbre: createTimbreDNA(domainSeeds.timbre, traits),
    arrangement: createArrangementDNA(domainSeeds.arrangement, traits),
    ornaments: createOrnamentDNA(domainSeeds.ornaments, traits),
  };
}

export function createLanternTrailMusicalDNA(
  input: LanternTrailGeneratorInput,
): LanternTrailMusicalDNA {
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

export type LanternTrailHarmonyTreatment =
  | "shared"
  | "pressure"
  | "lydian-radiance"
  | "parallel-major";

export interface ResolvedTrailHarmony {
  scaleIntervals: readonly number[];
  progressionDegrees: readonly number[];
  chordSize: 3 | 4;
}

const LYDIAN_INTERVALS = [0, 2, 4, 6, 7, 9, 11] as const;
const MAJOR_INTERVALS = [0, 2, 4, 5, 7, 9, 11] as const;

const SECTION_HARMONY_TREATMENTS: Readonly<
  Record<string, LanternTrailHarmonyTreatment>
> = {
  camp: "shared",
  explore: "shared",
  clue: "shared",
  danger: "pressure",
  sanctuary: "lydian-radiance",
  "quest-complete": "parallel-major",
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
  dna: LanternTrailMusicalDNA,
  plan: SectionPlan,
): ResolvedTrailHarmony {
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
    case "lydian-radiance":
      return {
        scaleIntervals: LYDIAN_INTERVALS,
        progressionDegrees: base.progressionDegrees,
        chordSize: base.chordSize,
      };
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

export function resolveLanternTrailSectionHarmony(
  dna: LanternTrailMusicalDNA,
  sectionId: string,
): ResolvedTrailHarmony {
  const plan = SECTION_PLANS.find((candidate) => candidate.id === sectionId);
  if (plan === undefined) {
    throw new Error(`Unknown Lantern Trail section: ${sectionId}`);
  }
  return resolveSectionHarmony(dna, plan);
}

function createChordPattern(
  dna: LanternTrailMusicalDNA,
  plan: SectionPlan,
  resolved: ResolvedTrailHarmony,
): string {
  const rootMidi = 48 + dna.harmony.rootPitchClass;
  const useSeventh = resolved.chordSize === 4 || plan.development >= 3;
  const chordOffsets = useSeventh ? [0, 2, 4, 6] : [0, 2, 4];
  const chords = resolved.progressionDegrees.map((progressionDegree) => {
    const notes = chordOffsets.map((offset) =>
      midiToNote(
        scalePitch(
          rootMidi,
          progressionDegree + offset,
          resolved.scaleIntervals,
        ),
      ),
    );
    return `[${notes.join(",")}]`;
  });
  return `<${chords.join(" ")}>`;
}

function createBassPattern(
  dna: LanternTrailMusicalDNA,
  plan: SectionPlan,
  resolved: ResolvedTrailHarmony,
): string {
  const rootMidi = 36 + dna.harmony.rootPitchClass;
  const bars = resolved.progressionDegrees.map((degree) => {
    const root = scalePitch(rootMidi, degree, resolved.scaleIntervals);
    const fifth = scalePitch(rootMidi, degree + 4, resolved.scaleIntervals);
    const notes: readonly [number, number] =
      dna.arrangement.bassApproach === "root-octave"
        ? [root, root + 12]
        : dna.arrangement.bassApproach === "fifth-root"
          ? [fifth, root]
          : [root, fifth];
    if (plan.id === "camp") {
      return `[${midiToNote(notes[0])} ~ ~ ~]`;
    }
    if (plan.id === "danger" || plan.id === "sanctuary") {
      return `[${midiToNote(notes[0])} ${midiToNote(notes[1])} ${midiToNote(notes[0] + 12)} ${midiToNote(notes[1])}]`;
    }
    return `[${midiToNote(notes[0])} ~ ${midiToNote(notes[1])} ~]`;
  });
  return `<${bars.join(" ")}>`;
}

function createMelodyPattern(
  dna: LanternTrailMusicalDNA,
  plan: SectionPlan,
  resolved: ResolvedTrailHarmony,
): string {
  const wonderOctave = dna.traits.wonder >= 0.67 ? 12 : 0;
  const rootMidi = 60 + dna.harmony.rootPitchClass + wonderOctave;
  const onsetOrder = new Map(
    dna.rhythm.melodyOnsets.map((step, index) => [step, index]),
  );
  const bars = Array.from({ length: 4 }, (_, barIndex) => {
    const tokens = Array.from({ length: dna.rhythm.stepsPerBar }, () => "~");
    for (const step of dna.rhythm.melodyOnsets) {
      const noteIndex = onsetOrder.get(step);
      if (noteIndex === undefined) {
        throw new Error(`Missing melody onset index: ${step}`);
      }
      const rotation = dna.ornaments.barRotations[barIndex];
      const passingOffset = dna.ornaments.passingOffsets[barIndex];
      if (rotation === undefined || passingOffset === undefined) {
        throw new Error(`Missing bar development: ${barIndex}`);
      }
      const motifIndex =
        (noteIndex + rotation + (barIndex === 0 ? 0 : plan.development)) %
        dna.motif.degrees.length;
      const motifDegree = dna.motif.degrees[motifIndex];
      if (motifDegree === undefined) {
        throw new Error(`Missing motif degree: ${motifIndex}`);
      }
      const isDiscovery =
        barIndex === 3 && step === dna.ornaments.discoveryStep;
      const degree =
        barIndex === 0
          ? motifDegree
          : isDiscovery
            ? 0
            : motifDegree + plan.lift + passingOffset;
      tokens[step] = midiToNote(
        scalePitch(rootMidi, degree, resolved.scaleIntervals),
      );
    }
    return `[${tokens.join(" ")}]`;
  });
  return `<${bars.join(" ")}>`;
}

function createLiftPattern(
  dna: LanternTrailMusicalDNA,
  resolved: ResolvedTrailHarmony,
): string {
  const rootMidi = 72 + dna.harmony.rootPitchClass;
  const chordOffsets = resolved.chordSize === 4 ? [0, 2, 4, 6] : [0, 2, 4];
  const bars = resolved.progressionDegrees.map((degree) => {
    const tokens = Array.from({ length: dna.rhythm.stepsPerBar }, (_, step) =>
      step % 2 === 0
        ? midiToNote(
            scalePitch(
              rootMidi,
              degree + (chordOffsets[(step / 2) % chordOffsets.length] ?? 0),
              resolved.scaleIntervals,
            ),
          )
        : "~",
    );
    return `[${tokens.join(" ")}]`;
  });
  return `<${bars.join(" ")}>`;
}

function createPercussionPattern(
  dna: LanternTrailMusicalDNA,
  plan: SectionPlan,
  sectionIndex: number,
): string {
  const kickOnsets = new Set(dna.rhythm.kickOnsets);
  const snareOnsets = new Set(dna.rhythm.snareOnsets);
  const dangerBias = (dna.traits.danger - 0.5) * 0.4;
  const density = clamp(plan.drumDensity + dangerBias, 0.15, 1);
  const hatStride = density < 0.45 ? 4 : density < 0.7 ? 2 : 1;
  const bars = Array.from({ length: 4 }, (_, barIndex) => {
    const tokens: string[] = Array.from(
      { length: dna.rhythm.stepsPerBar },
      (_, step) => {
        if (kickOnsets.has(step)) {
          return "kick";
        }
        if (snareOnsets.has(step)) {
          return "snare";
        }
        return (step + barIndex + sectionIndex) % hatStride === 0
          ? "hat"
          : "~";
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
  dna: LanternTrailMusicalDNA,
  plan: SectionPlan,
): number {
  return clamp(
    base + dna.traits.danger * 0.22 + plan.intensity * 0.24,
    0.1,
    0.96,
  );
}

function createSection(
  dna: LanternTrailMusicalDNA,
  plan: SectionPlan,
  sectionIndex: number,
): AuthoringSection {
  const resolved = resolveSectionHarmony(dna, plan);
  const lanes = [
    {
      kind: "note" as const,
      id: `${plan.id}-harmony`,
      pattern: createChordPattern(dna, plan, resolved),
      voice: dna.timbre.harmonyVoice,
      velocity: velocity(0.18, dna, plan),
      gate: dna.arrangement.chordGate,
    },
    {
      kind: "note" as const,
      id: `${plan.id}-melody`,
      pattern: createMelodyPattern(dna, plan, resolved),
      voice: dna.timbre.melodyVoice,
      role: "melody" as const,
      velocity: velocity(0.24, dna, plan),
      gate: dna.arrangement.melodyGate,
    },
    {
      kind: "note" as const,
      id: `${plan.id}-bass`,
      pattern: createBassPattern(dna, plan, resolved),
      voice: dna.timbre.bassVoice,
      velocity: velocity(0.3, dna, plan),
      gate: 0.58 + dna.traits.danger * 0.24,
    },
    {
      kind: "percussion" as const,
      id: `${plan.id}-kit`,
      pattern: createPercussionPattern(dna, plan, sectionIndex),
      velocity: velocity(0.2, dna, plan),
    },
    ...(plan.id === "sanctuary"
      ? [
          {
            kind: "note" as const,
            id: `${plan.id}-glow`,
            pattern: createLiftPattern(dna, resolved),
            voice: dna.timbre.melodyVoice,
            velocity: velocity(0.28, dna, plan),
            gate: 0.42,
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

const MODE_LABELS: Readonly<Record<LanternTrailMode, string>> = {
  aeolian: "Aeolian",
  dorian: "Dorian",
  mixolydian: "Mixolydian",
  lydian: "Lydian",
  phrygian: "Phrygian",
};

function scoreId(dna: LanternTrailMusicalDNA): string {
  const version = LANTERN_TRAIL_GENERATOR_VERSION.replaceAll(".", "-");
  const identity = hashText(
    `${dna.levelSeed}:${JSON.stringify(dna.traits)}`,
  )
    .toString(16)
    .padStart(8, "0");
  return `lantern-trail-generated-v${version}-${identity}`;
}

function createAuthoringScore(dna: LanternTrailMusicalDNA): AuthoringScore {
  const modeLabel = MODE_LABELS[dna.harmony.mode] ?? "Trail";
  return {
    id: scoreId(dna),
    title: `Lantern Trail in ${dna.harmony.key.toUpperCase()} ${modeLabel}`,
    bpm: dna.arrangement.bpm,
    beatsPerBar: 4,
    ticksPerBeat: 960,
    crossfadeBars: 2,
    defaultSection: "camp",
    sections: SECTION_PLANS.map((plan, index) => createSection(dna, plan, index)),
    rules: lanternTrailRules,
  };
}

export function generateLanternTrailAuthoringScore(
  input: LanternTrailGeneratorInput,
): AuthoringScore {
  return createAuthoringScore(createDNA(input));
}

export function generateLanternTrailAdventure(
  input: LanternTrailGeneratorInput,
): GeneratedLanternTrailAdventure {
  const dna = createDNA(input);
  const authoringScore = createAuthoringScore(dna);
  const portableScore = exportScore(
    authoringScore,
    `${LANTERN_TRAIL_GENERATOR_VERSION}:${dna.levelSeed}`,
  );
  return {
    generatorVersion: LANTERN_TRAIL_GENERATOR_VERSION,
    levelSeed: dna.levelSeed,
    traits: dna.traits,
    domainSeeds: dna.domainSeeds,
    dna,
    authoringScore,
    portableScore,
  };
}
