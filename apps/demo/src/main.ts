import "./style.css";
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
  generatePocketCircuitLevel,
  pocketCircuitExperiments,
  POCKET_CIRCUIT_GENERATOR_VERSION,
  type GeneratedPocketCircuitLevel,
  type NormalizedMusicTraits,
  type PocketCircuitStyle,
} from "../../../packages/studio/src/index.ts";
import { DemoAudioEngine, type SoloMode } from "./audio-engine.ts";

type ViewName = "lab" | "games" | "genres";

interface GenerationPreset {
  style: PocketCircuitStyle;
  traits: NormalizedMusicTraits;
}

const GENERATION_PRESETS = [
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

function requireElement<ElementType extends Element>(selector: string): ElementType {
  const element = document.querySelector<ElementType>(selector);
  if (element === null) {
    throw new Error(`Missing Audio Lab element: ${selector}`);
  }
  return element;
}

const elements = {
  shell: requireElement<HTMLElement>(".console-shell"),
  start: requireElement<HTMLButtonElement>("#start-audio"),
  scoreButtons: requireElement<HTMLDivElement>("#score-buttons"),
  levelSeed: requireElement<HTMLInputElement>("#level-seed"),
  applySeed: requireElement<HTMLButtonElement>("#apply-seed"),
  newTake: requireElement<HTMLButtonElement>("#new-take"),
  variationValue: requireElement<HTMLOutputElement>("#variation-value"),
  generationEnergy: requireElement<HTMLInputElement>("#generation-energy"),
  generationEnergyValue: requireElement<HTMLOutputElement>("#generation-energy-value"),
  generationComplexity: requireElement<HTMLInputElement>("#generation-complexity"),
  generationComplexityValue: requireElement<HTMLOutputElement>("#generation-complexity-value"),
  generationBrightness: requireElement<HTMLInputElement>("#generation-brightness"),
  generationBrightnessValue: requireElement<HTMLOutputElement>("#generation-brightness-value"),
  generationSyncopation: requireElement<HTMLInputElement>("#generation-syncopation"),
  generationSyncopationValue: requireElement<HTMLOutputElement>("#generation-syncopation-value"),
  generatorSummary: requireElement<HTMLOutputElement>("#generator-summary"),
  soloButtons: requireElement<HTMLDivElement>("#solo-buttons"),
  compareTake: requireElement<HTMLButtonElement>("#compare-take"),
  compareValue: requireElement<HTMLOutputElement>("#compare-value"),
  auditionStatus: requireElement<HTMLOutputElement>("#audition-status"),
  scoreTitle: requireElement<HTMLElement>("#score-title"),
  tempo: requireElement<HTMLElement>("#tempo-value"),
  phaseButtons: requireElement<HTMLDivElement>("#phase-buttons"),
  intensity: requireElement<HTMLInputElement>("#intensity"),
  intensityValue: requireElement<HTMLOutputElement>("#intensity-value"),
  pressure: requireElement<HTMLInputElement>("#pressure"),
  pressureValue: requireElement<HTMLOutputElement>("#pressure-value"),
  finalLap: requireElement<HTMLInputElement>("#final-lap"),
  moodName: requireElement<HTMLElement>("#mood-name"),
  moodFeeling: requireElement<HTMLElement>("#mood-feeling"),
  transitionLabel: requireElement<HTMLElement>("#transition-label"),
  bar: requireElement<HTMLElement>("#bar-value"),
  beat: requireElement<HTMLElement>("#beat-value"),
  sectionList: requireElement<HTMLOListElement>("#section-list"),
  runtimeSignal: requireElement<HTMLElement>("#runtime-signal"),
  orbit: requireElement<HTMLElement>("#orbit"),
  genreIndex: requireElement<HTMLOListElement>("#genre-index"),
};

let activeExperimentIndex = 0;
let levelSeed = "level-001";
let generationTraits: NormalizedMusicTraits = { ...GENERATION_PRESETS[0].traits };
let phase = "garage";
let generatedLevel = createGeneratedLevel();
let score: PortableScore = generatedLevel.portableScore;
let transport = new AdaptiveTransport(score);
let audio = new DemoAudioEngine(score);
let switchingScore = false;
let switchingAudio = false;
let generationQueue: Promise<void> = Promise.resolve();
let latestGenerationRequest = 0;
let soloMode: SoloMode = "full";
let comparisonBaseSeed = levelSeed;
let auditionOverride: SectionId | null = null;

const sectionRows = new Map<
  string,
  { item: HTMLLIElement; output: HTMLOutputElement }
>();

function generationPreset(index = activeExperimentIndex): GenerationPreset {
  const preset = GENERATION_PRESETS[index];
  if (preset === undefined) {
    throw new Error(`Missing generation preset: ${index}`);
  }
  return preset;
}

function createGeneratedLevel(): GeneratedPocketCircuitLevel {
  return generatePocketCircuitLevel({
    seed: levelSeed,
    style: generationPreset().style,
    traits: generationTraits,
  });
}

function currentState(): GameState {
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

function sectionById(id: string): PortableSection {
  const section = score.sections.find((candidate) => candidate.id === id);
  if (section === undefined) {
    throw new Error(`Missing score section: ${id}`);
  }
  return section;
}

function applyPlan(plan: TransitionPlan): void {
  audio.applyTransition(plan);
  elements.orbit.style.setProperty("--mood-color", sectionById(plan.to).color);
}

function renderRuntimeSignal(state: GameState, target: SectionId): void {
  elements.runtimeSignal.textContent = [
    `score: ${score.id}`,
    `seed: ${levelSeed}`,
    `style: ${generatedLevel.style}`,
    `generation: E${generationTraits.energy.toFixed(2)} C${generationTraits.complexity.toFixed(2)} B${generationTraits.brightness.toFixed(2)} S${generationTraits.syncopation.toFixed(2)}`,
    `racePhase: ${phase}`,
    `intensity: ${state.numeric.intensity?.toFixed(2)}`,
    `pressure: ${state.numeric.positionPressure?.toFixed(2)}`,
    `finalLap: ${state.numeric.finalLap === 1}`,
    `solo: ${soloMode}`,
    `target: ${target}`,
  ].join("  /  ");
}

function requestMusicState(clearAuditionOverride = false): void {
  const state = currentState();
  const target = selectSection(score, state);
  if (clearAuditionOverride) {
    auditionOverride = null;
  }
  if (auditionOverride !== null) {
    renderRuntimeSignal(state, target);
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
  renderRuntimeSignal(state, target);
}

function createSectionRows(): void {
  sectionRows.clear();
  const rows = score.sections.map((section) => {
    const item = document.createElement("li");
    const label = document.createElement("span");
    const meter = document.createElement("i");
    const output = document.createElement("output");
    const jump = document.createElement("button");
    label.textContent = section.label;
    jump.type = "button";
    jump.dataset.jumpSection = section.id;
    jump.textContent = "Jump";
    jump.setAttribute("aria-label", `Jump to ${section.label}`);
    item.style.setProperty("--section-color", section.color);
    item.append(label, meter, output, jump);
    sectionRows.set(section.id, { item, output });
    return item;
  });
  elements.sectionList.replaceChildren(...rows);
}

function renderSections(): void {
  const tick = audio.currentTick();
  const mix = new Map(
    transport.mixAt(tick).map((item) => [item.section, item.gain]),
  );
  for (const section of score.sections) {
    const row = sectionRows.get(section.id);
    if (row === undefined) {
      continue;
    }
    const gain = mix.get(section.id) ?? 0;
    row.item.style.setProperty("--gain", String(gain));
    row.item.classList.toggle("is-audible", gain > 0.001);
    row.output.value = `${Math.round(gain * 100)}%`;
  }
}

function renderScoreButtons(): void {
  const buttons = pocketCircuitExperiments.map((experiment, index) => {
    const button = document.createElement("button");
    const label = document.createElement("strong");
    const genre = document.createElement("span");
    button.type = "button";
    button.dataset.experimentIndex = String(index);
    button.setAttribute("aria-pressed", String(index === activeExperimentIndex));
    label.textContent = experiment.shortLabel;
    genre.textContent = experiment.genre;
    button.append(label, genre);
    return button;
  });
  elements.scoreButtons.replaceChildren(...buttons);
}

function renderGenreIndex(): void {
  const rows = pocketCircuitExperiments.map((experiment, index) => {
    const item = document.createElement("li");
    const number = document.createElement("span");
    const copy = document.createElement("div");
    const title = document.createElement("strong");
    const description = document.createElement("small");
    const button = document.createElement("button");
    number.textContent = String(index + 1).padStart(2, "0");
    title.textContent = experiment.genre;
    description.textContent = experiment.description;
    copy.append(title, description);
    button.type = "button";
    button.dataset.experimentIndex = String(index);
    button.textContent = `Open ${experiment.score.title}`;
    item.append(number, copy, button);
    return item;
  });
  elements.genreIndex.replaceChildren(...rows);
}

function renderScoreIdentity(): void {
  const experiment = pocketCircuitExperiments[activeExperimentIndex];
  if (experiment === undefined) {
    return;
  }
  elements.scoreTitle.textContent = score.title;
  elements.tempo.textContent = String(score.bpm);
  renderGenerationControls();
  document.title = `Gamestruments Audio Lab — ${score.title}`;
  renderScoreButtons();
  createSectionRows();
  renderAuditionControls();
  requestMusicState();
}

function renderGenerationControls(): void {
  const traitControls = [
    [elements.generationEnergy, elements.generationEnergyValue, generationTraits.energy],
    [
      elements.generationComplexity,
      elements.generationComplexityValue,
      generationTraits.complexity,
    ],
    [
      elements.generationBrightness,
      elements.generationBrightnessValue,
      generationTraits.brightness,
    ],
    [
      elements.generationSyncopation,
      elements.generationSyncopationValue,
      generationTraits.syncopation,
    ],
  ] as const;
  elements.levelSeed.value = levelSeed;
  elements.variationValue.value = levelSeed;
  for (const [input, output, value] of traitControls) {
    input.value = String(value);
    output.value = `${Math.round(value * 100)}%`;
  }
  const { harmony, motif, rhythm, timbre } = generatedLevel.dna;
  elements.generatorSummary.value = [
    `${harmony.key.toUpperCase()} ${harmony.mode}`,
    `${motif.degrees.length}-step motif`,
    `${rhythm.melodyOnsets.length} notes/bar`,
    `${timbre.melodyVoice} lead · ${timbre.harmonyVoice}/${timbre.driveHarmonyVoice} pads`,
    `generator v${POCKET_CIRCUIT_GENERATOR_VERSION}`,
  ].join(" / ");
}

function announceAudition(message: string): void {
  elements.auditionStatus.value = message;
}

function renderAuditionControls(): void {
  for (const button of elements.soloButtons.querySelectorAll<HTMLButtonElement>(
    "button[data-solo]",
  )) {
    button.setAttribute("aria-pressed", String(button.dataset.solo === soloMode));
  }
  const showingComparison = levelSeed !== comparisonBaseSeed;
  const nextSeed = showingComparison ? comparisonBaseSeed : `${comparisonBaseSeed}:B`;
  elements.compareTake.setAttribute("aria-pressed", String(showingComparison));
  elements.compareValue.value = `${showingComparison ? "Play A" : "Play B"} ${nextSeed}`;
}

function setStartButton(running: boolean): void {
  const light = document.createElement("span");
  light.className = "start-light";
  light.setAttribute("aria-hidden", "true");
  elements.start.classList.toggle("is-running", running);
  elements.start.setAttribute("aria-pressed", String(running));
  elements.start.replaceChildren(
    light,
    document.createTextNode(running ? "Stop engine" : "Start engine"),
  );
}

async function activateExperiment(
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
    generatedLevel = createGeneratedLevel();
    score = generatedLevel.portableScore;
    transport = new AdaptiveTransport(score, initialSection);
    audio = new DemoAudioEngine(score);
    audio.soloMode = soloMode;
    renderScoreIdentity();
    const startNext = wasRunning
      ? audio.start(initialSection)
      : Promise.resolve();
    await Promise.all([startNext, previousAudio.stop()]);
    setStartButton(wasRunning);
    return true;
  } finally {
    switchingScore = false;
  }
}

function requestExperiment(
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

function jumpToSection(target: SectionId): void {
  const section = sectionById(target);
  const tick = audio.currentTick();
  auditionOverride = target;
  transport.jumpSection(target, tick);
  audio.jumpSection(target, tick);
  elements.orbit.style.setProperty("--mood-color", section.color);
  announceAudition(`Jumped to ${section.label}`);
}

function renderView(): void {
  const requested = window.location.hash.slice(1);
  const view: ViewName =
    requested === "games" || requested === "genres" ? requested : "lab";
  elements.shell.dataset.view = view;
  for (const candidate of document.querySelectorAll<HTMLElement>("[data-view]")) {
    candidate.hidden = candidate.dataset.view !== view;
  }
  for (const link of document.querySelectorAll<HTMLAnchorElement>("[data-view-link]")) {
    if (link.dataset.viewLink === view) {
      link.setAttribute("aria-current", "page");
    } else {
      link.removeAttribute("aria-current");
    }
  }
}

function renderFrame(): void {
  const tick = audio.currentTick();
  const barTicks = score.beatsPerBar * score.ticksPerBeat;
  const nextPlan = transport.advance(tick);
  if (nextPlan !== null) {
    applyPlan(nextPlan);
  }
  const snapshot = transport.snapshot();
  const activeTransition = snapshot.transition;
  const primarySection =
    activeTransition !== null && tick >= activeTransition.startTick
      ? activeTransition.to
      : snapshot.currentSection;
  const section = sectionById(primarySection);

  elements.bar.textContent = String(Math.floor(tick / barTicks) + 1).padStart(2, "0");
  elements.beat.textContent = String(
    Math.floor((tick % barTicks) / score.ticksPerBeat) + 1,
  ).padStart(2, "0");
  elements.moodName.textContent = section.label;
  elements.moodFeeling.textContent = section.feeling;
  elements.transitionLabel.textContent = !audio.running
    ? "Engine offline"
    : auditionOverride !== null
      ? "Audition override"
      : activeTransition === null
        ? "Pattern locked"
        : tick < activeTransition.startTick
          ? "Waiting for next bar"
          : `Crossing from ${sectionById(activeTransition.from).label}`;
  elements.orbit.style.setProperty("--mood-color", section.color);
  elements.orbit.style.setProperty(
    "--beat-progress",
    String((tick % score.ticksPerBeat) / score.ticksPerBeat),
  );
  renderSections();
  window.requestAnimationFrame(renderFrame);
}

function nextLevelSeed(seed: string): string {
  const match = /^(.*?)(\d+)$/.exec(seed);
  if (match === null) {
    return `${seed}-2`;
  }
  const [, prefix = "", digits = ""] = match;
  return `${prefix}${String(Number(digits) + 1).padStart(digits.length, "0")}`;
}

function traitsFromControls(): NormalizedMusicTraits {
  return {
    energy: Number(elements.generationEnergy.value),
    complexity: Number(elements.generationComplexity.value),
    brightness: Number(elements.generationBrightness.value),
    syncopation: Number(elements.generationSyncopation.value),
  };
}

function applyLevelSeed(): void {
  const requestedSeed = elements.levelSeed.value.trim();
  if (requestedSeed.length === 0) {
    elements.levelSeed.value = levelSeed;
    announceAudition("Level seed cannot be empty");
    return;
  }
  void requestExperiment(activeExperimentIndex, requestedSeed).then((applied) => {
    if (!applied) {
      return;
    }
    comparisonBaseSeed = levelSeed;
    renderAuditionControls();
    announceAudition(`Generated level ${levelSeed}`);
  });
}

elements.start.addEventListener("click", async () => {
  if (switchingAudio || switchingScore) {
    return;
  }
  switchingAudio = true;
  elements.start.disabled = true;
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
      setStartButton(false);
      announceAudition("Engine stopped");
      return;
    }
    await audio.start(transport.snapshot().currentSection);
    setStartButton(true);
    requestMusicState();
    announceAudition("Engine started");
  } finally {
    elements.start.disabled = false;
    switchingAudio = false;
  }
});

elements.scoreButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
    "button[data-experiment-index]",
  );
  if (button === null) {
    return;
  }
  const index = Number(button.dataset.experimentIndex);
  void requestExperiment(index, levelSeed, { ...generationPreset(index).traits }).then((applied) => {
    if (!applied) {
      return;
    }
    comparisonBaseSeed = levelSeed;
    renderAuditionControls();
    announceAudition(`Generated ${score.title}`);
  });
});

