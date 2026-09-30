import type { GameState, SectionId } from "../../../packages/runtime/src/index.ts";
import { ADVENTURE_SCENE_SECTIONS, SUSPENSE_PHASE_SECTIONS } from "./playback-section.ts";
import { phaseName } from "./phase-names.ts";

export type LabRecipe = "racing" | "suspense" | "adventure";

/** The four generic generation sliders every recipe maps onto its own traits. */
export interface NormalizedMusicTraits {
  energy: number;
  complexity: number;
  brightness: number;
  syncopation: number;
}

export interface GenerationPreset {
  style: string;
  label?: string;
  traits: NormalizedMusicTraits;
}

/** The game-signal panel's readings: two sliders (0..1) and one switch. */
export interface SignalReadings {
  intensity: number;
  pressure: number;
  flag: boolean;
}

/** Everything the Lab needs to present and drive one game type. */
export interface LabRecipeProfile {
  id: LabRecipe;
  label: string;
  description: string;
  presets: readonly GenerationPreset[];
  /** Whether the Original arrangement gets the engine's automatic tour. */
  autoplay: boolean;
  openingPhase: string;
  /** The phase buttons, in order. */
  phases: readonly string[];
  /** Phase buttons that cue a section directly; absent when phases are game states. */
  phaseSections?: Readonly<Record<string, SectionId>>;
  traitLabels: Readonly<Record<keyof NormalizedMusicTraits, string>>;
  meters: { intensity: string; pressure: string; flag: string; flagCopy: string };
  signal(phase: string): string;
  gameState(phase: string, readings: SignalReadings): GameState;
}

const RACING: LabRecipeProfile = {
  id: "racing",
  label: "Racing",
  description: "Speed, pressure, final lap, finish",
  presets: [
    {
      style: "fusion",
      traits: { energy: 0.62, complexity: 0.68, brightness: 0.52, syncopation: 0.72 },
    },
    {
      style: "neon",
      traits: { energy: 0.7, complexity: 0.48, brightness: 0.82, syncopation: 0.35 },
    },
    {
      style: "funk",
      traits: { energy: 0.58, complexity: 0.75, brightness: 0.55, syncopation: 0.9 },
    },
    {
      style: "chip",
      traits: { energy: 0.8, complexity: 0.7, brightness: 0.72, syncopation: 0.62 },
    },
  ],
  autoplay: true,
  openingPhase: "garage",
  phases: ["garage", "grid", "race", "finish"],
  traitLabels: {
    energy: "Energy",
    complexity: "Complexity",
    brightness: "Brightness",
    syncopation: "Syncopation",
  },
  meters: {
    intensity: "Speed intensity",
    pressure: "Position pressure",
    flag: "Final lap",
    flagCopy: "Add the maximum-commitment layer",
  },
  signal: (phase) => `racePhase: ${phaseName(phase)}`,
  gameState: (phase, { intensity, pressure, flag }) => ({
    numeric: { intensity, positionPressure: pressure, finalLap: flag ? 1 : 0 },
    categorical: { racePhase: phase, finishResult: "win" },
  }),
};

const SUSPENSE: LabRecipeProfile = {
  id: "suspense",
  label: "Suspense",
  description: "Song form, hold & cue",
  presets: [
    {
      style: "terminal",
      traits: { energy: 0.62, complexity: 0.48, brightness: 0.72, syncopation: 0.55 },
    },
    {
      style: "cipher",
      traits: { energy: 0.58, complexity: 0.66, brightness: 0.6, syncopation: 0.7 },
    },
    {
      style: "noir",
      traits: { energy: 0.7, complexity: 0.4, brightness: 0.78, syncopation: 0.42 },
    },
    {
      style: "techno",
      traits: { energy: 0.6, complexity: 0.5, brightness: 0.5, syncopation: 0.5 },
    },
    {
      style: "trance",
      traits: { energy: 0.55, complexity: 0.4, brightness: 0.75, syncopation: 0.35 },
    },
  ],
  autoplay: false,
  openingPhase: "scan",
  phases: Object.keys(SUSPENSE_PHASE_SECTIONS),
  phaseSections: SUSPENSE_PHASE_SECTIONS,
  traitLabels: { energy: "Tension", complexity: "Heat", brightness: "Mystery", syncopation: "Pulse" },
  meters: {
    intensity: "Detection heat",
    pressure: "Focus",
    flag: "Extracted",
    flagCopy: "Hold the coda / disconnect",
  },
  signal: (phase) => `recipe: suspense  /  tracePhase: ${phaseName(phase)}`,
  gameState: (phase, { intensity, pressure, flag }) => ({
    numeric: { heat: intensity, focus: pressure, progress: flag ? 1 : 0 },
    categorical: { tracePhase: phase },
  }),
};

const ADVENTURE: LabRecipeProfile = {
  id: "adventure",
  label: "Adventure",
  description: "Camp, explore, town, dungeon, combat, boss, sanctuary, victory",
  presets: [
    {
      style: "folk",
      label: "Medieval Folk",
      traits: { energy: 0.5, complexity: 0.45, brightness: 0.68, syncopation: 0.5 },
    },
    {
      style: "dark",
      label: "Dark Fantasy",
      traits: { energy: 0.62, complexity: 0.55, brightness: 0.55, syncopation: 0.45 },
    },
    {
      style: "orchestral",
      label: "Orchestral RPG",
      traits: { energy: 0.4, complexity: 0.6, brightness: 0.7, syncopation: 0.3 },
    },
  ],
  autoplay: true,
  openingPhase: "camp",
  phases: Object.keys(ADVENTURE_SCENE_SECTIONS),
  phaseSections: ADVENTURE_SCENE_SECTIONS,
  traitLabels: { energy: "Danger", complexity: "Mystery", brightness: "Wonder", syncopation: "Motion" },
  meters: {
    intensity: "Discovery",
    pressure: "Threat",
    flag: "Quest complete",
    flagCopy: "Mark the quest complete",
  },
  signal: (phase) => `recipe: adventure  /  areaPhase: ${phaseName(phase)}`,
  gameState: (phase, { intensity, pressure, flag }) => ({
    numeric: { discovery: intensity, threat: pressure, questComplete: flag ? 1 : 0 },
    categorical: { areaPhase: phase },
  }),
};

export const LAB_RECIPE_PROFILES: Readonly<Record<LabRecipe, LabRecipeProfile>> = {
  racing: RACING,
  suspense: SUSPENSE,
  adventure: ADVENTURE,
};

/** The game-type selector's order. */
export const LAB_RECIPES: readonly LabRecipeProfile[] = [RACING, SUSPENSE, ADVENTURE];

export function isLabRecipe(value: string | undefined): value is LabRecipe {
  return value !== undefined && Object.hasOwn(LAB_RECIPE_PROFILES, value);
}
