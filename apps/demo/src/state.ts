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
import {
  generateScore,
  isArrangement,
  type Arrangement,
} from "./wasm-engine.ts";
import { DemoAudioEngine, type SoloMode } from "./audio-engine.ts";
import { elements } from "./dom";
import { playbackSectionOnScore, isDebugBarSection } from "./playback-section.ts";
import {
  LAB_RECIPE_PROFILES,
  type GenerationPreset,
  type LabRecipe,
  type LabRecipeProfile,
  type NormalizedMusicTraits,
} from "./recipes.ts";

export {
  LAB_RECIPES,
  isLabRecipe,
  type GenerationPreset,
  type LabRecipe,
  type NormalizedMusicTraits,
} from "./recipes.ts";

export function labRecipeInfo(recipe: LabRecipe): LabRecipeProfile {
  return LAB_RECIPE_PROFILES[recipe];
}

export let labRecipe: LabRecipe = "racing";
/** The arrangement per recipe, so switching game types never leaks one
 *  recipe's choice into another. */
const arrangements: Record<LabRecipe, Arrangement> = {
  racing: "seeded",
  suspense: "seeded",
  adventure: "seeded",
  cozy: "seeded",
};
export let activeExperimentIndex = 0;
export let levelSeed = "level-001";
export let generationTraits: NormalizedMusicTraits = { ...LAB_RECIPE_PROFILES.racing.presets[0]!.traits };
export let phase = "garage";
export let score!: PortableScore;
export let transport!: AdaptiveTransport;
export let audio!: DemoAudioEngine;
export let soloMode: SoloMode = "full";
let manualCue: SectionId | null = null;

/** Version axis: steps the current piece (seed) through an unbounded run of
 *  takes. The engine derives each take's seed from the project secret, the
 *  piece seed, and a take index (not a chained hash), so the control can
 *  advance forever and jump straight to any version. The version is a second
 *  axis hashed *inside* the piece, not a different piece. */
export let versionIndex = 0;

export function advanceVersion(): void {
  versionIndex += 1;
}

/** The 1-based version number for the next step; unbounded. */
export function nextVersionNumber(): number {
  return versionIndex + 1;
}

let switchingScore = false;
let switchingAudio = false;
let generationQueue: Promise<void> = Promise.resolve();
let latestGenerationRequest = 0;

function presetsFor(recipe: LabRecipe): readonly GenerationPreset[] {
  return LAB_RECIPE_PROFILES[recipe].presets;
}

export function currentPresets(): readonly GenerationPreset[] {
  return presetsFor(labRecipe);
}

export function currentArrangement(): Arrangement {
  return arrangements[labRecipe];
}

/** Every recipe accepts the same two arrangements: all-phases or seeded. */
function recipeArrangementValid(_recipe: LabRecipe, value: Arrangement): boolean {
  return isArrangement(value);
}

export function generationPreset(index = activeExperimentIndex): GenerationPreset {
  const preset = currentPresets()[index];
  if (preset === undefined) {
    throw new Error(`Missing generation preset: ${index}`);
  }
  return preset;
}

export async function generateCurrentScore(): Promise<PortableScore> {
  return generateRequestedScore(
    activeExperimentIndex,
    levelSeed,
    generationTraits,
    labRecipe,
    currentArrangement(),
  );
}

async function generateRequestedScore(
  index: number,
  requestedSeed: string,
  requestedTraits: NormalizedMusicTraits,
  recipe: LabRecipe,
  arrangement: Arrangement,
  reelIndex = 0,
): Promise<PortableScore> {
  const preset = presetsFor(recipe)[index];
  if (preset === undefined) {
    throw new Error(`Missing generation preset: ${index}`);
  }
  return generateScore({
    seed: requestedSeed,
    style: preset.style,
    recipe,
    arrangement,
    autoplay: LAB_RECIPE_PROFILES[recipe].autoplay,
    energy: requestedTraits.energy,
    complexity: requestedTraits.complexity,
    brightness: requestedTraits.brightness,
    syncopation: requestedTraits.syncopation,
    tension: requestedTraits.energy,
    heat: requestedTraits.complexity,
    mystery: requestedTraits.brightness,
    pulse: requestedTraits.syncopation,
    reelIndex,
  });
}

export async function initializeLab(): Promise<void> {
  score = await generateCurrentScore();
  transport = new AdaptiveTransport(score);
  audio = new DemoAudioEngine(score);
}

export function currentState(): GameState {
  return LAB_RECIPE_PROFILES[labRecipe].gameState(phase, {
    intensity: Number(elements.intensity.value),
    pressure: Number(elements.pressure.value),
    flag: elements.finalLap.checked,
  });
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
  const phases = score.sections.filter((section) => !isDebugBarSection(section.id));
  if (cueControlsBusy() || phases.length === 0) return;
  const tick = audio.currentTick();
  const snapshot = transport.snapshot();
  const active =
    snapshot.transition !== null && tick >= snapshot.transition.startTick
      ? snapshot.transition.to
      : snapshot.currentSection;
  const index = phases.findIndex((section) => section.id === active);
  const base = index === -1 ? 0 : index;
  const nextIndex = (base + delta + phases.length) % phases.length;
  const next = phases[nextIndex];
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

/** A phase button: cue its section, or send it as game state where the
 *  recipe's phases are states rather than sections. */
export function requestPhase(): void {
  const target = LAB_RECIPE_PROFILES[labRecipe].phaseSections?.[phase];
  if (target === undefined) requestMusicState();
  else cueSection(target);
}

export async function activateExperiment(
  index: number,
  nextSeed: string,
  nextTraits: NormalizedMusicTraits,
  nextArrangement: Arrangement,
  requestId: number,
  reelIndex = 0,
): Promise<boolean> {
  const requestedRecipe = labRecipe;
  if (
    switchingScore ||
    switchingAudio ||
    presetsFor(requestedRecipe)[index] === undefined ||
    !recipeArrangementValid(requestedRecipe, nextArrangement)
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
    const nextScore = await generateRequestedScore(
      index,
      nextSeed,
      nextTraits,
      requestedRecipe,
      nextArrangement,
      reelIndex,
    );
    if (requestId !== latestGenerationRequest) {
      return false;
    }
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
    arrangements[requestedRecipe] = nextArrangement;
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
  nextArrangement: Arrangement = currentArrangement(),
  reelIndex = 0,
): Promise<boolean> {
  const request = ++latestGenerationRequest;
  const pending = generationQueue.then(() =>
    request === latestGenerationRequest
      ? activateExperiment(index, nextSeed, nextTraits, nextArrangement, request, reelIndex)
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

/** A new piece (seed/style/traits) restarts the version axis at 1. */
export function resetVersion(): void {
  versionIndex = 0;
}

export function setPhase(value: string): void {
  phase = value;
}

export function setArrangement(value: Arrangement): Promise<boolean> {
  if (!recipeArrangementValid(labRecipe, value)) {
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
  phase = LAB_RECIPE_PROFILES[recipe].openingPhase;
  generationTraits = { ...generationPreset(0).traits };
  return requestExperiment(0, levelSeed, generationTraits);
}

export function setSoloMode(value: SoloMode): void {
  soloMode = value;
  audio.soloMode = value;
}
