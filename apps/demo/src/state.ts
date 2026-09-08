import {
  AdaptiveTransport,
  type GameState,
  type PortableScore,
  type PortableSection,
  type SectionId,
  type TransitionPlan,
} from "../../../packages/runtime/src/index.ts";
import {
  pocketCircuitExperiments,
  type NormalizedMusicTraits,
  type PocketCircuitStyle,
} from "../../../packages/studio/src/index.ts";
import { generateScore } from "./wasm-engine.ts";
import { DemoAudioEngine, type SoloMode } from "./audio-engine.ts";
import { elements } from "./dom";

export interface GenerationPreset {
  style: PocketCircuitStyle;
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

export function generationPreset(index = activeExperimentIndex): GenerationPreset {
  const preset = GENERATION_PRESETS[index];
  if (preset === undefined) {
    throw new Error(`Missing generation preset: ${index}`);
  }
  return preset;
}

export async function generateCurrentScore(): Promise<PortableScore> {
  const preset = generationPreset();
  return generateScore({
    seed: levelSeed,
    style: preset.style,
    energy: generationTraits.energy,
    complexity: generationTraits.complexity,
    brightness: generationTraits.brightness,
    syncopation: generationTraits.syncopation,
  });
}

export async function initializeLab(): Promise<void> {
  score = await generateCurrentScore();
  transport = new AdaptiveTransport(score);
  audio = new DemoAudioEngine(score);
}

export function currentState(): GameState {
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

export async function activateExperiment(
  index: number,
  nextSeed = levelSeed,
  nextTraits: NormalizedMusicTraits = generationTraits,
): Promise<boolean> {
  if (
    switchingScore ||
    switchingAudio ||
    pocketCircuitExperiments[index] === undefined ||
    GENERATION_PRESETS[index] === undefined
  ) {
    return false;
  }
  switchingScore = true;
  const previousAudio = audio;
  const wasRunning = previousAudio.running;
  const currentTick = previousAudio.currentTick();
  const previousSnapshot = transport.snapshot();
  const initialSection =
    auditionOverride ??
    (previousSnapshot.transition !== null &&
    currentTick >= previousSnapshot.transition.startTick
      ? previousSnapshot.transition.to
      : previousSnapshot.currentSection);
  try {
    activeExperimentIndex = index;
    levelSeed = nextSeed;
    generationTraits = { ...nextTraits };
    score = await generateCurrentScore();
    transport = new AdaptiveTransport(score, initialSection);
    audio = new DemoAudioEngine(score);
    audio.soloMode = soloMode;
    const startNext = wasRunning
      ? audio.start(initialSection)
      : Promise.resolve();
    await Promise.all([startNext, previousAudio.stop()]);
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

export function setSoloMode(value: SoloMode): void {
  soloMode = value;
  audio.soloMode = value;
}
