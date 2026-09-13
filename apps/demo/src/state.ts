import {
  AdaptiveTransport,
  selectSection,
  type GameState,
  type PortableScore,
  type PortableSection,
  type SectionId,
  type TransitionPlan,
  type TransitionRequest,
} from "../../../packages/runtime/src/index.ts";
import { generateScore, type SuspenseArrangement } from "./wasm-engine.ts";
import { DemoAudioEngine, type SoloMode } from "./audio-engine.ts";
import { elements } from "./dom";
import {
  playbackSectionOnScore,
  ADVENTURE_SCENE_SECTIONS,
  SUSPENSE_PHASE_SECTIONS,
} from "./playback-section.ts";

export type LabRecipe = "racing" | "suspense" | "adventure";

/** Display metadata for the game-type selector. Add a recipe here and the
 *  selector, trigger, and option list all pick it up. */
export interface LabRecipeInfo {
  id: LabRecipe;
  label: string;
  description: string;
}

export const LAB_RECIPES: readonly LabRecipeInfo[] = [
  {
    id: "racing",
    label: "Racing",
    description: "Speed, pressure, final lap, finish",
  },
  {
    id: "suspense",
    label: "Suspense",
    description: "Song form, hold & cue",
  },
  {
    id: "adventure",
    label: "Adventure",
    description: "Camp, explore, town, dungeon, combat, boss, sanctuary, victory",
  },
];

export function isLabRecipe(value: string | undefined): value is LabRecipe {
  return LAB_RECIPES.some((recipe) => recipe.id === value);
}

export function labRecipeInfo(recipe: LabRecipe): LabRecipeInfo {
  return LAB_RECIPES.find((entry) => entry.id === recipe) ?? LAB_RECIPES[0]!;
}

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

export const ADVENTURE_PRESETS = [
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
] as const satisfies readonly GenerationPreset[];

export let labRecipe: LabRecipe = "racing";
export let suspenseArrangement: SuspenseArrangement = "extended";
export let activeExperimentIndex = 0;
export let levelSeed = "level-001";
export let generationTraits: NormalizedMusicTraits = { ...GENERATION_PRESETS[0].traits };
export let phase = "garage";
export let score!: PortableScore;
export let transport!: AdaptiveTransport;
export let audio!: DemoAudioEngine;
export let soloMode: SoloMode = "full";
export let comparisonBaseSeed = levelSeed;
let manualCue: SectionId | null = null;

let switchingScore = false;
let switchingAudio = false;
let generationQueue: Promise<void> = Promise.resolve();
let latestGenerationRequest = 0;

const AUTOPLAY_RECIPES: ReadonlySet<LabRecipe> = new Set(["racing", "adventure"]);

