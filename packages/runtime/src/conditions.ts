import type {
  AdaptiveCondition,
  GameState,
  PortableScore,
  SectionId,
} from "./types.js";

export function matchesCondition(
  condition: AdaptiveCondition,
  state: GameState,
): boolean {
  for (const [name, range] of Object.entries(condition.numeric ?? {})) {
    const value = state.numeric[name];
    if (value === undefined || !Number.isFinite(value)) {
      return false;
    }
    if (range.min !== undefined && value < range.min) {
      return false;
    }
    if (range.max !== undefined && value > range.max) {
      return false;
    }
  }

  for (const [name, accepted] of Object.entries(
    condition.categorical ?? {},
  )) {
    const value = state.categorical[name];
    if (value === undefined) {
      return false;
    }
    const values = Array.isArray(accepted) ? accepted : [accepted];
    if (!values.includes(value)) {
      return false;
    }
  }

  return true;
}

export function selectSection(
  score: PortableScore,
  state: GameState,
): SectionId {
  const matchingRule = score.rules
    .map((rule, index) => ({ rule, index }))
    .filter(({ rule }) => matchesCondition(rule.when, state))
    .sort(
      (left, right) =>
        right.rule.priority - left.rule.priority || left.index - right.index,
    )[0]?.rule;

  return matchingRule?.target ?? score.defaultSection;
}
