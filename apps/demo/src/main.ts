import "./style.css";
import type { SectionId } from "../../../packages/runtime/src/index.ts";
import type { SoloMode } from "./audio-engine.ts";
import { versionLabel } from "./lab-copy.ts";
import { isArrangement, type Arrangement } from "./wasm-engine.ts";
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
  generationTraits,
  generationPreset,
  initializeLab,
  levelSeed,
  labRecipe,
  currentArrangement,
  setArrangement,
  currentPresets,
  phase,
  score,
  sectionById,
  soloMode,
  transport,
  applyPlan,
  requestMusicState,
  requestPhase,
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
  resetVersion,
  advanceVersion,
  nextVersionNumber,
  setPhase,
  setLabRecipe,
  isLabRecipe,
  labRecipeInfo,
  type LabRecipe,
  setSoloMode,
  versionIndex,
} from "./state";

import { setupScoreDebugger, renderDebuggerGrid } from "./score-grid.ts";

await initializeLab();
setupScoreDebugger();

/** The one status line per arrangement, for every recipe. */
const ARRANGEMENT_READY: Record<Arrangement, string> = {
  "all-phases": "All phases arrangement ready",
  seeded: "Seeded arrangement ready",
};

function renderCurrentScore(): void {
  renderScoreIdentity(
    activeExperimentIndex,
    score,
    levelSeed,
    generationTraits,
    soloMode,
    labRecipe,
    currentPresets(),
    currentArrangement(),
    phase,
  );
  renderDebuggerGrid();
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
  elements.runtimeSignal.textContent = labRecipeInfo(labRecipe).signal(phase);
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
    resetVersion();
    renderCurrentScore();
    announceAudition(`Generated ${score.title}`);
  });
});

elements.newPiece.addEventListener("click", () => {
  const nextSeed = nextLevelSeed(levelSeed);
  applyGenerationRequest(requestExperiment(activeExperimentIndex, nextSeed), () => {
    resetVersion();
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
    resetVersion();
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
    applyGenerationRequest(requestExperiment(activeExperimentIndex, levelSeed, traitsFromControls(), currentArrangement(), versionIndex), () => {
      // keep same take (reel) and do not resetVersion; activate keeps current section if present
      // so trait change flows without restarting to intro
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
  renderAuditionControls(levelSeed, soloMode);
  announceAudition(soloMode === "full" ? "Full mix on" : `${soloMode} solo on`);
});

elements.newVersion.addEventListener("click", () => {
  const take = nextVersionNumber();
  applyGenerationRequest(
    requestExperiment(
      activeExperimentIndex,
      levelSeed,
      generationTraits,
      currentArrangement(),
      take,
    ),
    async () => {
      advanceVersion();
      renderCurrentScore();
      if (!audio.running) {
        setStartButton(await toggleEngine());
      }
      announceAudition(versionLabel(levelSeed, take));
    },
  );
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
    resetVersion();
    renderCurrentScore();
    window.location.hash = "lab";
    announceAudition(`Generated ${score.title}`);
  });
});

elements.arrangementButtons.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>("button[data-arrangement]");
  const value = button?.dataset.arrangement;
  if (value === undefined) {
    return;
  }
  // Every recipe shares the same two arrangements; a stale value from another
  // recipe (e.g. a retired id left in the container) is ignored.
  if (!isArrangement(value)) {
    return;
  }
  applyGenerationRequest(setArrangement(value), () => {
    renderCurrentScore();
    announceAudition(ARRANGEMENT_READY[value]);
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
  requestPhase();
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
