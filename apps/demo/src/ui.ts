import { racingGenreExperiments } from "./genre-catalog.ts";
import type {
  AdaptiveTransport,
  PortableScore,
  PortableSection,
  SectionId,
  TransitionPlan,
  TransportSnapshot,
} from "../../../packages/runtime/src/index.ts";
import type { DemoAudioEngine, SoloMode } from "./audio-engine.ts";
import type { Arrangement } from "./wasm-engine.ts";
import { requireElement, elements } from "./dom";
import { orbitStyleAt, orbitFrameAt, type OrbitFrame } from "./orbit-visualizer.ts";
import { cueView } from "./section-cues.ts";
import {
  ADVENTURE_SCENE_SECTIONS,
  SUSPENSE_PHASE_SECTIONS,
} from "./playback-section.ts";
import { phaseName } from "./phase-names.ts";
import { APPLY_PIECE, NEW_PIECE, NEW_VERSION, PIECE_AXIS, versionLabel } from "./lab-copy.ts";
import { LAB_RECIPES, labRecipeInfo, nextVersionNumber, type LabRecipe, type NormalizedMusicTraits } from "./state.ts";

const PART_COLORS = ["#d7ff3f", "#6be3ff", "#ffb347", "#ff8ad8", "#f1eee5", "#b9a7ff"] as const;

const RACING_PHASES = ["garage", "grid", "race", "finish"] as const;

function getPartRings(): HTMLElement[] {
  // Re-query each frame: N=6 is trivial; survives DOM clones in tests (e.g. firefox compat)
  return Array.from(
    document.querySelectorAll<HTMLElement>("#orbit [data-orbit-part]"),
  );
}

function fmt(value: number): string {
  return String(Math.round(value * 10000) / 10000);
}

export type ViewName = "lab" | "games" | "genres";
export { requireElement, elements };

interface ArrangementOption {
  id: Arrangement;
  label: string;
  description: string;
  summary: string;
}

const RACING_ARRANGEMENTS: readonly ArrangementOption[] = [
  {
    id: "original",
    label: "Original",
    description: "Six original phases",
    summary: "The original musical parts, unchanged.",
  },
  {
    id: "extended",
    label: "Extended",
    description: "Ten-section race · ignition → cooldown",
    summary: "Ten-section race: ignition, slipstream, redline, and cooldown extend the tour.",
  },
  {
    id: "composed",
    label: "Composed",
    description: "Six phases composed into a song form",
    summary: "A seeded composer orders the six phases into a song form, re-times them by role, and loops the tour back to cruise.",
  },
];

const SUSPENSE_ARRANGEMENTS: readonly ArrangementOption[] = [
  {
    id: "all-phases",
    label: "All phases",
    description: "Whole song · canonical order",
    summary: "Every pool phase once, intro to coda, looping back to the first groove.",
  },
  {
    id: "seeded",
    label: "Seeded",
    description: "Composed from the phase pool",
    summary: "A composer picks the count, roles, order, and loop point from the seed; step the seed reel to hear the variety.",
  },
];

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
  elements.holdForm.textContent = transport.formHeld ? "Resume auto tour" : "Hold auto tour";
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
  const suspense = recipe === "suspense";
  const adventure = recipe === "adventure";
  elements.shell.dataset.recipe = recipe;
  elements.gameSignals.dataset.recipe = recipe;
  renderRecipeSelect(recipe);
  elements.traitEnergyLabel.textContent = suspense ? "Tension" : adventure ? "Danger" : "Energy";
  elements.traitComplexityLabel.textContent = suspense ? "Heat" : adventure ? "Mystery" : "Complexity";
  elements.traitBrightnessLabel.textContent = suspense ? "Mystery" : adventure ? "Wonder" : "Brightness";
  elements.traitSyncopationLabel.textContent = suspense ? "Pulse" : adventure ? "Motion" : "Syncopation";
  elements.meterIntensityLabel.textContent = suspense ? "Detection heat" : adventure ? "Discovery" : "Speed intensity";
  elements.meterPressureLabel.textContent = suspense ? "Focus" : adventure ? "Threat" : "Position pressure";
  elements.meterFinalLabel.textContent = suspense ? "Extracted" : adventure ? "Quest complete" : "Final lap";
  elements.meterFinalCopy.textContent = suspense
    ? "Hold the coda / disconnect"
    : adventure
      ? "Mark the quest complete"
      : "Add the maximum-commitment layer";
  const phaseSections = suspense
    ? SUSPENSE_PHASE_SECTIONS
    : adventure
      ? ADVENTURE_SCENE_SECTIONS
      : null;
  const phaseIds: readonly string[] = phaseSections === null
    ? RACING_PHASES
    : Object.keys(phaseSections);
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

