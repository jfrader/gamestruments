import {
  AdaptiveTransport,
  selectSection,
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

export type ViewName = "lab" | "games" | "genres";

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
export let activeStyle: PocketCircuitStyle = GENERATION_PRESETS[0].style;
export let phase = "garage";
export let score!: PortableScore;
export let transport!: AdaptiveTransport;
export let audio!: DemoAudioEngine;
export let switchingScore = false;
export let switchingAudio = false;
export let generationQueue: Promise<void> = Promise.resolve();
export let latestGenerationRequest = 0;
export let soloMode: SoloMode = "full";
export let comparisonBaseSeed = levelSeed;
export let auditionOverride: SectionId | null = null;

export function generationPreset(index = activeExperimentIndex): GenerationPreset {
  const preset = GENERATION_PRESETS[index];
  if (preset === undefined) {
    throw new Error(`Missing generation preset: ${index}`);
  }
  return preset;
}

export async function generateCurrentScore(): Promise<PortableScore> {
  const preset = generationPreset();
  activeStyle = preset.style;
  return generateScore({
    seed: levelSeed,
    style: preset.style,
    energy: generationTraits.energy,
    complexity: generationTraits.complexity,
    brightness: generationTraits.brightness,
    syncopation: generationTraits.syncopation,
  });
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
  try {
    elements.orbit.style.setProperty("--mood-color", sectionById(plan.to).color);
  } catch {}
}

export function requestMusicState(clearAuditionOverride = false): void {
  const state = currentState();
  const _target = selectSection(score as any, state);
  void _target;
  if (clearAuditionOverride) {
    (auditionOverride as any) = null;
  }
  if (auditionOverride !== null) {
    // render via ui
    return;
  }
  if (audio.running) {
    const request = (transport as any).requestState(state, audio.currentTick());
    if (request.status === "scheduled") {
      if (request.replacedPlan !== undefined) {
        audio.cancelTransition(request.replacedPlan);
      }
      applyPlan(request.plan);
    } else if (request.status === "cancelled") {
      audio.cancelTransition(request.plan);
    }
  }
  // runtime signal via ui caller
}

export function jumpToSection(_s: SectionId): void {
  const target = _s;
  const tick = audio.currentTick();
  (auditionOverride as any) = target;
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
  try {
    return {
      energy: Number((elements as any).generationEnergy.value),
      complexity: Number((elements as any).generationComplexity.value),
      brightness: Number((elements as any).generationBrightness.value),
      syncopation: Number((elements as any).generationSyncopation.value),
    };
  } catch {
    return { ...generationTraits };
  }
}

export function applyLevelSeed(requested: string): void {
  // logic moved; wiring in main calls with value from ui
  if (requested.length === 0) {
    return;
  }
  void requestExperiment(activeExperimentIndex, requested).then((applied) => {
    if (!applied) return;
    comparisonBaseSeed = levelSeed;
  });
}

export async function toggleEngine(): Promise<void> {
  if (switchingAudio || switchingScore) {
    return;
  }
  switchingAudio = true;
  // direct elements for sync
  const startEl = (elements as any).start;
  const centerEl = (elements as any).centerPlay;
  if (startEl) startEl.disabled = true;
  if (centerEl) centerEl.disabled = true;
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
      // sync buttons via ui/set if avail
      return;
    }
    await audio.start(transport.snapshot().currentSection);
  } finally {
    if (startEl) startEl.disabled = false;
    if (centerEl) centerEl.disabled = false;
    switchingAudio = false;
  }
}

// setters for ESM import binding mutation from main
export function setComparisonBaseSeed(v: string) { comparisonBaseSeed = v; }
export function setLevelSeed(v: string) { levelSeed = v; }
export function setGenerationTraits(v: any) { generationTraits = v; }
export function setPhase(v: string) { phase = v; }
export function setSoloMode(v: SoloMode) { soloMode = v; }
export function setAuditionOverride(v: any) { auditionOverride = v; }
export function setActiveExperimentIndex(v: number) { activeExperimentIndex = v; }