elements.newTake.addEventListener("click", () => {
  const nextSeed = nextLevelSeed(comparisonBaseSeed);
  void requestExperiment(activeExperimentIndex, nextSeed).then((applied) => {
    if (!applied) {
      return;
    }
    comparisonBaseSeed = levelSeed;
    renderAuditionControls();
    announceAudition(`Generated level ${levelSeed}`);
  });
});

elements.applySeed.addEventListener("click", applyLevelSeed);
elements.levelSeed.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    applyLevelSeed();
  }
});

for (const [input, output] of [
  [elements.generationEnergy, elements.generationEnergyValue],
  [elements.generationComplexity, elements.generationComplexityValue],
  [elements.generationBrightness, elements.generationBrightnessValue],
  [elements.generationSyncopation, elements.generationSyncopationValue],
] as const) {
  input.addEventListener("input", () => {
    output.value = `${Math.round(Number(input.value) * 100)}%`;
  });
  input.addEventListener("change", () => {
    void requestExperiment(activeExperimentIndex, levelSeed, traitsFromControls()).then(
      (applied) => {
        if (!applied) {
          return;
        }
        comparisonBaseSeed = levelSeed;
        renderAuditionControls();
        announceAudition(`Regenerated ${score.title}`);
      },
    );
  });
}

