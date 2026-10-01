import type { PortableScore, PortableSection, SectionId } from "../../../packages/runtime/src/index.ts";
import {
  generateScore,
  isArrangement,
  rootPitchClass,
  type Arrangement,
  type GenerateScoreParams,
} from "./wasm-engine.ts";
import { elements } from "./dom";
import { isDebugBarSection } from "./playback-section.ts";
import { soundingSection, type LabScore, type SoloMode } from "./playback.ts";
import { EnginePlayback } from "./engine-playback.ts";
import {
  LAB_RECIPE_PROFILES,
  type GenerationPreset,
  type LabRecipe,
  type LabRecipeProfile,
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
};
export let activeExperimentIndex = 0;
export let levelSeed = "level-001";
export let generationTraits: NormalizedMusicTraits = { ...LAB_RECIPE_PROFILES.racing.presets[0]!.traits };
export let phase = "garage";
export let score!: PortableScore;
export let playback!: EnginePlayback;
export let soloMode: SoloMode = "full";

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

export async function generateCurrentScore(): Promise<LabScore> {
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
): Promise<LabScore> {
  const preset = presetsFor(recipe)[index];
  if (preset === undefined) {
    throw new Error(`Missing generation preset: ${index}`);
  }
  const params: GenerateScoreParams = {
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
  };
  return {
    score: await generateScore(params),
    seed: requestedSeed,
    rootPitchClass: await rootPitchClass(params),
    profile: LAB_RECIPE_PROFILES[recipe],
  };
}

export async function initializeLab(): Promise<void> {
  const lab = await generateCurrentScore();
  score = lab.score;
  playback = await EnginePlayback.create(lab, score.defaultSection, false);
}

function sectionById(id: string): PortableSection {
  const section = score.sections.find((candidate) => candidate.id === id);
  if (section === undefined) {
    throw new Error(`Missing score section: ${id}`);
  }
  return section;
}

export function requestMusicState(): void {
  if (cueControlsBusy()) return;
  playback.request(phase, {
    intensity: Number(elements.intensity.value),
    pressure: Number(elements.pressure.value),
    flag: elements.finalLap.checked,
  });
}

export function cueSection(target: SectionId): void {
  if (cueControlsBusy()) return;
  sectionById(target);
  playback.cue(target);
}

// Cue the previous (-1) or next (+1) section in the score order, wrapping at the ends.
export function stepSection(delta: number): void {
  const phases = score.sections.filter((section) => !isDebugBarSection(section.id));
  if (cueControlsBusy() || phases.length === 0) return;
  const frame = playback.frame();
  const active = soundingSection(frame.snapshot, frame.tick);
  const index = phases.findIndex((section) => section.id === active);
  const base = index === -1 ? 0 : index;
  const nextIndex = (base + delta + phases.length) % phases.length;
  const next = phases[nextIndex];
  if (next === undefined) return;
  cueSection(next.id);
}

export function setFormHold(held: boolean): void {
  if (cueControlsBusy()) return;
  playback.setHold(held);
}

export function advanceSection(): void {
  if (cueControlsBusy()) return;
  playback.advance();
}

export function cancelCue(): void {
  if (cueControlsBusy()) return;
  playback.cancelCue();
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
  try {
    const next = await generateRequestedScore(
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
    playback.switchTo(next);
    activeExperimentIndex = index;
    levelSeed = nextSeed;
    generationTraits = { ...nextTraits };
    arrangements[requestedRecipe] = nextArrangement;
    score = next.score;
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
    return playback.running;
  }
  switchingAudio = true;
  try {
    if (playback.running) {
      await playback.stop();
      return false;
    }
    await playback.start();
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
  phase = LAB_RECIPE_PROFILES[recipe].openingPhase;
  generationTraits = { ...generationPreset(0).traits };
  return requestExperiment(0, levelSeed, generationTraits);
}

export function setSoloMode(value: SoloMode): void {
  soloMode = value;
  playback.soloMode = value;
}
