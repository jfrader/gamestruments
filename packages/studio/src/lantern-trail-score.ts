import type { AuthoringScore } from "./export-score.js";

export const lanternTrailRules = [
  {
    target: "quest-complete",
    priority: 100,
    when: { categorical: { areaPhase: "complete" } },
  },
  {
    target: "quest-complete",
    priority: 95,
    when: { numeric: { questComplete: { min: 1 } } },
  },
  {
    target: "sanctuary",
    priority: 90,
    when: { numeric: { discovery: { min: 0.85 } } },
  },
  {
    target: "danger",
    priority: 80,
    when: { numeric: { threat: { min: 0.7 } } },
  },
  {
    target: "danger",
    priority: 70,
    when: { categorical: { areaPhase: "danger" } },
  },
  {
    target: "clue",
    priority: 60,
    when: { numeric: { discovery: { min: 0.3 } } },
  },
  {
    target: "clue",
    priority: 50,
    when: { categorical: { areaPhase: "clue" } },
  },
  {
    target: "explore",
    priority: 30,
    when: { categorical: { areaPhase: "explore" } },
  },
  {
    target: "camp",
    priority: 10,
    when: { categorical: { areaPhase: "camp" } },
  },
] satisfies AuthoringScore["rules"];