elements.soloButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
    "button[data-solo]",
  );
  const requested = button?.dataset.solo;
  if (
    requested !== "full" &&
    requested !== "melody" &&
    requested !== "rhythm"
  ) {
    return;
  }
  soloMode = requested;
  audio.soloMode = soloMode;
  renderAuditionControls();
  const state = currentState();
  renderRuntimeSignal(state, selectSection(score, state));
  announceAudition(
    soloMode === "full" ? "Full mix on" : `${soloMode} solo on`,
  );
});

elements.compareTake.addEventListener("click", () => {
  const nextSeed =
    levelSeed === comparisonBaseSeed ? `${comparisonBaseSeed}:B` : comparisonBaseSeed;
  void requestExperiment(activeExperimentIndex, nextSeed).then((applied) => {
    if (!applied) {
      return;
    }
    renderAuditionControls();
    const side = levelSeed === comparisonBaseSeed ? "A" : "B";
    announceAudition(`Playing seed ${side}: ${levelSeed}`);
  });
});

elements.sectionList.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
    "button[data-jump-section]",
  );
  if (button?.dataset.jumpSection !== undefined) {
    jumpToSection(button.dataset.jumpSection);
  }
});

elements.genreIndex.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
    "button[data-experiment-index]",
  );
  if (button === null) {
    return;
  }
  const index = Number(button.dataset.experimentIndex);
  void requestExperiment(index, levelSeed, { ...generationPreset(index).traits }).then((applied) => {
    if (!applied) {
      return;
    }
    comparisonBaseSeed = levelSeed;
    renderAuditionControls();
    window.location.hash = "lab";
    announceAudition(`Generated ${score.title}`);
  });
});

for (const button of document.querySelectorAll<HTMLButtonElement>("[data-open-lab]")) {
  button.addEventListener("click", () => {
    window.location.hash = "lab";
  });
}

elements.phaseButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
    "button[data-phase]",
  );
  if (button === null) {
    return;
  }
  phase = button.dataset.phase ?? "garage";
  for (const candidate of elements.phaseButtons.querySelectorAll("button")) {
    candidate.setAttribute("aria-pressed", String(candidate === button));
  }
  requestMusicState(true);
});

elements.intensity.addEventListener("input", () => {
  elements.intensityValue.value = `${Math.round(Number(elements.intensity.value) * 100)}%`;
  requestMusicState(true);
});

elements.pressure.addEventListener("input", () => {
  elements.pressureValue.value = `${Math.round(Number(elements.pressure.value) * 100)}%`;
  requestMusicState(true);
});

elements.finalLap.addEventListener("change", () => requestMusicState(true));
window.addEventListener("hashchange", renderView);

renderGenreIndex();
renderScoreIdentity();
renderView();
setStartButton(false);
window.requestAnimationFrame(renderFrame);
