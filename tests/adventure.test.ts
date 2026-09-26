import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import {
  AdaptiveTransport,
  validatePortableScore,
  type GameState,
  type PortableScore,
} from "../packages/runtime/src/index.ts";

const { instance } = await WebAssembly.instantiate(new Uint8Array(await readFile(
  new URL("../apps/demo/public/engine/gamestruments_engine.wasm", import.meta.url),
)));
const exports = instance.exports;
const memory = exports.memory;
assert.ok(memory instanceof WebAssembly.Memory);

const SECTIONS = [
  "camp", "explore", "town", "festival", "reunion", "dungeon", "skirmish",
  "combat", "chase", "boss", "assault", "sanctuary", "dawn", "victory",
];
function generate(style: string, autoplay?: boolean): { bytes: Uint8Array; score: PortableScore } {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    recipe: "adventure",
    secret: "",
    seed: "trail-001",
    style,
    ...(autoplay === undefined ? {} : { autoplay }),
    energy: 0.5,
    complexity: 0.45,
    brightness: 0.68,
    syncopation: 0.5,
  }));
  (exports.gamestruments_reset as () => void)();
  const ptr = (exports.gamestruments_alloc as (length: number) => number)(input.length);
  new Uint8Array(memory.buffer).set(input, ptr);
  const result = (exports.gamestruments_score_json as (ptr: number, length: number) => number)(ptr, input.length);
  const length = (exports.gamestruments_output_len as () => number)();
  const bytes = new Uint8Array(memory.buffer, result, length).slice();
  const text = new TextDecoder().decode(bytes);
  if ((exports.gamestruments_status as () => number)() !== 0) {
    throw new Error(text);
  }
  const score = JSON.parse(text) as PortableScore;
  validatePortableScore(score);
  return { bytes, score };
}

