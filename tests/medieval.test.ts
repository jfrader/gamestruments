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

const SCENES = ["explore", "town", "dungeon", "combat", "boss", "tavern", "victory"];

function generate(style: string): { bytes: Uint8Array; score: PortableScore } {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    recipe: "medieval",
    secret: "",
    seed: "realm-001",
    style,
    energy: 0.62,
    complexity: 0.5,
    brightness: 0.62,
    syncopation: 0.66,
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

describe("Medieval recipe through the shipped WASM", () => {
  it("generates the seven adventure-arc sections", () => {
    const { score } = generate("minstrel");
    assert.deepEqual(score.sections.map((section) => section.id), SCENES);
    assert.equal(score.defaultSection, "explore");
    assert.equal(score.form, undefined);
    const barTicks = score.beatsPerBar * score.ticksPerBeat;
    for (const section of score.sections) {
      assert.equal(section.lengthTicks, 8 * barTicks, `${section.id} length`);
      assert.ok(section.events.length > 8, `${section.id} should carry events`);
    }
  });

  it("is deterministic and style-sensitive", () => {
    const first = generate("court");
    const repeated = generate("court");
    assert.deepEqual(first.bytes, repeated.bytes);
    const chapel = generate("chapel");
    assert.notEqual(first.score.id, chapel.score.id);
    assert.notEqual(first.score.bpm, chapel.score.bpm);
    assert.notEqual(first.score.title, chapel.score.title);
  });

  it("maps every scene to its section", () => {
    const { score } = generate("minstrel");
    for (const scene of SCENES) {
      const state: GameState = { numeric: { danger: 0.2 }, categorical: { scene } };
      const request = new AdaptiveTransport(score).requestState(state, 0);
      if (scene === "explore") {
        assert.equal(request.status, "unchanged", "explore is the starting scene");
        continue;
      }
      assert.equal(request.status, "scheduled", `${scene} should schedule`);
      if (request.status === "scheduled") {
        assert.equal(request.plan.to, scene);
      }
    }
  });

  it("escalates a dangerous combat into the boss section", () => {
    const { score } = generate("court");
    const request = new AdaptiveTransport(score).requestState(
      { numeric: { danger: 0.9 }, categorical: { scene: "combat" } },
      0,
    );
    assert.equal(request.status, "scheduled");
    if (request.status === "scheduled") {
      assert.equal(request.plan.to, "boss");
    }
  });
});
