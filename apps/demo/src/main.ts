import "./style.css";
import type {
  TransitionPlan,
  SectionId,
  PortableScore,
} from "../../../packages/runtime/src/index.ts";
import type { NormalizedMusicTraits } from "../../../packages/studio/src/index.ts";
import { DemoAudioEngine, type SoloMode } from "./audio-engine.ts";

// state (kept in main for mutable binding; logic split to modules)
let activeExperimentIndex = 0;
let levelSeed = "level-001";
let generationTraits: NormalizedMusicTraits = { energy: 0.62, complexity: 0.68, brightness: 0.52, syncopation: 0.72 };
let phase = "garage";
let score!: PortableScore;
let transport!: any;
let audio!: DemoAudioEngine;
let soloMode: SoloMode = "full";
let comparisonBaseSeed = levelSeed;
let auditionOverride: SectionId | null = null;

// Focused modules (bounded split for maintainability)
import {
  elements,
  setStartButton,
  announceAudition,
  renderGenreIndex,
  renderScoreIdentity,
  renderAuditionControls,
  renderView,
  renderFrame,
  renderSections as uiRenderSections,
} from "./ui";
import {
  generationPreset,
  generateCurrentScore,
  sectionById,
  applyPlan,
  requestMusicState,
  jumpToSection,
  toggleEngine,
  requestExperiment,
  nextLevelSeed,
  traitsFromControls,
} from "./state";

// Load initial score from the shared WASM engine (top-level await kept in main.ts as bootstrap).
// (state module provides the generator; assignment uses live export lets)
const _initialScore = await generateCurrentScore();
(score as any) = _initialScore;
(transport as any) = new (await import("../../../packages/runtime/src/index.ts")).AdaptiveTransport(score);
(audio as any) = new DemoAudioEngine(score);

// Wire initial from modules (ui holds elements + pure renders)
renderGenreIndex();
renderScoreIdentity(activeExperimentIndex, score, levelSeed, generationTraits, () => {}, (ls: string, cb: string) => renderAuditionControls(ls, cb, soloMode), () => requestMusicState());
renderView();
setStartButton(false);
window.requestAnimationFrame(() => renderFrame(audio, score, transport, auditionOverride, (p: TransitionPlan) => { applyPlan(p); (elements as any).orbit.style.setProperty("--mood-color", sectionById(p.to).color); }, sectionById, (s: any, t: any, a: any) => uiRenderSections(s, t, a)));

// --- listeners (wiring stays here; handlers delegate to modules) ---
elements.start.addEventListener("click", () => { void toggleEngine(); });
if (elements.centerPlay) {
  elements.centerPlay.addEventListener("click", () => { void toggleEngine(); });
}
document.addEventListener("keydown", (ev) => {
  if (ev.key !== " " && ev.key !== "Spacebar") return;
  const target = ev.target as HTMLElement | null;
  if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) return;
  ev.preventDefault();
  void toggleEngine();
});

elements.scoreButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-experiment-index]");
  if (button === null) return;
  const index = Number(button.dataset.experimentIndex);
  void requestExperiment(index, levelSeed, { ...generationPreset(index).traits }).then((applied) => {
    if (!applied) return;
    comparisonBaseSeed = levelSeed;
    renderAuditionControls(levelSeed, comparisonBaseSeed, soloMode);
    announceAudition(`Generated ${score.title}`);
  });
});

elements.newTake.addEventListener("click", () => {
  const nextSeed = nextLevelSeed(comparisonBaseSeed);
  void requestExperiment(activeExperimentIndex, nextSeed).then((applied) => {
    if (!applied) return;
    comparisonBaseSeed = levelSeed;
    renderAuditionControls(levelSeed, comparisonBaseSeed, soloMode);
    announceAudition(`Generated level ${levelSeed}`);
  });
});

elements.applySeed.addEventListener("click", () => {
  const requestedSeed = elements.levelSeed.value.trim();
  if (requestedSeed.length === 0) {
    elements.levelSeed.value = levelSeed;
    announceAudition("Level seed cannot be empty");
    return;
  }
  void requestExperiment(activeExperimentIndex, requestedSeed).then((applied) => {
    if (!applied) return;
    comparisonBaseSeed = levelSeed;
    renderAuditionControls(levelSeed, comparisonBaseSeed, soloMode);
    announceAudition(`Generated level ${levelSeed}`);
  });
});
elements.levelSeed.addEventListener("keydown", (event) => {
  if (event.key === "Enter") { event.preventDefault(); /* apply via click above */ }
});

