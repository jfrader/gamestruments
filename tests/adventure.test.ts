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

const SECTIONS = ["camp", "explore", "town", "dungeon", "combat", "boss", "sanctuary", "victory"];
const LONG = ["explore", "town", "combat", "victory"];

function generate(style: string): { bytes: Uint8Array; score: PortableScore } {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    recipe: "adventure",
    secret: "",
    seed: "trail-001",
    style,
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
  it("generates eight sections, four of them long two-movement arrangements", () => {
    const { score } = generate("campfire");
    assert.deepEqual(score.sections.map((section) => section.id), SECTIONS);
    assert.equal(score.defaultSection, "camp");
    assert.equal(score.form, undefined);
    const barTicks = score.beatsPerBar * score.ticksPerBeat;
    for (const section of score.sections) {
      const expected = (LONG.includes(section.id) ? 16 : 8) * barTicks;
      assert.equal(section.lengthTicks, expected, `${section.id} length`);
      assert.ok(section.events.length > 8, `${section.id} should carry events`);
    }
  });

  it("is deterministic and style-sensitive", () => {
    const first = generate("wilds");
    const repeated = generate("wilds");
    assert.deepEqual(first.bytes, repeated.bytes);
    const chapel = generate("chapel");
    assert.notEqual(first.score.id, chapel.score.id);
    assert.notEqual(first.score.bpm, chapel.score.bpm);
    assert.notEqual(first.score.title, chapel.score.title);
  });

  it("maps area phase, discovery, threat, and quest progress to sections", () => {
    const { score } = generate("court");
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
});
