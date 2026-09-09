import { selectMatchingRule, selectSection } from "./conditions.js";
import type {
  GameState,
  PortableScore,
  SectionGain,
  SectionId,
  TransitionPlan,
  TransitionRequest,
  TransportSnapshot,
} from "./types.js";
import { validatePortableScore } from "./validate-score.js";

function quantizeUp(tick: number, quantum: number): number {
  return Math.ceil(tick / quantum) * quantum;
}

export class AdaptiveTransport {
  readonly #score: PortableScore;
  readonly #barTicks: number;
  #currentSection: SectionId;
  #pendingSection: SectionId | null = null;
  #transition: TransitionPlan | null = null;
  #formStepIndex = 0;
  #sectionEnteredAt = 0;
  #cueTarget: SectionId | null = null;

  constructor(score: PortableScore, initialSection = score.defaultSection) {
    validatePortableScore(score);
    const sectionIds = new Set(score.sections.map((section) => section.id));
    if (!sectionIds.has(initialSection)) {
      throw new Error(`Unknown initial section: ${initialSection}`);
    }
    for (const rule of score.rules) {
      if (!sectionIds.has(rule.target)) {
        throw new Error(`Rule targets unknown section: ${rule.target}`);
      }
    }

    this.#score = score;
    this.#barTicks = score.beatsPerBar * score.ticksPerBeat;
    this.#currentSection = initialSection;
    this.#formStepIndex = formIndexFor(score, initialSection) ?? 0;
  }

  requestState(state: GameState, atTick: number): TransitionRequest {
    if (this.#score.form === undefined) {
      return this.requestSection(selectSection(this.#score, state), atTick);
    }
    this.#assertTick(atTick);
    this.advance(atTick);
    const match = selectMatchingRule(this.#score, state);
    if (match === undefined) {
      this.#cueTarget = null;
      return { status: "unchanged" };
    }
    if (match.hold !== false) {
      this.#cueTarget = null;
      this.#syncFormTo(match.target);
      return this.requestSection(match.target, atTick);
    }
    if (this.#cueTarget === match.target) {
      return { status: "unchanged" };
    }
    this.#cueTarget = match.target;
    this.#syncFormTo(match.target);
    return this.requestSection(match.target, atTick);
  }

  requestSection(target: SectionId, atTick: number): TransitionRequest {
    this.#assertTick(atTick);
    this.advance(atTick);
    this.#cueTarget = null;
    this.#syncFormTo(target);

    if (
      target === this.#currentSection &&
      this.#transition === null &&
      this.#pendingSection === null
    ) {
      return { status: "unchanged" };
    }

    if (!this.#score.sections.some((section) => section.id === target)) {
      throw new Error(`Unknown target section: ${target}`);
    }

    if (this.#transition !== null) {
      if (target === this.#transition.to) {
        this.#pendingSection = null;
        return { status: "unchanged" };
      }
      if (atTick < this.#transition.startTick) {
        const replacedPlan = this.#transition;
        this.#pendingSection = null;
        if (target === this.#currentSection) {
          this.#transition = null;
          return { status: "cancelled", plan: replacedPlan };
        }
        const plan = this.#createPlan(this.#currentSection, target, atTick);
        this.#transition = plan;
        return { status: "scheduled", plan, replacedPlan };
      }
      this.#pendingSection = target;
      return { status: "queued", target };
    }

    const plan = this.#createPlan(this.#currentSection, target, atTick);
    this.#transition = plan;
    return { status: "scheduled", plan };
  }

  jumpSection(target: SectionId, atTick: number): void {
    this.#assertTick(atTick);
    if (!this.#score.sections.some((section) => section.id === target)) {
      throw new Error(`Unknown target section: ${target}`);
    }
    this.#currentSection = target;
    this.#pendingSection = null;
    this.#transition = null;
    this.#sectionEnteredAt = atTick;
    this.#syncFormTo(target);
  }

  advance(atTick: number): TransitionPlan | null {
    this.#assertTick(atTick);
    if (this.#transition !== null && atTick >= this.#transition.endTick) {
      this.#currentSection = this.#transition.to;
      this.#sectionEnteredAt = atTick;
      this.#syncFormTo(this.#currentSection);
      this.#transition = null;

      const queued = this.#pendingSection;
      this.#pendingSection = null;
      if (queued !== null && queued !== this.#currentSection) {
        const plan = this.#createPlan(this.#currentSection, queued, atTick);
        this.#transition = plan;
        return plan;
      }
    }

    return this.#maybeAdvanceForm(atTick);
  }

  mixAt(atTick: number): SectionGain[] {
    this.#assertTick(atTick);
    const transition = this.#transition;
    if (transition === null || atTick < transition.startTick) {
      return [{ section: this.#currentSection, gain: 1 }];
    }
    if (atTick >= transition.endTick) {
      return [{ section: transition.to, gain: 1 }];
    }

    const progress =
      (atTick - transition.startTick) /
      (transition.endTick - transition.startTick);
    return [
      {
        section: transition.from,
        gain: Math.cos((progress * Math.PI) / 2),
      },
      {
        section: transition.to,
        gain: Math.sin((progress * Math.PI) / 2),
      },
    ];
  }

  snapshot(): TransportSnapshot {
    return {
      currentSection: this.#currentSection,
      pendingSection: this.#pendingSection,
      transition: this.#transition,
    };
  }

  #createPlan(
    from: SectionId,
    to: SectionId,
    requestedAtTick: number,
  ): TransitionPlan {
    const startTick = quantizeUp(requestedAtTick, this.#barTicks);
    return {
      from,
      to,
      requestedAtTick,
      startTick,
      endTick: startTick + this.#score.crossfadeBars * this.#barTicks,
    };
  }

  #assertTick(tick: number): void {
    if (!Number.isSafeInteger(tick) || tick < 0) {
      throw new Error(`Tick must be a non-negative safe integer: ${tick}`);
    }
  }

  #syncFormTo(section: SectionId): void {
    const index = formIndexFor(this.#score, section);
    if (index !== undefined) {
      this.#formStepIndex = index;
    }
  }

  #maybeAdvanceForm(atTick: number): TransitionPlan | null {
    const form = this.#score.form;
    if (
      form === undefined ||
      this.#transition !== null ||
      this.#pendingSection !== null
    ) {
      return null;
    }
    const step = form.steps[this.#formStepIndex];
    if (step === undefined || step.section !== this.#currentSection) {
      return null;
    }
    const section = this.#score.sections.find(
      (candidate) => candidate.id === this.#currentSection,
    );
    if (section === undefined) {
      return null;
    }
    const repeats = step.repeats ?? 1;
    if (atTick - this.#sectionEnteredAt < section.lengthTicks * repeats) {
      return null;
    }
    const nextIndex =
      this.#formStepIndex + 1 < form.steps.length
        ? this.#formStepIndex + 1
        : form.loopFrom;
    if (nextIndex === undefined || form.steps[nextIndex] === undefined) {
      return null;
    }
    const nextSection = form.steps[nextIndex].section;
    this.#formStepIndex = nextIndex;
    if (nextSection === this.#currentSection) {
      this.#sectionEnteredAt = atTick;
      return null;
    }
    const plan = this.#createPlan(this.#currentSection, nextSection, atTick);
    this.#transition = plan;
    return plan;
  }
}

function formIndexFor(score: PortableScore, section: SectionId): number | undefined {
  const index = score.form?.steps.findIndex((step) => step.section === section);
  return index === undefined || index < 0 ? undefined : index;
}
