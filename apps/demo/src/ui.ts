import { pocketCircuitExperiments } from "../../../packages/studio/src/index.ts";
import type { NormalizedMusicTraits } from "../../../packages/studio/src/index.ts";
import type {
  AdaptiveTransport,
  PortableScore,
  PortableSection,
  SectionId,
  TransitionPlan,
} from "../../../packages/runtime/src/index.ts";
import type { DemoAudioEngine, SoloMode } from "./audio-engine.ts";
import type { SuspenseArrangement } from "./wasm-engine.ts";
import { requireElement, elements } from "./dom";
import { orbitMotionAt, orbitStyleAt } from "./orbit-visualizer.ts";
import { cueView } from "./section-cues.ts";
import { SUSPENSE_PHASE_SECTIONS } from "./playback-section.ts";

export type ViewName = "lab" | "games" | "genres";
const ARRANGEMENT_DESCRIPTIONS: Record<SuspenseArrangement, string> = {
  original: "The current sound, unchanged.",
  extended: "Scan → Scan II and Breach → Breach II are independent 16-bar sections. Hold, cue or advance them to match gameplay. Anomaly stays intact.",
};
export { requireElement, elements };

const sectionRows = new Map<
  string,
  { item: HTMLLIElement; output: HTMLOutputElement; button: HTMLButtonElement }
>();

export function createSectionRows(score: PortableScore): void {
  sectionRows.clear();
  const rows = score.sections.map((section) => {
    const item = document.createElement("li");
    const label = document.createElement("span");
    const meter = document.createElement("i");
    const output = document.createElement("output");
    const jump = document.createElement("button");
    label.textContent = score.form === undefined
      ? section.label
      : `${section.label} · ${section.lengthTicks / (score.beatsPerBar * score.ticksPerBeat)} bars`;
    jump.type = "button";
    jump.dataset.cueSection = section.id;
    jump.textContent = "Cue";
    jump.setAttribute("aria-label", `Cue ${section.label}`);
    item.style.setProperty("--section-color", section.color);
    item.append(label, meter, output, jump);
    sectionRows.set(section.id, { item, output, button: jump });
    return item;
  });
  elements.sectionList.replaceChildren(...rows);
  elements.sectionSelect.replaceChildren(...score.sections.map((section) => {
    const option = document.createElement("option");
    option.value = section.id;
    option.textContent = `${section.label} · ${section.lengthTicks / (score.beatsPerBar * score.ticksPerBeat)} bars`;
    return option;
  }));
}

export function renderSections(
  score: PortableScore,
  transport: AdaptiveTransport,
  audio: DemoAudioEngine,
  requested: SectionId | null,
  busy: boolean,
): void {
  const visualTick = audio.currentVisualTick();
  const tick = Math.floor(visualTick);
  const mix = new Map(
    transport.mixAt(tick).map((item) => [item.section, item.gain]),
  );
  const view = cueView(score, transport.snapshot(), tick, audio.running, requested, transport.formHeld);
  const next = transport.nextFormSection(tick);
  elements.holdForm.disabled = busy || score.form === undefined;
  elements.holdForm.setAttribute("aria-pressed", String(transport.formHeld));
  elements.holdForm.textContent = transport.formHeld ? "Resume automatic" : "Hold section";
  elements.advanceForm.disabled = busy || next === null || transport.snapshot().transition !== null;
  elements.advanceForm.textContent = next === null ? "Next section" : `Next: ${score.sections.find((section) => section.id === next)?.label ?? next}`;
  const status = busy ? "Preparing playback…" : view.status;
  const detail = busy ? "Please wait before cueing a section." : view.detail;
  if (elements.cueStatus.textContent !== status) elements.cueStatus.textContent = status;
  if (elements.cueDetail.textContent !== detail) elements.cueDetail.textContent = detail;
  elements.cancelCue.disabled = busy || !view.cancellable;
  elements.sectionSelect.disabled = busy;
  for (const control of [elements.intensity, elements.pressure, elements.finalLap, ...elements.phaseButtons.querySelectorAll<HTMLButtonElement>("button")]) control.disabled = busy;
  if (document.activeElement !== elements.sectionSelect) elements.sectionSelect.value = view.select;
  for (const section of score.sections) {
    const row = sectionRows.get(section.id);
    if (row === undefined) {
      continue;
    }
    const gain = mix.get(section.id) ?? 0;
    row.item.style.setProperty("--gain-width", `${gain * 100}%`);
    row.item.classList.toggle("is-audible", gain > 0.001);
    const queued = section.id === view.target;
    const current = section.id === view.current;
    row.item.classList.toggle("is-cued", queued);
    if (current) row.item.setAttribute("aria-current", "true");
    else row.item.removeAttribute("aria-current");
    row.button.textContent = queued ? view.cancellable ? "Queued" : "Next" : current ? audio.running ? "Playing" : "Selected" : "Cue";
    row.button.disabled = busy || (current && !view.cancellable);
    row.output.value = `${Math.round(gain * 100)}%`;
  }
}

