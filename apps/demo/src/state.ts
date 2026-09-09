import {
  AdaptiveTransport,
  type GameState,
  type PortableScore,
  type PortableSection,
  type SectionId,
  type TransitionPlan,
} from "../../../packages/runtime/src/index.ts";
import { type NormalizedMusicTraits } from "../../../packages/studio/src/index.ts";
import { generateScore } from "./wasm-engine.ts";
import { DemoAudioEngine, type SoloMode } from "./audio-engine.ts";
import { elements } from "./dom";
import {
  playbackSectionOnScore,
  SUSPENSE_PHASE_SECTIONS,
} from "./playback-section.ts";

export type LabRecipe = "pocket-circuit" | "suspense";

export interface GenerationPreset {
  style: string;
  traits: NormalizedMusicTraits;
}

export const GENERATION_PRESETS = [
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
] as const satisfies readonly GenerationPreset[];

export const SUSPENSE_PRESETS = [
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
] as const satisfies readonly GenerationPreset[];

export let labRecipe: LabRecipe = "pocket-circuit";
export let activeExperimentIndex = 0;
export let levelSeed = "level-001";
export let generationTraits: NormalizedMusicTraits = { ...GENERATION_PRESETS[0].traits };
export let phase = "garage";
export let score!: PortableScore;
export let transport!: AdaptiveTransport;
export let audio!: DemoAudioEngine;
export let soloMode: SoloMode = "full";
export let comparisonBaseSeed = levelSeed;
export let auditionOverride: SectionId | null = null;

let switchingScore = false;
let switchingAudio = false;
let generationQueue: Promise<void> = Promise.resolve();
let latestGenerationRequest = 0;

export function currentPresets(): readonly GenerationPreset[] {
  return labRecipe === "suspense" ? SUSPENSE_PRESETS : GENERATION_PRESETS;
}

export function generationPreset(index = activeExperimentIndex): GenerationPreset {
  const preset = currentPresets()[index];
  if (preset === undefined) {
    throw new Error(`Missing generation preset: ${index}`);
  }
  return preset;
}

export async function generateCurrentScore(): Promise<PortableScore> {
  return generateRequestedScore(activeExperimentIndex, levelSeed, generationTraits);
}

async function generateRequestedScore(
  index: number,
  requestedSeed: string,
  requestedTraits: NormalizedMusicTraits,
): Promise<PortableScore> {
  const preset = generationPreset(index);
  return generateScore({
    seed: requestedSeed,
    style: preset.style,
    recipe: labRecipe,
    energy: requestedTraits.energy,
    complexity: requestedTraits.complexity,
    brightness: requestedTraits.brightness,
    syncopation: requestedTraits.syncopation,
    tension: requestedTraits.energy,
    heat: requestedTraits.complexity,
    mystery: requestedTraits.brightness,
    pulse: requestedTraits.syncopation,
  });
}

export async function initializeLab(): Promise<void> {
  score = await generateCurrentScore();
  transport = new AdaptiveTransport(score);
  audio = new DemoAudioEngine(score);
}

export function currentState(): GameState {
  if (labRecipe === "suspense") {
    return {
      numeric: {
        heat: Number(elements.intensity.value),
        focus: Number(elements.pressure.value),
        progress: elements.finalLap.checked ? 1 : 0,
      },
      categorical: {
        tracePhase: phase,
      },
    };
  }
  return {
    numeric: {
      intensity: Number(elements.intensity.value),
      positionPressure: Number(elements.pressure.value),
      finalLap: elements.finalLap.checked ? 1 : 0,
    },
    categorical: {
      racePhase: phase,
      finishResult: "win",
    },
  };
}

export function sectionById(id: string): PortableSection {
  const section = score.sections.find((candidate) => candidate.id === id);
  if (section === undefined) {
    throw new Error(`Missing score section: ${id}`);
  }
  return section;
}

export function applyPlan(plan: TransitionPlan): void {
  audio.applyTransition(plan);
  elements.orbit.style.setProperty("--mood-color", sectionById(plan.to).color);
}

export function requestMusicState(clearAuditionOverride = false): void {
  const state = currentState();
  if (clearAuditionOverride) {
    auditionOverride = null;
  }
  if (auditionOverride !== null) {
    // render via ui
    return;
  }
  if (audio.running) {
    const request = transport.requestState(state, audio.currentTick());
    if (request.status === "scheduled") {
      if (request.replacedPlan !== undefined) {
        audio.cancelTransition(request.replacedPlan);
      }
      applyPlan(request.plan);
    } else if (request.status === "cancelled") {
      audio.cancelTransition(request.plan);
    }
  }
}

