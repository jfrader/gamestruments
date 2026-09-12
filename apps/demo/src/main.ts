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
  pendingCue,
  audio,
  comparisonBaseSeed,
  generationTraits,
  generationPreset,
  initializeLab,
  levelSeed,
  labRecipe,
  suspenseArrangement,
  setSuspenseArrangement,
  currentPresets,
  phase,
  score,
  sectionById,
  soloMode,
  transport,
  applyPlan,
  requestMusicState,
  requestSuspensePhase,
  requestAdventureScene,
  cueSection,
  stepSection,
  cancelCue,
  cueControlsBusy,
  setFormHold,
  advanceSection,
  toggleEngine,
  requestExperiment,
  nextLevelSeed,
  traitsFromControls,
  setComparisonBaseSeed,
  setPhase,
  setLabRecipe,
  isLabRecipe,
  labRecipeInfo,
  type LabRecipe,
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
    labRecipe,
    currentPresets(),
    suspenseArrangement,
    phase,
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
  renderFrame(audio, score, transport, pendingCue(), applyPlan, sectionById, cueControlsBusy());
  window.requestAnimationFrame(animate);
}

function renderRuntimeSignal(): void {
  elements.runtimeSignal.textContent =
    labRecipe === "suspense"
      ? `recipe: suspense  /  tracePhase: ${phase}`
      : labRecipe === "adventure"
        ? `recipe: adventure  /  areaPhase: ${phase}`
        : `racePhase: ${phase}`;
}

renderGenreIndex();
renderCurrentScore();
renderView();
setStartButton(false);
window.requestAnimationFrame(animate);

// Reserve exactly the fixed controls bar's height on mobile so it never covers
// the masthead at rest, and keep it correct across font/zoom changes.
function syncStickyBarHeight(): void {
  const fixed = window.getComputedStyle(elements.mastheadControls).position === "fixed";
  const height = fixed ? Math.ceil(elements.mastheadControls.getBoundingClientRect().height) : 0;
  document.documentElement.style.setProperty("--sticky-bar-h", `${height}px`);
}
syncStickyBarHeight();
window.addEventListener("resize", syncStickyBarHeight);
window.addEventListener("load", syncStickyBarHeight);

elements.masterVolume.value = String(Math.round(audio.volume * 100));
elements.volumeReadout.textContent = `${elements.masterVolume.value}%`;

elements.masterVolume.addEventListener("input", () => {
  const vol = parseInt(elements.masterVolume.value, 10) / 100;
  audio.volume = vol;
  elements.volumeReadout.textContent = `${elements.masterVolume.value}%`;
});

elements.start.addEventListener("click", () => {
  void togglePlayback();
});
elements.prevSection.addEventListener("click", () => {
  stepSection(-1);
});
elements.nextSection.addEventListener("click", () => {
  stepSection(1);
});
elements.centerPlay.addEventListener("click", () => {
  void togglePlayback();
});
document.addEventListener("keydown", (ev) => {
  if (ev.repeat) return;
  if (ev.key !== " " && ev.key !== "Spacebar") {
    return;
  }
  const target = ev.target as HTMLElement | null;
  if (
    target !== null &&
    (target.closest("input, textarea, select, button, summary") !== null || target.matches("[data-scroll-panel]") || target.isContentEditable)
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
    "button[data-cue-section]",
  );
  if (button?.dataset.cueSection !== undefined) {
    cueSection(button.dataset.cueSection as SectionId);
  }
});

elements.sectionSelect.addEventListener("change", () => cueSection(elements.sectionSelect.value));
elements.cancelCue.addEventListener("click", cancelCue);
elements.holdForm.addEventListener("click", () => setFormHold(!transport.formHeld));
elements.advanceForm.addEventListener("click", advanceSection);

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