export function renderScoreButtons(
  activeExperimentIndex: number,
  presets: readonly { style: string }[],
  recipe: "racing" | "suspense",
): void {
  const racingLabels = ["Tiny Torque", "Neon Drift", "Countertop", "8-Bit"];
  const buttons = presets.map((preset, index) => {
    const button = document.createElement("button");
    const label = document.createElement("strong");
    const genre = document.createElement("span");
    button.type = "button";
    button.dataset.experimentIndex = String(index);
    button.setAttribute("aria-pressed", String(index === activeExperimentIndex));
    label.textContent =
      recipe === "suspense"
        ? preset.style
        : (racingLabels[index] ?? preset.style);
    genre.textContent = preset.style;
    button.append(label, genre);
    return button;
  });
  elements.scoreButtons.replaceChildren(...buttons);
}

export function renderRecipeChrome(recipe: "racing" | "suspense", phase: string): void {
  const suspense = recipe === "suspense";
  elements.shell.dataset.recipe = recipe;
  elements.sectionControl.hidden = !suspense;
  elements.gameSignals.dataset.recipe = recipe;
  for (const button of elements.recipeButtons.querySelectorAll<HTMLButtonElement>(
    "button[data-recipe]",
  )) {
    button.setAttribute(
      "aria-pressed",
      String(button.dataset.recipe === recipe),
    );
  }
  elements.traitEnergyLabel.textContent = suspense ? "Tension" : "Energy";
  elements.traitComplexityLabel.textContent = suspense ? "Heat" : "Complexity";
  elements.traitBrightnessLabel.textContent = suspense ? "Mystery" : "Brightness";
  elements.traitSyncopationLabel.textContent = suspense ? "Pulse" : "Syncopation";
  elements.meterIntensityLabel.textContent = suspense ? "Detection heat" : "Speed intensity";
  elements.meterPressureLabel.textContent = suspense ? "Focus" : "Position pressure";
  elements.meterFinalLabel.textContent = suspense ? "Extracted" : "Final lap";
  elements.meterFinalCopy.textContent = suspense
    ? "Hold the coda / disconnect"
    : "Add the maximum-commitment layer";
  const phases = suspense
    ? Object.keys(SUSPENSE_PHASE_SECTIONS).map((id) => [id, id.charAt(0).toUpperCase() + id.slice(1)] as const)
    : ([
        ["garage", "Garage"],
        ["grid", "Grid"],
        ["race", "Race"],
        ["finish", "Finish"],
      ] as const);
  const buttons = phases.map(([id, label]) => {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.phase = id;
    button.textContent = label;
    button.setAttribute("aria-pressed", String(id === phase));
    return button;
  });
  elements.phaseButtons.replaceChildren(...buttons);
}