export function jumpToSection(target: SectionId): void {
  const tick = audio.currentTick();
  auditionOverride = target;
  transport.jumpSection(target, tick);
  audio.jumpSection(target, tick);
}

export function requestSuspensePhase(): void {
  const target = SUSPENSE_PHASE_SECTIONS[phase];
  if (target === undefined) {
    requestMusicState(true);
    return;
  }
  auditionOverride = null;
  if (!audio.running) {
    transport.jumpSection(target, 0);
    return;
  }
  const request = transport.requestSection(target, audio.currentTick());
  if (request.status === "scheduled") {
    if (request.replacedPlan !== undefined) {
      audio.cancelTransition(request.replacedPlan);
    }
    applyPlan(request.plan);
  } else if (request.status === "cancelled") {
    audio.cancelTransition(request.plan);
  }
}

export async function activateExperiment(
  index: number,
  nextSeed = levelSeed,
  nextTraits: NormalizedMusicTraits = generationTraits,
): Promise<boolean> {
  if (
    switchingScore ||
    switchingAudio ||
    currentPresets()[index] === undefined
  ) {
    return false;
  }
  switchingScore = true;
  const previousAudio = audio;
  const wasRunning = previousAudio.running;
  const currentTick = previousAudio.currentTick();
  const previousSnapshot = transport.snapshot();
  const requestedSection =
    auditionOverride ??
    (previousSnapshot.transition !== null &&
    currentTick >= previousSnapshot.transition.startTick
      ? previousSnapshot.transition.to
      : previousSnapshot.currentSection);
  try {
    const nextScore = await generateRequestedScore(index, nextSeed, nextTraits);
    const initialSection = playbackSectionOnScore(nextScore, requestedSection);
    const nextTransport = new AdaptiveTransport(nextScore, initialSection);
    const nextAudio = new DemoAudioEngine(nextScore);
    nextAudio.soloMode = soloMode;
    const startNext = wasRunning
      ? nextAudio.start(initialSection)
      : Promise.resolve();
    await Promise.all([startNext, previousAudio.stop()]);
    activeExperimentIndex = index;
    levelSeed = nextSeed;
    generationTraits = { ...nextTraits };
    score = nextScore;
    transport = nextTransport;
    audio = nextAudio;
    return true;
  } finally {
    switchingScore = false;
  }
}

export function requestExperiment(
  index: number,
  nextSeed = levelSeed,
  nextTraits: NormalizedMusicTraits = generationTraits,
): Promise<boolean> {
  const request = ++latestGenerationRequest;
  const pending = generationQueue.then(() =>
    request === latestGenerationRequest
      ? activateExperiment(index, nextSeed, nextTraits)
      : false,
  );
  generationQueue = pending.then(
    () => undefined,
    () => undefined,
  );
  return pending;
}

export function nextLevelSeed(seed: string): string {
  const match = /^(.*?)(\d+)$/.exec(seed);
  if (match === null) {
    return `${seed}-2`;
  }
  const [, prefix = "", digits = ""] = match;
  return `${prefix}${String(Number(digits) + 1).padStart(digits.length, "0")}`;
}

export function traitsFromControls(): NormalizedMusicTraits {
  return {
    energy: Number(elements.generationEnergy.value),
    complexity: Number(elements.generationComplexity.value),
    brightness: Number(elements.generationBrightness.value),
    syncopation: Number(elements.generationSyncopation.value),
  };
}

export async function toggleEngine(): Promise<boolean> {
  if (switchingAudio || switchingScore) {
    return audio.running;
  }
  switchingAudio = true;
  try {
    if (audio.running) {
      const tick = audio.currentTick();
      const snapshot = transport.snapshot();
      const currentSection = auditionOverride ??
        (snapshot.transition !== null && tick >= snapshot.transition.startTick
          ? snapshot.transition.to
          : snapshot.currentSection);
      transport = new AdaptiveTransport(score, currentSection);
      await audio.stop();
      return false;
    }
    await audio.start(transport.snapshot().currentSection);
    return true;
  } finally {
    switchingAudio = false;
  }
}

export function setComparisonBaseSeed(value: string): void {
  comparisonBaseSeed = value;
}

export function setPhase(value: string): void {
  phase = value;
}

export async function setLabRecipe(recipe: LabRecipe): Promise<boolean> {
  if (recipe === labRecipe) {
    return true;
  }
  labRecipe = recipe;
  activeExperimentIndex = 0;
  auditionOverride = null;
  phase = recipe === "suspense" ? "scan" : "garage";
  generationTraits = { ...generationPreset(0).traits };
  return requestExperiment(0, levelSeed, generationTraits);
}

export function setSoloMode(value: SoloMode): void {
  soloMode = value;
  audio.soloMode = value;
}