describe("Adventure recipe through the shipped WASM", () => {
  it("keeps game-mode generation state-driven and gives every quest phase room to develop", () => {
    const { score } = generate("folk");
    assert.deepEqual(score.sections.map((section) => section.id), SECTIONS);
    assert.equal(score.defaultSection, "camp");
    assert.equal(score.form, undefined);
    const barTicks = score.beatsPerBar * score.ticksPerBeat;
    for (const section of score.sections) {
      const bars = section.lengthTicks / barTicks;
      assert.equal(Number.isInteger(bars), true, `${section.id} must end on a bar`);
      assert.equal(bars % 8, 0, `${section.id} must contain complete movements`);
      assert.ok(bars >= 16, `${section.id} should have at least two movements`);
      assert.ok(section.events.length > 8, `${section.id} should carry events`);
    }
  });

  it("is deterministic and gives all three styles distinct voices", () => {
    const first = generate("folk");
    const repeated = generate("folk");
    assert.deepEqual(first.bytes, repeated.bytes);
    const scores = [first.score, generate("dark").score, generate("orchestral").score];
    assert.equal(new Set(scores.map((score) => score.id)).size, 3);
    assert.equal(new Set(scores.map((score) => score.title)).size, 3);
    const voiceArrangements = scores.map((score) => score.sections.map((section) =>
      section.events
        .filter((event) => event.kind === "note" || event.kind === "percussion")
        .map((event) => `${event.lane}:${event.voice}`)
        .join("|"),
    ).join("/"));
    assert.equal(new Set(voiceArrangements).size, 3);
  });

  it("maps area phase, discovery, threat, and quest progress to sections", () => {
    const { score } = generate("dark");
    const request = (state: GameState) => new AdaptiveTransport(score).requestState(state, 0);
    const phase = (areaPhase: string) => ({ numeric: {}, categorical: { areaPhase } });

    assert.equal(request(phase("camp")).status, "unchanged");
    for (const id of ["explore", "town", "dungeon", "boss", "sanctuary", "victory"]) {
      const result = request(phase(id));
      assert.equal(result.status, "scheduled", `${id} should schedule`);
      if (result.status === "scheduled") assert.equal(result.plan.to, id);
    }

    const sanctuary = request({ numeric: { discovery: 0.9 }, categorical: {} });
    assert.equal(sanctuary.status, "scheduled");
    if (sanctuary.status === "scheduled") assert.equal(sanctuary.plan.to, "sanctuary");

    const combat = request({ numeric: { threat: 0.8 }, categorical: {} });
    assert.equal(combat.status, "scheduled");
    if (combat.status === "scheduled") assert.equal(combat.plan.to, "combat");

    const boss = request({ numeric: { threat: 0.9 }, categorical: { areaPhase: "combat" } });
    assert.equal(boss.status, "scheduled");
    if (boss.status === "scheduled") assert.equal(boss.plan.to, "boss");

    const complete = request({ numeric: { questComplete: 1 }, categorical: {} });
    assert.equal(complete.status, "scheduled");
    if (complete.status === "scheduled") assert.equal(complete.plan.to, "victory");
  });

  it("supersedes an in-flight manual form step with quest_complete and drops queued cues", () => {
    const { score } = generate("folk", true);
    const bar = score.beatsPerBar * score.ticksPerBeat;
    const transport = new AdaptiveTransport(score);
    assert.equal(transport.snapshot().currentSection, "camp");

    // A manual form step commits at the next bar boundary.
    const step = transport.advanceForm(bar / 2);
    assert.equal(step.status, "scheduled");
    if (step.status !== "scheduled") return;
    assert.equal(step.plan.to, "explore");
    assert.equal(step.plan.startTick, bar);

    // The form step is now in flight: past its start tick but not complete.
    transport.advance(step.plan.startTick);
    assert.equal(transport.snapshot().currentSection, "camp");

    // A direct cue during the in-flight form step defers (queues) behind it.
    const cue = transport.requestSection("town", step.plan.startTick);
    assert.equal(cue.status, "queued");

    // quest_complete arrives mid-transition and must supersede the stale form step.
    const victory = transport.requestState({ numeric: { questComplete: 1 }, categorical: {} }, step.plan.startTick);
    assert.equal(victory.status, "scheduled");
    if (victory.status !== "scheduled") return;
    assert.equal(victory.plan.to, "victory");

    // The superseded transition reaches victory.
    transport.advance(victory.plan.endTick);
    assert.equal(transport.snapshot().currentSection, "victory");
    assert.equal(transport.snapshot().transition, null);
    assert.equal(transport.snapshot().pendingSection, null);
  });

  it("supersedes an in-flight automatic form transition with quest_complete", () => {
    const { score } = generate("folk", true);
    const campLen = score.sections.find(s => s.id === "camp")!.lengthTicks;
    const transport = new AdaptiveTransport(score);
    assert.equal(transport.snapshot().currentSection, "camp");

    // Let the form auto-advance camp -> explore.
    transport.advance(campLen);
    const automatic = transport.snapshot().transition;
    assert.ok(automatic);
    assert.equal(automatic!.to, "explore");

    // quest_complete mid-flight must supersede the automatic progression.
    const victory = transport.requestState({ numeric: { questComplete: 1 }, categorical: {} }, automatic!.startTick + 1);
    assert.equal(victory.status, "scheduled");
    if (victory.status !== "scheduled") return;
    assert.equal(victory.plan.to, "victory");
    
    transport.advance(victory.plan.endTick);
    assert.equal(transport.snapshot().currentSection, "victory");
  });

  it("a held cue still defers to the active cue after the supersede change", () => {
    const { score } = generate("folk", true);
    const transport = new AdaptiveTransport(score);
    
    const cue = transport.requestSection("explore", 0);
    assert.equal(cue.status, "scheduled");
    if (cue.status !== "scheduled") return;
    
    transport.advance(cue.plan.startTick);
    
    // quest_complete during an explicit held cue defers (queues) behind it.
    const victory = transport.requestState({ numeric: { questComplete: 1 }, categorical: {} }, cue.plan.startTick + 1);
    assert.equal(victory.status, "queued");
    assert.equal(transport.snapshot().currentSection, "camp");
  });
});
