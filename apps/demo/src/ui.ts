import { racingGenreExperiments } from "./genre-catalog.ts";
import type {
  PortableScore,
  PortableSection,
  SectionId,
} from "../../../packages/runtime/src/index.ts";
import type { SoloMode } from "./playback.ts";
import {
  soundingSection,
  type PlaybackFrame,
  type PlaybackSnapshot,
  type PlaybackTransition,
} from "./playback.ts";
import type { Arrangement } from "./wasm-engine.ts";
import { requireElement, elements } from "./dom";
import { orbitStyleAt, orbitFrameAt, type OrbitFrame } from "./orbit-visualizer.ts";
import { cueView } from "./section-cues.ts";
import { isDebugBarSection } from "./playback-section.ts";
import { phaseName } from "./phase-names.ts";
import { APPLY_PIECE, NEW_PIECE, NEW_VERSION, PIECE_AXIS, versionLabel } from "./lab-copy.ts";
import { labRecipeInfo, nextVersionNumber } from "./state.ts";
import { LAB_RECIPES, type LabRecipe, type NormalizedMusicTraits } from "./recipes.ts";

const PART_COLORS = ["#d7ff3f", "#6be3ff", "#ffb347", "#ff8ad8", "#f1eee5", "#b9a7ff"] as const;

function getPartRings(): HTMLElement[] {
  // Re-query each frame: N=6 is trivial; survives DOM clones in tests (e.g. firefox compat)
  return Array.from(
    document.querySelectorAll<HTMLElement>("#orbit [data-orbit-part]"),
  );
}

function fmt(value: number): string {
  return String(Math.round(value * 10000) / 10000);
}

// renderFrame runs on every animation frame, so its DOM writes go through these:
// an unchanged value is never rewritten, and a still frame costs no style,
// layout or paint work.
function setText(element: HTMLElement, text: string): void {
  if (element.textContent !== text) element.textContent = text;
}

function setAttribute(element: Element, name: string, value: string): void {
  if (element.getAttribute(name) !== value) element.setAttribute(name, value);
}

function setStyleProperty(element: HTMLElement, property: string, value: string): void {
  if (element.style.getPropertyValue(property) !== value) element.style.setProperty(property, value);
}

function setDisabled(control: HTMLButtonElement | HTMLInputElement | HTMLSelectElement, disabled: boolean): void {
  if (control.disabled !== disabled) control.disabled = disabled;
}

/** The score's real phases: the debugger's transient bar slices are not phases. */
function phaseSections(score: PortableScore): readonly PortableSection[] {
  return score.sections.filter((section) => !isDebugBarSection(section.id));
}

export type ViewName = "lab" | "games" | "genres" | "debugger";
export { requireElement, elements };

interface ArrangementOption {
  id: Arrangement;
  label: string;
  description: string;
  summary: string;
}

const ARRANGEMENTS: readonly ArrangementOption[] = [
  {
    id: "all-phases",
    label: "All phases",
    description: "Whole song · canonical order",
    summary: "Every phase of the recipe once, in canonical order, looping back to the first groove.",
  },
  {
    id: "seeded",
    label: "Seeded",
    description: "Composed song form",
    summary: "Play the composed song form.",
  },
];

const sectionRows = new Map<
  string,
  { item: HTMLLIElement; output: HTMLOutputElement; button: HTMLButtonElement }
>();