for (const [input, output] of [
  [elements.generationEnergy, elements.generationEnergyValue],
  [elements.generationComplexity, elements.generationComplexityValue],
  [elements.generationBrightness, elements.generationBrightnessValue],
  [elements.generationSyncopation, elements.generationSyncopationValue],
] as const) {
  input.addEventListener("input", () => { output.value = `${Math.round(Number(input.value) * 100)}%`; });
  input.addEventListener("change", () => {
    void requestExperiment(activeExperimentIndex, levelSeed, traitsFromControls()).then((applied) => {
      if (!applied) return;
      comparisonBaseSeed = levelSeed;
      renderAuditionControls(levelSeed, comparisonBaseSeed, soloMode);
      announceAudition(`Regenerated ${score.title}`);
    });
  });
}

elements.soloButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-solo]");
  const requested = button?.dataset.solo as SoloMode | undefined;
  if (requested !== "full" && requested !== "melody" && requested !== "rhythm") return;
  (soloMode as any) = requested;
  audio.soloMode = soloMode;
  renderAuditionControls(levelSeed, comparisonBaseSeed, soloMode);
  // runtime signal via ui if needed
  announceAudition(soloMode === "full" ? "Full mix on" : `${soloMode} solo on`);
});

elements.compareTake.addEventListener("click", () => {
  const nextSeed = levelSeed === comparisonBaseSeed ? `${comparisonBaseSeed}:B` : comparisonBaseSeed;
  void requestExperiment(activeExperimentIndex, nextSeed).then((applied) => {
    if (!applied) return;
    renderAuditionControls(levelSeed, comparisonBaseSeed, soloMode);
    const side = levelSeed === comparisonBaseSeed ? "A" : "B";
    announceAudition(`Playing seed ${side}: ${levelSeed}`);
  });
});

elements.sectionList.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-jump-section]");
  if (button?.dataset.jumpSection) jumpToSection(button.dataset.jumpSection as SectionId);
});

elements.genreIndex.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-experiment-index]");
  if (button === null) return;
  const index = Number(button.dataset.experimentIndex);
  void requestExperiment(index, levelSeed, { ...generationPreset(index).traits }).then((applied) => {
    if (!applied) return;
    comparisonBaseSeed = levelSeed;
    renderAuditionControls(levelSeed, comparisonBaseSeed, soloMode);
    window.location.hash = "lab";
    announceAudition(`Generated ${score.title}`);
  });
});

for (const button of document.querySelectorAll<HTMLButtonElement>("[data-open-lab]")) {
  button.addEventListener("click", () => { window.location.hash = "lab"; });
}

elements.phaseButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-phase]");
  if (button === null) return;
  (phase as any) = button.dataset.phase ?? "garage";
  for (const c of elements.phaseButtons.querySelectorAll("button")) c.setAttribute("aria-pressed", String(c === button));
  requestMusicState(true);
});

elements.intensity.addEventListener("input", () => { elements.intensityValue.value = `${Math.round(Number(elements.intensity.value) * 100)}%`; requestMusicState(true); });
elements.pressure.addEventListener("input", () => { elements.pressureValue.value = `${Math.round(Number(elements.pressure.value) * 100)}%`; requestMusicState(true); });
elements.finalLap.addEventListener("change", () => requestMusicState(true));

window.addEventListener("hashchange", renderView);

// final bootstrap (kept in main)
renderScoreIdentity(activeExperimentIndex, score, levelSeed, generationTraits, () => {}, (ls: string, cb: string) => renderAuditionControls(ls, cb, soloMode), () => requestMusicState());
setStartButton(false);
window.requestAnimationFrame(() => renderFrame(audio, score, transport, auditionOverride, (p: any) => { applyPlan(p); (elements as any).orbit.style.setProperty("--mood-color", sectionById(p.to).color); }, sectionById, (s:any,t:any,a:any) => uiRenderSections(s,t,a) ));
