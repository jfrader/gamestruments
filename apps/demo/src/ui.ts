import "./style.css";
import { pocketCircuitExperiments } from "../../../packages/studio/src/index.ts";
import type { SoloMode } from "./audio-engine.ts";
import { requireElement, elements } from "./dom";

export type ViewName = "lab" | "games" | "genres";
export { requireElement, elements };

const sectionRows = new Map<
  string,
  { item: HTMLLIElement; output: HTMLOutputElement }
>();

export function createSectionRows(score: any): void {
  sectionRows.clear();
  const rows = score.sections.map((section: any) => {
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

export function renderSections(score: any, transport: any, audio: any): void {
  const tick = audio.currentTick();
  const mix = new Map(
    transport.mixAt(tick).map((item: any) => [item.section, item.gain]),
  );
  for (const section of score.sections) {
    const row = sectionRows.get(section.id);
    if (row === undefined) {
      continue;
    }
    const gain: number = (mix.get(section.id) ?? 0) as number;
    row.item.style.setProperty("--gain", String(gain));
    row.item.classList.toggle("is-audible", (gain as number) > 0.001);
    row.output.value = `${Math.round((gain as number) * 100)}%`;
  }
}

export function renderScoreButtons(activeExperimentIndex: number): void {
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
  score: any,
  levelSeed: string,
  generationTraits: any,
  renderGenerationControlsFn: Function,
  renderAuditionControlsFn: Function,
  requestMusicStateFn: Function
): void {
  const experiment = pocketCircuitExperiments[activeExperimentIndex];
  if (experiment === undefined) {
    return;
  }
  elements.scoreTitle.textContent = score.title;
  elements.tempo.textContent = String(score.bpm);
  renderGenerationControlsFn(levelSeed, generationTraits);
  document.title = `Gamestruments Audio Lab — ${score.title}`;
  renderScoreButtons(activeExperimentIndex);
  createSectionRows(score);
  renderAuditionControlsFn(levelSeed, generationTraits);
  requestMusicStateFn();
}

export function renderGenerationControls(levelSeed: string, generationTraits: any): void {
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
    (window as any).__labScoreId || 'score',
    `sections @ bpm`,
    `engine: wasm`,
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
  const c = elements.centerPlay;
  if (c) {
    c.classList.toggle("is-running", running);
    c.setAttribute("aria-pressed", String(running));
    const glyph = c.querySelector<HTMLElement>(".center-glyph");
    const label = c.querySelector<HTMLElement>(".center-label");
    if (glyph) glyph.textContent = running ? "❚❚" : "▶";
    if (label) label.textContent = running ? "PAUSE" : "PLAY";
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
  audio: any,
  score: any,
  transport: any,
  auditionOverride: any,
  applyPlanFn: Function,
  sectionByIdFn: Function,
  renderSectionsFn: Function
): void {
  const tick = audio.currentTick();
  const barTicks = score.beatsPerBar * score.ticksPerBeat;
  const nextPlan = transport.advance(tick);
  if (nextPlan !== null) {
    applyPlanFn(nextPlan);
  }
  const snapshot = transport.snapshot();
  const activeTransition = snapshot.transition;
  const primarySection =
    activeTransition !== null && tick >= activeTransition.startTick
      ? activeTransition.to
      : snapshot.currentSection;
  const section = sectionByIdFn(primarySection);

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
          : `Crossing from ${sectionByIdFn(activeTransition.from).label}`;
  elements.orbit.style.setProperty("--mood-color", section.color);
  elements.orbit.style.setProperty(
    "--beat-progress",
    String((tick % score.ticksPerBeat) / score.ticksPerBeat),
  );
  renderSectionsFn(score, transport, audio);
  window.requestAnimationFrame(() => renderFrame(audio, score, transport, auditionOverride, applyPlanFn, sectionByIdFn, renderSectionsFn));
}