export function createSectionRows(score: PortableScore): void {
  sectionRows.clear();
  const phases = phaseSections(score);
  const rows = phases.map((section) => {
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
  elements.sectionSelect.replaceChildren(...phases.map((section) => {
    const option = document.createElement("option");
    option.value = section.id;
    option.textContent = `${section.label} · ${section.lengthTicks / (score.beatsPerBar * score.ticksPerBeat)} bars`;
    return option;
  }));
}

export function renderSections(frame: PlaybackFrame, running: boolean, busy: boolean): void {
  const { score } = frame;
  const tick = Math.floor(frame.tick);
  const mix = new Map(frame.mix.map((item) => [item.section, item.gain]));
  const view = cueView(score, frame.snapshot, tick, running, frame.requestedCue, frame.formHeld, frame.pendingScore);
  const next = frame.nextFormSection;
  setDisabled(elements.holdForm, busy || score.form === undefined);
  setAttribute(elements.holdForm, "aria-pressed", String(frame.formHeld));
  setText(elements.holdForm, frame.formHeld ? "Resume auto tour" : "Hold auto tour");
  setDisabled(elements.advanceForm, busy || next === null || frame.snapshot.transition !== null);
  setText(elements.advanceForm, next === null ? "Next section" : `Next: ${score.sections.find((section) => section.id === next)?.label ?? next}`);
  const status = busy ? "Preparing playback…" : view.status;
  const detail = busy ? "Please wait before cueing a section." : view.detail;
  setText(elements.cueStatus, status);
  setText(elements.cueDetail, detail);
  setDisabled(elements.cancelCue, busy || !view.cancellable);
  setDisabled(elements.sectionSelect, busy);
  for (const control of [elements.intensity, elements.pressure, elements.finalLap, ...elements.phaseButtons.querySelectorAll<HTMLButtonElement>("button")]) setDisabled(control, busy);
  if (document.activeElement !== elements.sectionSelect && elements.sectionSelect.value !== view.select) {
    elements.sectionSelect.value = view.select;
  }
  for (const section of score.sections) {
    const row = sectionRows.get(section.id);
    if (row === undefined) {
      continue;
    }
    const gain = mix.get(section.id) ?? 0;
    setStyleProperty(row.item, "--gain-width", `${gain * 100}%`);
    row.item.classList.toggle("is-audible", gain > 0.001);
    const queued = section.id === view.target;
    const current = section.id === view.current;
    row.item.classList.toggle("is-cued", queued);
    if (current) setAttribute(row.item, "aria-current", "true");
    else row.item.removeAttribute("aria-current");
    setText(row.button, queued ? view.cancellable ? "Queued" : "Next" : current ? running ? "Playing" : "Selected" : "Cue");
    setDisabled(row.button, busy || (current && !view.cancellable));
    setText(row.output, `${Math.round(gain * 100)}%`);
  }
}

export function renderScoreButtons(
  activeExperimentIndex: number,
  presets: readonly { style: string; label?: string }[],
  recipe: LabRecipe,
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
      recipe === "racing"
        ? (racingLabels[index] ?? preset.style)
        : preset.label ?? preset.style.charAt(0).toUpperCase() + preset.style.slice(1);
    genre.textContent = preset.style;
    button.append(label, genre);
    return button;
  });
  elements.scoreButtons.replaceChildren(...buttons);
}

export function renderRecipeSelect(recipe: LabRecipe): void {
  const active = labRecipeInfo(recipe);
  elements.recipeSelectLabel.textContent = active.label;
  elements.recipeSelectDescription.textContent = active.description;
  elements.recipeSelect.dataset.active = recipe;
  const options = LAB_RECIPES.map((entry) => {
    const button = document.createElement("button");
    button.type = "button";
    button.setAttribute("role", "option");
    button.dataset.recipe = entry.id;
    button.setAttribute("aria-selected", String(entry.id === recipe));
    const strong = document.createElement("strong");
    strong.textContent = entry.label;
    const small = document.createElement("small");
    small.textContent = entry.description;
    button.append(strong, small);
    return button;
  });
  elements.recipeSelectMenu.replaceChildren(...options);
}

export function renderRecipeChrome(recipe: LabRecipe, phase: string): void {
  const profile = labRecipeInfo(recipe);
  elements.shell.dataset.recipe = recipe;
  elements.gameSignals.dataset.recipe = recipe;
  elements.gameSignals.hidden = profile.engineUpdate === undefined;
  renderRecipeSelect(recipe);
  elements.traitEnergyLabel.textContent = profile.traitLabels.energy;
  elements.traitComplexityLabel.textContent = profile.traitLabels.complexity;
  elements.traitBrightnessLabel.textContent = profile.traitLabels.brightness;
  elements.traitSyncopationLabel.textContent = profile.traitLabels.syncopation;
  if (profile.meters !== undefined) {
    elements.meterIntensityLabel.textContent = profile.meters.intensity;
    elements.meterPressureLabel.textContent = profile.meters.pressure;
    elements.meterFinalLabel.textContent = profile.meters.flag;
    elements.meterFinalCopy.textContent = profile.meters.flagCopy;
  }
  const phaseIds = profile.phases;
  const buttons = phaseIds.map((id) => {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.phase = id;
    button.textContent = phaseName(id);
    button.setAttribute("aria-pressed", String(id === phase));
    return button;
  });
  elements.phaseButtons.replaceChildren(...buttons);
}

