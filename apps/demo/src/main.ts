import "./style.css";
import type { SectionId } from "../../../packages/runtime/src/index.ts";
import type { SoloMode } from "./audio-engine.ts";
import {
  elements,
  setStartButton,
  setPlaybackPending,
  announceAudition,
  renderGenreIndex,
  renderScoreIdentity,
  renderAuditionControls,
  renderView,
  renderFrame,
} from "./ui";
import {
  activeExperimentIndex,
  auditionOverride,
  audio,
  comparisonBaseSeed,
  generationTraits,
  generationPreset,
  initializeLab,
  levelSeed,
  phase,
  score,
  sectionById,
  soloMode,
  transport,
  applyPlan,
  requestMusicState,
  jumpToSection,
  toggleEngine,
  requestExperiment,
  nextLevelSeed,
  traitsFromControls,
  setComparisonBaseSeed,
  setPhase,
  setSoloMode,
} from "./state";

await initializeLab();

function renderCurrentScore(): void {
  renderScoreIdentity(
    activeExperimentIndex,
    score,
    levelSeed,
    generationTraits,
    comparisonBaseSeed,
    soloMode,
  );
  requestMusicState();
}

async function togglePlayback(): Promise<void> {
  setPlaybackPending(true);
  try {
    setStartButton(await toggleEngine());
  } finally {
    setPlaybackPending(false);
  }
}

function applyGenerationRequest(request: Promise<boolean>, onApplied: () => void): void {
  void request
    .then((applied) => {
      if (applied) {
        onApplied();
      }
    })
    .catch((error: unknown) => {
      const detail = error instanceof Error ? error.message : "Unknown generation error";
      announceAudition(`Generation failed: ${detail}`);
    });
}

function animate(): void {
  renderFrame(audio, score, transport, auditionOverride, applyPlan, sectionById);
  window.requestAnimationFrame(animate);
}

function renderRuntimeSignal(): void {
  elements.runtimeSignal.textContent = `racePhase: ${phase}`;
}

renderGenreIndex();
renderCurrentScore();
renderView();
setStartButton(false);
window.requestAnimationFrame(animate);

elements.start.addEventListener("click", () => {
  void togglePlayback();
});
elements.centerPlay.addEventListener("click", () => {
  void togglePlayback();
});
document.addEventListener("keydown", (ev) => {
  if (ev.key !== " " && ev.key !== "Spacebar") {
    return;
  }
  const target = ev.target as HTMLElement | null;
  if (
    target !== null &&
    (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)
  ) {
    return;
  }
  ev.preventDefault();
  void togglePlayback();
});

elements.scoreButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
    "button[data-experiment-index]",
  );
  if (button === null) {
    return;
  }
  const index = Number(button.dataset.experimentIndex);
  applyGenerationRequest(requestExperiment(index, levelSeed, {
    ...generationPreset(index).traits,
  }), () => {
    setComparisonBaseSeed(levelSeed);
    renderCurrentScore();
    announceAudition(`Generated ${score.title}`);
  });
});

elements.newTake.addEventListener("click", () => {
  const nextSeed = nextLevelSeed(comparisonBaseSeed);
  applyGenerationRequest(requestExperiment(activeExperimentIndex, nextSeed), () => {
    setComparisonBaseSeed(levelSeed);
    renderCurrentScore();
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
  applyGenerationRequest(requestExperiment(activeExperimentIndex, requestedSeed), () => {
    setComparisonBaseSeed(levelSeed);
    renderCurrentScore();
    announceAudition(`Generated level ${levelSeed}`);
  });
});
elements.levelSeed.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    elements.applySeed.click();
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
    applyGenerationRequest(requestExperiment(activeExperimentIndex, levelSeed, traitsFromControls()), () => {
      setComparisonBaseSeed(levelSeed);
      renderCurrentScore();
      announceAudition(`Regenerated ${score.title}`);
    });
  });
}

elements.soloButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
    "button[data-solo]",
  );
  const requested = button?.dataset.solo as SoloMode | undefined;
  if (requested !== "full" && requested !== "melody" && requested !== "rhythm") {
    return;
  }
  setSoloMode(requested);
  renderAuditionControls(levelSeed, comparisonBaseSeed, soloMode);
  announceAudition(soloMode === "full" ? "Full mix on" : `${soloMode} solo on`);
});

elements.compareTake.addEventListener("click", () => {
  const nextSeed =
    levelSeed === comparisonBaseSeed ? `${comparisonBaseSeed}:B` : comparisonBaseSeed;
  applyGenerationRequest(requestExperiment(activeExperimentIndex, nextSeed), () => {
    renderCurrentScore();
    const side = levelSeed === comparisonBaseSeed ? "A" : "B";
    announceAudition(`Playing seed ${side}: ${levelSeed}`);
  });
});

elements.sectionList.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
    "button[data-jump-section]",
  );
  if (button?.dataset.jumpSection !== undefined) {
    jumpToSection(button.dataset.jumpSection as SectionId);
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
  applyGenerationRequest(requestExperiment(index, levelSeed, {
    ...generationPreset(index).traits,
  }), () => {
    setComparisonBaseSeed(levelSeed);
    renderCurrentScore();
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
  setPhase(button.dataset.phase ?? "garage");
  for (const candidate of elements.phaseButtons.querySelectorAll("button")) {
    candidate.setAttribute("aria-pressed", String(candidate === button));
  }
  renderRuntimeSignal();
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
