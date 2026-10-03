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
  #formHeld = false;
  #formNotBefore = 0;
  #transitionSource: "automatic" | "form" | "explicit" = "explicit";

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
      return this.#requestSectionAs(selectSection(this.#score, state), atTick, "explicit", true);
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
      return this.#requestSectionAs(match.target, atTick, "explicit", true);
    }
    if (
      this.#cueTarget === match.target &&
      this.#currentSection === match.target &&
      this.#transition === null
    ) {
      return { status: "unchanged" };
    }
    const result = this.#requestSectionAs(match.target, atTick, "explicit", true);
    this.#cueTarget = match.target;
    return result;
  }

  requestSection(target: SectionId, atTick: number): TransitionRequest {
    return this.#requestSectionAs(target, atTick, "explicit", false);
  }

  #requestSectionAs(
    target: SectionId,
    atTick: number,
    source: "automatic" | "form" | "explicit",
    supersedeForm: boolean
  ): TransitionRequest {
    this.#assertTick(atTick);
    if (!this.#score.sections.some((section) => section.id === target)) {
      throw new Error(`Unknown target section: ${target}`);
    }
    this.advance(atTick);
    this.#cueTarget = null;

    if (
      target === this.#currentSection &&
      this.#transition === null &&
      this.#pendingSection === null
    ) {
      return { status: "unchanged" };
    }

    if (this.#transition !== null) {
      if (target === this.#transition.to) {
        this.#pendingSection = null;
        return { status: "unchanged" };
      }

      const supersede = supersedeForm && (this.#transitionSource === "automatic" || this.#transitionSource === "form");
      if (atTick < this.#transition.startTick || supersede) {
        const replacedPlan = this.#transition;
        this.#pendingSection = null;
        if (target === this.#currentSection) {
          this.#transition = null;
          this.#transitionSource = "explicit";
          this.#syncFormTo(this.#currentSection);
          return { status: "cancelled", plan: replacedPlan };
        }
        const plan = this.#createPlan(this.#currentSection, target, atTick);
        this.#transition = plan;
        this.#transitionSource = source;
        return { status: "scheduled", plan, replacedPlan };
      }
      this.#pendingSection = target;
      return { status: "queued", target };
    }

    const plan = this.#createPlan(this.#currentSection, target, atTick);
    this.#transition = plan;
    this.#transitionSource = source;
    return { status: "scheduled", plan };
  }

  cancelPending(atTick: number): TransitionPlan | null {
    this.#assertTick(atTick);
    this.#pendingSection = null;
    if (this.#transition !== null && atTick < this.#transition.startTick) {
      const plan = this.#transition;
      this.#transition = null;
      this.#transitionSource = "explicit";
      this.#syncFormTo(this.#currentSection);
      this.#cueTarget = null;
      return plan;
    }
    return null;
  }

  jumpSection(target: SectionId, atTick: number): void {
    this.#assertTick(atTick);
    if (!this.#score.sections.some((section) => section.id === target)) {
      throw new Error(`Unknown target section: ${target}`);
    }
    this.#currentSection = target;
    this.#pendingSection = null;
    this.#transition = null;
    this.#transitionSource = "explicit";
    this.#formNotBefore = 0;
    this.#cueTarget = null;
    this.#sectionEnteredAt = atTick;
    this.#syncFormTo(target);
  }

  advance(atTick: number, lookaheadTicks = 0): TransitionPlan | null {
    this.#assertTick(atTick);
    this.#assertTick(lookaheadTicks);
    if (this.#transition !== null && atTick >= this.#transition.endTick) {
      this.#currentSection = this.#transition.to;
      this.#cueTarget = null;
      this.#sectionEnteredAt = this.#score.form?.origin === "transitionStart"
        ? this.#transition.startTick
        : this.#score.form === undefined ? atTick : this.#transition.endTick;
      this.#syncFormTo(this.#currentSection);
      this.#transition = null;
      this.#transitionSource = "explicit";
      this.#formNotBefore = 0;

      const queued = this.#pendingSection;
      this.#pendingSection = null;
      if (queued !== null && queued !== this.#currentSection) {
        const plan = this.#createPlan(this.#currentSection, queued, atTick);
        this.#transition = plan;
        return plan;
      }
    }

    return this.#maybeAdvanceForm(atTick, lookaheadTicks);
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

  get formHeld(): boolean { return this.#formHeld; }

  setFormHeld(held: boolean, atTick: number): TransitionPlan | null {
    this.#assertTick(atTick);
    if (this.#score.form === undefined || this.#formHeld === held) return null;
    this.#formHeld = held;
    if (!held) this.#formNotBefore = atTick;
    if (held && this.#transitionSource === "automatic" && this.#transition !== null && atTick < this.#transition.startTick) {
      return this.cancelPending(atTick);
    }
    return null;
  }

  nextFormSection(atTick: number): SectionId | null {
    this.#assertTick(atTick);
    const form = this.#score.form;
    const current = this.#transition !== null && atTick >= this.#transition.startTick ? this.#transition.to : this.#currentSection;
    const index = formIndexFrom(this.#score, current, this.#formStepIndex);
    if (form === undefined || index === undefined) return null;
    let next = index;
    for (let step = 0; step < form.steps.length; step++) {
      const candidate = next + 1 < form.steps.length ? next + 1 : form.loopFrom;
      if (candidate === undefined) return null;
      next = candidate;
      const target = form.steps[next]?.section;
      if (target === undefined) return null;
      if (target !== current) return target;
    }
    return null;
  }

  advanceForm(atTick: number): TransitionRequest {
    const target = this.nextFormSection(atTick);
    return target === null ? { status: "unchanged" } : this.#requestSectionAs(target, atTick, "form", false);
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
    const index = formIndexFrom(this.#score, section, this.#formStepIndex);
    if (index !== undefined) {
      this.#formStepIndex = index;
    }
  }

  #maybeAdvanceForm(atTick: number, lookaheadTicks: number): TransitionPlan | null {
    const form = this.#score.form;
    if (
      form === undefined || this.#formHeld ||
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
    const cycles = Math.max(repeats, Math.ceil(Math.max(0, this.#formNotBefore - this.#sectionEnteredAt) / section.lengthTicks));
    const boundaryTick = this.#sectionEnteredAt + section.lengthTicks * cycles;
    if (atTick + lookaheadTicks < boundaryTick) {
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
    if (nextSection === this.#currentSection) {
      if (atTick >= boundaryTick) {
        this.#formStepIndex = nextIndex;
        this.#sectionEnteredAt = boundaryTick;
      }
      return null;
    }
    const plan: TransitionPlan = {
      from: this.#currentSection,
      to: nextSection,
      requestedAtTick: atTick,
      startTick: boundaryTick,
      endTick: boundaryTick + this.#score.crossfadeBars * this.#barTicks,
    };
    this.#transition = plan;
    this.#transitionSource = "automatic";
    return plan;
  }
}

function formIndexFor(score: PortableScore, section: SectionId): number | undefined {
  const index = score.form?.steps.findIndex((step) => step.section === section);
  return index === undefined || index < 0 ? undefined : index;
}

function formIndexFrom(score: PortableScore, section: SectionId, current: number): number | undefined {
  const form = score.form;
  if (form === undefined) return undefined;
  const steps = form.steps;
  if (steps[current]?.section === section) return current;
  const loopFrom = Math.min(form.loopFrom ?? 0, current);
  for (const [start, end] of [[current + 1, steps.length], [loopFrom, current], [0, loopFrom]] as const) {
    for (let index = start; index < end; index++) {
      if (steps[index]?.section === section) return index;
    }
  }
  return undefined;
}
