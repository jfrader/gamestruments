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

const SECTIONS = ["camp", "explore", "clue", "danger", "sanctuary", "quest-complete"];

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
  it("generates the six lantern-trail sections", () => {
    const { score } = generate("campfire");
    assert.deepEqual(score.sections.map((section) => section.id), SECTIONS);
    assert.equal(score.defaultSection, "camp");
    assert.equal(score.form, undefined);
    const barTicks = score.beatsPerBar * score.ticksPerBeat;
    for (const section of score.sections) {
      assert.equal(section.lengthTicks, 8 * barTicks, `${section.id} length`);
      assert.ok(section.events.length > 8, `${section.id} should carry events`);
    }
  });

  it("is deterministic and style-sensitive", () => {
    const first = generate("wilds");
    const repeated = generate("wilds");
    assert.deepEqual(first.bytes, repeated.bytes);
    const ruins = generate("ruins");
    assert.notEqual(first.score.id, ruins.score.id);
    assert.notEqual(first.score.bpm, ruins.score.bpm);
    assert.notEqual(first.score.title, ruins.score.title);
  });

  it("maps area phase, discovery, threat, and quest progress to sections", () => {
    const { score } = generate("campfire");
    const request = (state: GameState) => new AdaptiveTransport(score).requestState(state, 0);
    const phase = (areaPhase: string) => ({ numeric: {}, categorical: { areaPhase } });

    assert.equal(request(phase("camp")).status, "unchanged");
    assert.equal(request(phase("explore")).status, "scheduled");
    assert.equal(request(phase("clue")).status, "scheduled");
    assert.equal(request(phase("danger")).status, "scheduled");

    const byDiscovery = request({ numeric: { discovery: 0.9 }, categorical: {} });
    assert.equal(byDiscovery.status, "scheduled");
    if (byDiscovery.status === "scheduled") assert.equal(byDiscovery.plan.to, "sanctuary");

    const byThreat = request({ numeric: { threat: 0.8 }, categorical: {} });
    assert.equal(byThreat.status, "scheduled");
    if (byThreat.status === "scheduled") assert.equal(byThreat.plan.to, "danger");

    const complete = request({ numeric: { questComplete: 1 }, categorical: {} });
    assert.equal(complete.status, "scheduled");
    if (complete.status === "scheduled") assert.equal(complete.plan.to, "quest-complete");
  });
});