elements.arrangementButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-arrangement]");
  const value = button?.dataset.arrangement;
  if (value !== "original" && value !== "extended") {
    return;
  }
  applyGenerationRequest(setSuspenseArrangement(value), () => {
    renderCurrentScore();
    const messages = {
      original: "Original arrangement restored",
      extended: "Extended arrangement ready",
    };
    announceAudition(messages[value]);
  });
});

function setRecipeMenuOpen(open: boolean): void {
  elements.recipeSelectMenu.hidden = !open;
  elements.recipeSelect.dataset.open = String(open);
  elements.recipeSelectTrigger.setAttribute("aria-expanded", String(open));
  if (open) {
    const selected = elements.recipeSelectMenu.querySelector<HTMLButtonElement>(
      'button[aria-selected="true"]',
    );
    (selected ?? elements.recipeSelectMenu.querySelector<HTMLButtonElement>("button"))?.focus();
  } else if (elements.recipeSelectMenu.contains(document.activeElement)) {
    elements.recipeSelectTrigger.focus();
  }
}

function chooseRecipe(recipe: LabRecipe): void {
  setRecipeMenuOpen(false);
  applyGenerationRequest(setLabRecipe(recipe), () => {
    renderCurrentScore();
    renderRuntimeSignal();
    announceAudition(`Opened ${labRecipeInfo(recipe).label}`);
  });
}

elements.recipeSelectTrigger.addEventListener("click", () => {
  setRecipeMenuOpen(elements.recipeSelectMenu.hidden);
});

elements.recipeSelectTrigger.addEventListener("keydown", (event) => {
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    setRecipeMenuOpen(true);
  }
});

elements.recipeSelectMenu.addEventListener("click", (event) => {
  const option = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-recipe]");
  if (option !== null && isLabRecipe(option.dataset.recipe)) {
    chooseRecipe(option.dataset.recipe);
  }
});

elements.recipeSelectMenu.addEventListener("keydown", (event) => {
  const options = [
    ...elements.recipeSelectMenu.querySelectorAll<HTMLButtonElement>("button[data-recipe]"),
  ];
  const current = options.indexOf(document.activeElement as HTMLButtonElement);
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    const delta = event.key === "ArrowDown" ? 1 : -1;
    options[(current + delta + options.length) % options.length]?.focus();
  } else if ((event.key === "Enter" || event.key === " ") && current >= 0) {
    event.preventDefault();
    const recipe = options[current]?.dataset.recipe;
    if (isLabRecipe(recipe)) {
      chooseRecipe(recipe);
    }
  } else if (event.key === "Escape") {
    event.preventDefault();
    setRecipeMenuOpen(false);
  }
});

document.addEventListener("click", (event) => {
  if (!elements.recipeSelect.contains(event.target as Node)) {
    setRecipeMenuOpen(false);
  }
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !elements.recipeSelectMenu.hidden) {
    setRecipeMenuOpen(false);
  }
});

for (const button of document.querySelectorAll<HTMLButtonElement>("[data-open-lab]")) {
  button.addEventListener("click", () => {
    const recipe = button.dataset.recipe;
    window.location.hash = "lab";
    if (isLabRecipe(recipe)) {
      applyGenerationRequest(setLabRecipe(recipe), () => {
        renderCurrentScore();
        renderRuntimeSignal();
      });
    }
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
  if (labRecipe === "suspense") {
    requestSuspensePhase();
  } else if (labRecipe === "adventure") {
    requestAdventureScene();
  } else {
    requestMusicState();
  }
});

elements.intensity.addEventListener("input", () => {
  elements.intensityValue.value = `${Math.round(Number(elements.intensity.value) * 100)}%`;
  requestMusicState();
});
elements.pressure.addEventListener("input", () => {
  elements.pressureValue.value = `${Math.round(Number(elements.pressure.value) * 100)}%`;
  requestMusicState();
});
elements.finalLap.addEventListener("change", () => requestMusicState());

window.addEventListener("hashchange", renderView);
