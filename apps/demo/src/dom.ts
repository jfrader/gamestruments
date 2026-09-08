export function requireElement<ElementType extends Element>(selector: string): ElementType {
  const element = document.querySelector<ElementType>(selector);
  if (element === null) {
    throw new Error(`Missing Audio Lab element: ${selector}`);
  }
  return element;
}

export const elements = {
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
  centerPlay: requireElement<HTMLButtonElement>("#center-play"),
};