export function renderGenreIndex(): void {
  const rows = racingGenreExperiments.map((experiment, index) => {
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
    button.textContent = `Open ${experiment.scoreTitle}`;
    item.append(number, copy, button);
    return item;
  });
  elements.genreIndex.replaceChildren(...rows);
}

export function renderArrangementControl(_recipe: LabRecipe, arrangement: Arrangement): void {
  elements.arrangementControl.hidden = false;
  elements.arrangementButtons.replaceChildren(
    ...ARRANGEMENTS.map((option) => {
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.arrangement = option.id;
      button.setAttribute("aria-pressed", String(option.id === arrangement));
      const strong = document.createElement("strong");
      strong.textContent = option.label;
      const span = document.createElement("span");
      span.textContent = option.description;
      button.append(strong, span);
      return button;
    }),
  );
  elements.arrangementSummary.value =
    ARRANGEMENTS.find((option) => option.id === arrangement)?.summary ?? "";
}

export function renderScoreIdentity(
  activeExperimentIndex: number,
  score: PortableScore,
  levelSeed: string,
  generationTraits: NormalizedMusicTraits,
  soloMode: SoloMode,
  recipe: LabRecipe,
  presets: readonly { style: string; label?: string }[],
  arrangement: Arrangement,
  phase: string,
): void {
  elements.scoreTitle.textContent = score.title;
  elements.tempo.textContent = String(score.bpm);
  renderRecipeChrome(recipe, phase);
  elements.sectionControl.hidden = score.form === undefined;
  renderArrangementControl(recipe, arrangement);
  renderGenerationControls(score, levelSeed, generationTraits);
  document.title = `Gamestruments Audio Lab — ${score.title}`;
  renderScoreButtons(activeExperimentIndex, presets, recipe);
  createSectionRows(score);
  renderAuditionControls(levelSeed, soloMode);
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
  renderAxisCopy();
  elements.levelSeed.value = levelSeed;
  elements.variationValue.value = levelSeed;
  for (const [input, output, value] of traitControls) {
    input.value = String(value);
    output.value = `${Math.round(value * 100)}%`;
  }
  elements.generatorSummary.value = [
    score.id,
    `${phaseSections(score).length} sections @ ${score.bpm} bpm`,
    "engine: wasm",
  ].join(" / ");
}

export function announceAudition(message: string): void {
  elements.auditionStatus.value = message;
}

/** The two axis words and their control labels come from `lab-copy.ts`, so the
 *  Lab never names the same axis two ways. */
function renderAxisCopy(): void {
  elements.pieceLabel.textContent = PIECE_AXIS;
  elements.applySeed.textContent = APPLY_PIECE;
  elements.newPieceLabel.textContent = NEW_PIECE;
  elements.newVersionLabel.textContent = NEW_VERSION;
}

export function renderAuditionControls(levelSeed: string, soloMode: SoloMode): void {
  for (const button of elements.soloButtons.querySelectorAll<HTMLButtonElement>(
    "button[data-solo]",
  )) {
    button.setAttribute("aria-pressed", String(button.dataset.solo === soloMode));
  }
  renderVersion(levelSeed);
}

/** The version readout: `level-001 · Version 3`. */
export function renderVersion(levelSeed: string): void {
  elements.versionValue.value = versionLabel(levelSeed, nextVersionNumber());
}