export function renderArrangementControl(recipe: LabRecipe, arrangement: Arrangement): void {
  const visible = recipe !== "adventure";
  elements.arrangementControl.hidden = !visible;
  if (!visible) {
    elements.arrangementSummary.value = "";
    return;
  }
  const options = recipe === "suspense" ? SUSPENSE_ARRANGEMENTS : RACING_ARRANGEMENTS;
  elements.arrangementButtons.replaceChildren(
    ...options.map((option) => {
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
    options.find((option) => option.id === arrangement)?.summary ?? "";
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
    `${score.sections.length} sections @ ${score.bpm} bpm`,
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

/** The version readout: `level-001 · Versión 3`. */
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
  snapshot: TransportSnapshot,
  busy: boolean,
): void {
  const sections = score.sections;
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
    button.disabled = !enabled;
    const title = `${label} section: ${target.label}`;
    if (button.title !== title) {
      button.title = title;
      button.setAttribute("aria-label", title);
    }
  }
}

export function renderEngineButton(
  button: HTMLButtonElement,
  running: boolean,
  tick: number,
  activeTransition: TransitionPlan | null,
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

  button.setAttribute("data-engine-state", state);
  button.setAttribute("aria-label", ariaLabel);
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
        fromEl.setAttribute("title", plain);
      }
    } else {
      if (fromEl.textContent !== plain) {
        fromEl.textContent = plain;
      }
      fromEl.setAttribute("title", plain.length > 12 ? plain : "");
    }
  }
  if (toEl) {
    if (toEl.textContent !== toText) {
      toEl.textContent = toText;
    }
    toEl.setAttribute("title", toText.length > 12 ? toText : "");
  }
  if (stopEl) {
    stopEl.style.display = state === "offline" ? "none" : "";
  }
  if (progEl) {
    if (state === "crossing") {
      progEl.style.setProperty("--cross-progress", crossProgress.toFixed(4));
      progEl.style.setProperty("--from-color", fromColor);
      progEl.style.setProperty("--to-color", toColor);
    } else {
      progEl.style.removeProperty("--cross-progress");
      progEl.style.removeProperty("--from-color");
      progEl.style.removeProperty("--to-color");
    }
  }
  // also expose on button for gradient targeting
  if (state === "crossing") {
    button.style.setProperty("--cross-progress", crossProgress.toFixed(4));
    button.style.setProperty("--from-color", fromColor);
    button.style.setProperty("--to-color", toColor);
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
  const frame: OrbitFrame = audio.running
    ? orbitFrameAt(
        section,
        visualTick,
        audio.sectionVisualTick(section.id, visualTick),
        score.ticksPerBeat,
        score.beatsPerBar,
      )
    : { beatPulse: 0, playheadTurns: 0, glowPulse: 0, parts: [] };
  for (const [property, value] of Object.entries(orbitStyleAt(frame))) {
    elements.orbit.style.setProperty(property, value);
  }
  const rings = getPartRings();
  const n = frame.parts.length;
  for (let i = 0; i < rings.length; i++) {
    const el = rings[i]!;
    const part = i < n ? frame.parts[i] : undefined;
    const shouldHide = !part;
    if (shouldHide) {
      if (!el.hasAttribute("hidden")) el.setAttribute("hidden", "");
      continue;
    }
    if (el.hasAttribute("hidden")) el.removeAttribute("hidden");

    if (el.getAttribute("data-part") !== part.id) {
      el.setAttribute("data-part", part.id);
    }
    const title = `${part.label} · ${part.instrument}`;
    if (el.title !== title) {
      el.title = title;
      el.setAttribute("aria-label", title);
    }

    const color = PART_COLORS[part.colorIndex % PART_COLORS.length]!;
    if (el.style.getPropertyValue("--part-color") !== color) {
      el.style.setProperty("--part-color", color);
    }

    // spread ~4% to ~34% for up to 6 rings
    const insetPct = 4 + i * 6;
    const inset = `${insetPct}%`;
    if (el.style.getPropertyValue("--part-inset") !== inset) {
      el.style.setProperty("--part-inset", inset);
    }

    const p = part.pulse;
    const opacity = 0.5 + p * 0.45;
    const scale = 1 + p * 0.06;
    const rot = `${fmt(part.turns)}turn`;
    const opStr = fmt(opacity);
    const scStr = fmt(scale);

    if (el.style.getPropertyValue("--part-rotation") !== rot) {
      el.style.setProperty("--part-rotation", rot);
    }
    if (el.style.getPropertyValue("--part-opacity") !== opStr) {
      el.style.setProperty("--part-opacity", opStr);
    }
    if (el.style.getPropertyValue("--part-scale") !== scStr) {
      el.style.setProperty("--part-scale", scStr);
    }
  }
  renderSections(score, transport, audio, requested, busy);

  renderEngineButton(
    elements.start,
    audio.running,
    tick,
    activeTransition,
    section,
    sectionById,
  );
  renderSectionSteps(score, tick, snapshot, busy);
}
