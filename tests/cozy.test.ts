import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import {
  AdaptiveTransport,
  selectSection,
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

const SECTIONS = ["dawn", "morning", "market", "noon", "rain", "evening", "festival", "night"];

function generate(style: string, arrangement = "", autoplay = false): { bytes: Uint8Array; score: PortableScore } {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    recipe: "cozy",
    secret: "",
    seed: "day-001",
    style,
    arrangement,
    autoplay,
    energy: 0.5,
    complexity: 0.6,
    brightness: 0.6,
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

const day = (hour: number, place = "home", rain = 0): GameState => ({
  numeric: { hour, rain },
  categorical: { place },
});

describe("Cozy recipe through the shipped WASM", () => {
  it("generates the day's eight sections, state-driven, for all three styles", () => {
    const scores = ["acoustic", "lofi", "bossa"].map((style) => generate(style).score);
    for (const score of scores) {
      assert.deepEqual(score.sections.map((section) => section.id), SECTIONS);
      assert.equal(score.defaultSection, "dawn");
      assert.equal(score.form, undefined);
    }
    assert.equal(new Set(scores.map((score) => score.title)).size, 3);
    const voices = scores.map((score) => {
      const used = score.sections.flatMap((section) =>
        section.events.flatMap((event) =>
          event.kind === "note" || event.kind === "percussion" ? [event.voice] : [],
        ),
      );
      return [...new Set(used)].sort().join(",");
    });
    assert.equal(new Set(voices).size, 3, "each style has its own ensemble");
  });

  it("is deterministic", () => {
    assert.deepEqual(generate("lofi").bytes, generate("lofi").bytes);
  });

  it("follows the clock, the place and the rain", () => {
    const { score } = generate("bossa");
    for (const [state, expected] of [
      [day(3), "night"],
      [day(6.5), "dawn"],
      [day(9), "morning"],
      [day(9, "town"), "market"],
      [day(14), "noon"],
      [day(18.5), "evening"],
      [day(22, "town"), "night"],
      [day(14, "fields", 0.8), "rain"],
      [day(23, "festival", 1), "festival"],
    ] as const) {
      assert.equal(selectSection(score, state), expected, JSON.stringify(state));
    }
    const moved = new AdaptiveTransport(score).requestState(day(9), 0);
    assert.equal(moved.status, "scheduled");
    if (moved.status === "scheduled") assert.equal(moved.plan.to, "morning");
  });

  it("offers the day tour, all phases and a seeded day", () => {
    const tour = generate("acoustic", "", true).score.form;
    assert.deepEqual(tour?.steps.map((step) => step.section), ["dawn", "morning", "noon", "market", "evening", "night"]);
    const all = generate("acoustic", "all-phases").score.form;
    assert.deepEqual(all?.steps.map((step) => step.section), SECTIONS);
    const seeded = generate("acoustic", "seeded").score.form;
    assert.equal(seeded?.steps[0]?.section, "dawn");
    assert.equal(seeded?.steps.at(-1)?.section, "night");
  });
});