export function setStartButton(running: boolean): void {
  elements.start.classList.toggle("is-running", running);
  elements.start.setAttribute("aria-pressed", String(running));
  // rich label/state owned by renderFrame / renderEngineButton (stable children)
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

export function renderSectionSteps(
  score: PortableScore,
  tick: number,
  snapshot: PlaybackSnapshot,
  busy: boolean,
): void {
  const sections = phaseSections(score);
  const enabled = sections.length > 1 && !busy;
  const active =
    snapshot.transition !== null && tick >= snapshot.transition.startTick
      ? snapshot.transition.to
      : snapshot.currentSection;
  const index = sections.findIndex((section) => section.id === active);
  const base = index === -1 ? 0 : index;
  const prevTarget = sections[(base - 1 + sections.length) % sections.length];
  const nextTarget = sections[(base + 1) % sections.length];
  if (prevTarget === undefined || nextTarget === undefined) return;
  const targets = [
    { button: elements.prevSection, target: prevTarget, label: "Previous" },
    { button: elements.nextSection, target: nextTarget, label: "Next" },
  ];
  for (const { button, target, label } of targets) {
    setDisabled(button, !enabled);
    const title = `${label} section: ${target.label}`;
    setAttribute(button, "title", title);
    setAttribute(button, "aria-label", title);
  }
}

export function renderEngineButton(
  button: HTMLButtonElement,
  running: boolean,
  tick: number,
  activeTransition: PlaybackTransition | null,
  section: PortableSection,
  sectionById: (id: string) => PortableSection,
): void {
  let state: "offline" | "playing" | "waiting" | "crossing" = "offline";
  let fromText = "Start engine";
  let toText = "";
  let crossProgress = 0;
  let fromColor = "";
  let toColor = "";
  let ariaLabel = "Start engine";

  if (running) {
    const currentLabel = section.label;
    if (activeTransition === null) {
      state = "playing";
      fromText = currentLabel;
      ariaLabel = `Stop engine — playing ${currentLabel}`;
    } else if (tick < activeTransition.startTick) {
      state = "waiting";
      fromText = currentLabel;
      const toSec = sectionById(activeTransition.to);
      toText = `→ ${toSec.label}`;
      ariaLabel = `Stop engine — waiting for ${toSec.label}`;
    } else {
      state = "crossing";
      const fromSec = sectionById(activeTransition.from);
      const toSec = sectionById(activeTransition.to);
      fromText = `${fromSec.label} → ${toSec.label}`;
      toText = "";
      const dur = Math.max(1, activeTransition.endTick - activeTransition.startTick);
      crossProgress = Math.max(0, Math.min(1, (tick - activeTransition.startTick) / dur));
      fromColor = fromSec.color;
      toColor = toSec.color;
      ariaLabel = `Stop engine — crossing from ${fromSec.label} to ${toSec.label}`;
    }
  }

  setAttribute(button, "data-engine-state", state);
  setAttribute(button, "aria-label", ariaLabel);
  // aria-pressed kept by setStartButton

  const fromEl = button.querySelector<HTMLElement>(".engine-label--from");
  const toEl = button.querySelector<HTMLElement>(".engine-label--to");
  const stopEl = button.querySelector<HTMLElement>(".engine-stop");
  const progEl = button.querySelector<HTMLElement>(".engine-progress");

  if (fromEl) {
    const crossing = state === "crossing";
    const fromSec = crossing ? sectionById(activeTransition!.from) : null;
    const toSec = crossing ? sectionById(activeTransition!.to) : null;
    const plain = fromText;
    if (crossing && fromSec !== null && toSec !== null) {
      if (fromEl.textContent !== plain) {
        const arrow = document.createElement("span");
        arrow.className = "engine-arrow";
        arrow.textContent = "→";
        fromEl.replaceChildren(
          document.createTextNode(`${fromSec.label} `),
          arrow,
          document.createTextNode(` ${toSec.label}`),
        );
        setAttribute(fromEl, "title", plain);
      }
    } else {
      setText(fromEl, plain);
      setAttribute(fromEl, "title", plain.length > 12 ? plain : "");
    }
  }
  if (toEl) {
    setText(toEl, toText);
    setAttribute(toEl, "title", toText.length > 12 ? toText : "");
  }
  if (stopEl) {
    setStyleProperty(stopEl, "display", state === "offline" ? "none" : "");
  }
  if (progEl) {
    if (state === "crossing") {
      setStyleProperty(progEl, "--cross-progress", crossProgress.toFixed(4));
      setStyleProperty(progEl, "--from-color", fromColor);
      setStyleProperty(progEl, "--to-color", toColor);
    } else {
      progEl.style.removeProperty("--cross-progress");
      progEl.style.removeProperty("--from-color");
      progEl.style.removeProperty("--to-color");
    }
  }
  // also expose on button for gradient targeting
  if (state === "crossing") {
    setStyleProperty(button, "--cross-progress", crossProgress.toFixed(4));
    setStyleProperty(button, "--from-color", fromColor);
    setStyleProperty(button, "--to-color", toColor);
  } else {
    button.style.removeProperty("--cross-progress");
    button.style.removeProperty("--from-color");
    button.style.removeProperty("--to-color");
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
  // The score debugger is an authoring tool, not part of the public Lab: it is
  // reachable only with an explicit `?debug` flag, and its nav link is hidden
  // for every other visitor.
  const debugEnabled = new URLSearchParams(window.location.search).has("debug");
  const debugLink = document.querySelector('[data-view-link="debugger"]');
  if (debugLink instanceof HTMLElement) {
    debugLink.hidden = !debugEnabled;
  }
  const view: ViewName =
    requested === "games" || requested === "genres" || (requested === "debugger" && debugEnabled)
      ? requested
      : "lab";
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

export function renderFrame(frame: PlaybackFrame, running: boolean, busy = false): void {
  const { score, snapshot } = frame;
  const sectionById = sectionLookup(score);
  const tick = Math.floor(frame.tick);
  const barTicks = score.beatsPerBar * score.ticksPerBeat;
  const activeTransition = snapshot.transition;
  const section = sectionById(soundingSection(snapshot, tick));
  const pendingLabel = snapshot.pendingSection === null
    ? null
    : sectionLookup(frame.pendingScore ?? score)(snapshot.pendingSection).label;

  setText(elements.bar, String(Math.floor(tick / barTicks) + 1).padStart(2, "0"));
  setText(elements.beat, String(
    Math.floor((tick % barTicks) / score.ticksPerBeat) + 1,
  ).padStart(2, "0"));
  setText(elements.moodName, section.label);
  setText(elements.moodFeeling, section.feeling);
  setText(elements.transitionLabel, !running
    ? "Engine offline"
    : frame.pendingScore !== null
      ? `Next: ${frame.pendingScore.title}${pendingLabel === null ? "" : ` · ${pendingLabel}`}`
    : pendingLabel !== null
      ? `Queued: ${pendingLabel} · after this blend`
      : activeTransition === null
        ? score.form === undefined
          ? "Pattern locked"
           : frame.formHeld ? "Holding section" : "Form playing"
        : tick < activeTransition.startTick
           ? `Waiting for next bar → ${sectionById(activeTransition.to).label}`
          : `Crossing from ${sectionById(activeTransition.from).label}`);
  setStyleProperty(elements.orbit, "--mood-color", section.color);
  elements.orbit.classList.toggle("is-running", running);
  const orbit: OrbitFrame = running
    ? orbitFrameAt(
        section,
        frame.tick,
        frame.sectionTick(section.id),
        score.ticksPerBeat,
        score.beatsPerBar,
      )
    : { beatPulse: 0, playheadTurns: 0, glowPulse: 0, parts: [] };
  for (const [property, value] of Object.entries(orbitStyleAt(orbit))) {
    setStyleProperty(elements.orbit, property, value);
  }
  const rings = getPartRings();
  const n = orbit.parts.length;
  for (let i = 0; i < rings.length; i++) {
    const el = rings[i]!;
    const part = i < n ? orbit.parts[i] : undefined;
    const shouldHide = !part;
    if (shouldHide) {
      if (!el.hasAttribute("hidden")) el.setAttribute("hidden", "");
      continue;
    }
    if (el.hasAttribute("hidden")) el.removeAttribute("hidden");

    setAttribute(el, "data-part", part.id);
    const title = `${part.label} · ${part.instrument}`;
    setAttribute(el, "title", title);
    setAttribute(el, "aria-label", title);
    setStyleProperty(el, "--part-color", PART_COLORS[part.colorIndex % PART_COLORS.length]!);
    // spread ~4% to ~34% for up to 6 rings
    setStyleProperty(el, "--part-inset", `${4 + i * 6}%`);

    const p = part.pulse;
    const opacity = 0.5 + p * 0.45;
    const scale = 1 + p * 0.06;
    setStyleProperty(el, "--part-rotation", `${fmt(part.turns)}turn`);
    setStyleProperty(el, "--part-opacity", fmt(opacity));
    setStyleProperty(el, "--part-scale", fmt(scale));
  }
  renderSections(frame, running, busy);

  renderEngineButton(
    elements.start,
    running,
    tick,
    activeTransition,
    section,
    sectionById,
  );
  renderSectionSteps(score, tick, snapshot, busy);
}

/** Look sections up by id in `score`. */
function sectionLookup(score: PortableScore): (id: SectionId) => PortableSection {
  return (id) => {
    const section = score.sections.find((candidate) => candidate.id === id);
    if (section === undefined) throw new Error(`Missing score section: ${id}`);
    return section;
  };
}