export function currentPresets(): readonly GenerationPreset[] {
  if (labRecipe === "suspense") return SUSPENSE_PRESETS;
  if (labRecipe === "adventure") return ADVENTURE_PRESETS;
  return GENERATION_PRESETS;
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
  arrangement: SuspenseArrangement = suspenseArrangement,
): Promise<PortableScore> {
  const preset = generationPreset(index);
  return generateScore({
    seed: requestedSeed,
    style: preset.style,
    recipe: labRecipe,
    arrangement,
    autoplay: AUTOPLAY_RECIPES.has(labRecipe),
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
  if (labRecipe === "adventure") {
    return {
      numeric: {
        discovery: Number(elements.intensity.value),
        threat: Number(elements.pressure.value),
        questComplete: elements.finalLap.checked ? 1 : 0,
      },
      categorical: {
        areaPhase: phase,
      },
    };
  }
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

function applyRequest(request: TransitionRequest): void {
  if (request.status === "scheduled") {
    if (request.replacedPlan !== undefined) audio.cancelTransition(request.replacedPlan);
    manualCue = request.plan.to;
    applyPlan(request.plan);
  } else if (request.status === "queued") {
    manualCue = request.target;
  } else if (request.status === "cancelled") {
    manualCue = null;
    audio.cancelTransition(request.plan);
  }
}

export function pendingCue(): SectionId | null {
  const snapshot = transport.snapshot();
  const target = snapshot.pendingSection ?? (snapshot.transition !== null && audio.currentTick() < snapshot.transition.startTick ? snapshot.transition.to : null);
  if (target !== manualCue) manualCue = null;
  return manualCue;
}

export function requestMusicState(): void {
  if (cueControlsBusy()) return;
  if (!audio.running) {
    cueSection(selectSection(score, currentState()));
    return;
  }
  const tick = audio.currentTick();
  const automatic = transport.advance(tick);
  if (automatic !== null) applyPlan(automatic);
  applyRequest(transport.requestState(currentState(), tick));
}

export function cueSection(target: SectionId): void {
  if (cueControlsBusy()) return;
  sectionById(target);
  if (!audio.running) {
    manualCue = null;
    const held = transport.formHeld;
    transport = new AdaptiveTransport(score, target);
    transport.setFormHeld(held, 0);
    return;
  }
  const tick = audio.currentTick();
  const automatic = transport.advance(tick);
  if (automatic !== null) applyPlan(automatic);
  applyRequest(transport.requestSection(target, tick));
  const snapshot = transport.snapshot();
  if (snapshot.transition?.to === target && tick < snapshot.transition.startTick) manualCue = target;
}

// Cue the previous (-1) or next (+1) section in the score order, wrapping at the ends.
export function stepSection(delta: number): void {
  if (cueControlsBusy() || score.sections.length === 0) return;
  const tick = audio.currentTick();
  const snapshot = transport.snapshot();
  const active =
    snapshot.transition !== null && tick >= snapshot.transition.startTick
      ? snapshot.transition.to
      : snapshot.currentSection;
  const index = score.sections.findIndex((section) => section.id === active);
  const base = index === -1 ? 0 : index;
  const nextIndex = (base + delta + score.sections.length) % score.sections.length;
  const next = score.sections[nextIndex];
  if (next === undefined) return;
  cueSection(next.id);
}

export function setFormHold(held: boolean): void {
  if (cueControlsBusy()) return;
  const cancelled = transport.setFormHeld(held, audio.currentTick());
  if (cancelled !== null && audio.running) audio.cancelTransition(cancelled);
}

export function advanceSection(): void {
  if (cueControlsBusy() || transport.snapshot().transition !== null) return;
  const target = transport.nextFormSection(audio.currentTick());
  if (target !== null) cueSection(target);
}

export function cancelCue(): void {
  if (cueControlsBusy()) return;
  if (pendingCue() === null) return;
  const plan = transport.cancelPending(audio.currentTick());
  if (plan !== null) audio.cancelTransition(plan);
  manualCue = null;
}

export function cueControlsBusy(): boolean {
  return switchingAudio || switchingScore;
}

export function requestSuspensePhase(): void {
  const target = SUSPENSE_PHASE_SECTIONS[phase];
  if (target === undefined) requestMusicState();
  else cueSection(target);
}

export function requestAdventureScene(): void {
  const target = ADVENTURE_SCENE_SECTIONS[phase];
  if (target === undefined) requestMusicState();
  else cueSection(target);
}

export async function activateExperiment(
  index: number,
  nextSeed = levelSeed,
  nextTraits: NormalizedMusicTraits = generationTraits,
  nextArrangement: SuspenseArrangement = suspenseArrangement,
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
    (previousSnapshot.transition !== null &&
    currentTick >= previousSnapshot.transition.startTick
      ? previousSnapshot.transition.to
      : previousSnapshot.currentSection);
  try {
    const nextScore = await generateRequestedScore(index, nextSeed, nextTraits, nextArrangement);
    const initialSection = playbackSectionOnScore(nextScore, requestedSection);
    const nextTransport = new AdaptiveTransport(nextScore, initialSection);
    nextTransport.setFormHeld(transport.formHeld, 0);
    const nextAudio = new DemoAudioEngine(nextScore);
    nextAudio.soloMode = soloMode;
    const startNext = wasRunning
      ? nextAudio.start(
          initialSection,
          nextScore.form === undefined ? undefined : nextTransport.advance.bind(nextTransport),
        )
      : Promise.resolve();
    await Promise.all([startNext, previousAudio.stop()]);
    activeExperimentIndex = index;
    levelSeed = nextSeed;
    generationTraits = { ...nextTraits };
    suspenseArrangement = nextArrangement;
    score = nextScore;
    transport = nextTransport;
    audio = nextAudio;
    manualCue = null;
    return true;
  } finally {
    switchingScore = false;
  }
}

export function requestExperiment(
  index: number,
  nextSeed = levelSeed,
  nextTraits: NormalizedMusicTraits = generationTraits,
  nextArrangement: SuspenseArrangement = suspenseArrangement,
): Promise<boolean> {
  const request = ++latestGenerationRequest;
  const pending = generationQueue.then(() =>
    request === latestGenerationRequest
      ? activateExperiment(index, nextSeed, nextTraits, nextArrangement)
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
      const currentSection =
        (snapshot.transition !== null && tick >= snapshot.transition.startTick
          ? snapshot.transition.to
          : snapshot.currentSection);
      const held = transport.formHeld;
      transport = new AdaptiveTransport(score, currentSection);
      transport.setFormHeld(held, 0);
      manualCue = null;
      await audio.stop();
      return false;
    }
    await audio.start(
      transport.snapshot().currentSection,
      score.form === undefined ? undefined : transport.advance.bind(transport),
    );
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

export function setSuspenseArrangement(value: SuspenseArrangement): Promise<boolean> {
  if (labRecipe !== "suspense") {
    return Promise.resolve(false);
  }
  return requestExperiment(activeExperimentIndex, levelSeed, generationTraits, value);
}

export async function setLabRecipe(recipe: LabRecipe): Promise<boolean> {
  if (recipe === labRecipe) {
    return true;
  }
  labRecipe = recipe;
  activeExperimentIndex = 0;
  manualCue = null;
  phase =
    recipe === "suspense" ? "scan" : recipe === "adventure" ? "camp" : "garage";
  generationTraits = { ...generationPreset(0).traits };
  return requestExperiment(0, levelSeed, generationTraits);
}

export function setSoloMode(value: SoloMode): void {
  soloMode = value;
  audio.soloMode = value;
}