export function renderGenreIndex(): void {
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

export function renderScoreIdentity(
  activeExperimentIndex: number,
  score: PortableScore,
  levelSeed: string,
  generationTraits: NormalizedMusicTraits,
  comparisonBaseSeed: string,
  soloMode: SoloMode,
  recipe: "racing" | "suspense",
  presets: readonly { style: string }[],
  arrangement: SuspenseArrangement,
  phase: string,
): void {
  elements.scoreTitle.textContent = score.title;
  elements.tempo.textContent = String(score.bpm);
  renderRecipeChrome(recipe, phase);
  elements.arrangementControl.hidden = recipe !== "suspense";
  for (const button of elements.arrangementButtons.querySelectorAll<HTMLButtonElement>("button[data-arrangement]")) {
    button.setAttribute("aria-pressed", String(button.dataset.arrangement === arrangement));
  }
  elements.arrangementSummary.value = recipe === "suspense" ? ARRANGEMENT_DESCRIPTIONS[arrangement] : "";
  renderGenerationControls(score, levelSeed, generationTraits);
  document.title = `Gamestruments Audio Lab — ${score.title}`;
  renderScoreButtons(activeExperimentIndex, presets, recipe);
  createSectionRows(score);
  renderAuditionControls(levelSeed, comparisonBaseSeed, soloMode);
}

export function renderGenerationControls(
  score: PortableScore,
  levelSeed: string,
  generationTraits: NormalizedMusicTraits,
): void {
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
  elements.generatorSummary.value = [
    score.id,
    `${score.sections.length} sections @ ${score.bpm} bpm`,
    "engine: wasm",
  ].join(" / ");
}

export function announceAudition(message: string): void {
  elements.auditionStatus.value = message;
}

export function renderAuditionControls(levelSeed: string, comparisonBaseSeed: string, soloMode: SoloMode): void {
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

export function setStartButton(running: boolean): void {
  const light = document.createElement("span");
  light.className = "start-light";
  light.setAttribute("aria-hidden", "true");
  elements.start.classList.toggle("is-running", running);
  elements.start.setAttribute("aria-pressed", String(running));
  elements.start.replaceChildren(
    light,
    document.createTextNode(running ? "Stop engine" : "Start engine"),
  );
  elements.centerPlay.classList.toggle("is-running", running);
  elements.centerPlay.setAttribute("aria-pressed", String(running));
  const glyph = elements.centerPlay.querySelector<HTMLElement>(".center-glyph");
  const label = elements.centerPlay.querySelector<HTMLElement>(".center-label");
  if (glyph !== null) {
    glyph.textContent = running ? "❚❚" : "▶";
  }
  if (label !== null) {
    label.textContent = running ? "PAUSE" : "PLAY";
  }
}

export function setPlaybackPending(pending: boolean): void {
  elements.start.disabled = pending;
  elements.centerPlay.disabled = pending;
  if (pending) {
    elements.sectionSelect.disabled = true;
    elements.cancelCue.disabled = true;
    elements.holdForm.disabled = true;
    elements.advanceForm.disabled = true;
    for (const row of sectionRows.values()) row.button.disabled = true;
  }
}

export function renderView(): void {
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

export function renderFrame(
  audio: DemoAudioEngine,
  score: PortableScore,
  transport: AdaptiveTransport,
  requested: SectionId | null,
  applyPlan: (plan: TransitionPlan) => void,
  sectionById: (id: string) => PortableSection,
  busy = false,
): void {
  const visualTick = audio.currentVisualTick();
  const tick = Math.floor(visualTick);
  const barTicks = score.beatsPerBar * score.ticksPerBeat;
  const nextPlan = score.form === undefined ? transport.advance(tick) : null;
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
  if (elements.moodName.textContent !== section.label) elements.moodName.textContent = section.label;
  elements.moodFeeling.textContent = section.feeling;
  elements.transitionLabel.textContent = !audio.running
    ? "Engine offline"
    : snapshot.pendingSection !== null
      ? `Queued: ${sectionById(snapshot.pendingSection).label} · after this blend`
      : activeTransition === null
        ? score.form === undefined
          ? "Pattern locked"
           : transport.formHeld ? "Holding section" : "Form playing"
        : tick < activeTransition.startTick
           ? `Waiting for next bar → ${sectionById(activeTransition.to).label}`
          : `Crossing from ${sectionById(activeTransition.from).label}`;
  elements.orbit.style.setProperty("--mood-color", section.color);
  elements.orbit.classList.toggle("is-running", audio.running);
  const motion = audio.running
    ? orbitMotionAt(
        section,
        visualTick,
        audio.sectionVisualTick(section.id, visualTick),
        score.ticksPerBeat,
        score.beatsPerBar,
      )
    : {
        beatPulse: 0,
        innerTurns: 0,
        melodyPulse: 0,
        outerTurns: 0,
        playheadTurns: 0,
        rhythmPulse: 0,
      };
  for (const [property, value] of Object.entries(orbitStyleAt(motion))) {
    elements.orbit.style.setProperty(property, value);
  }
  renderSections(score, transport, audio, requested, busy);
}
