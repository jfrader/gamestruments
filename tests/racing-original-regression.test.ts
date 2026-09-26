import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import {
  validatePortableScore,
  type PortableScore,
} from "../packages/runtime/src/index.ts";

const ORIGINAL_ORDER = [
  "garage",
  "grid",
  "cruise",
  "attack",
  "final-lap",
  "victory",
] as const;

const { instance } = await WebAssembly.instantiate(new Uint8Array(await readFile(
  new URL("../apps/demo/public/engine/gamestruments_engine.wasm", import.meta.url),
)));
const exports = instance.exports;
const memory = exports.memory;
assert.ok(memory instanceof WebAssembly.Memory);

interface RacingCase {
  name: string;
  style: string;
  seed: string;
  /** Project secret; defaults to "" to match the lab, must be non-empty for Godot. */
  secret?: string;
  energy: number;
  complexity: number;
  brightness: number;
  syncopation: number;
}

/**
 * Frozen SHA256 digests of the canonicalized original six-section musical
 * material. They guard the Original Racing material — especially the high
 * register — against unauthorized re-voicing or pitch changes. Never update
 * these to the current output to make a failing test pass.
 *
 * Authorized re-baseline GURI-1240: the garage intro is an intentional
 * re-voice (its held pad and bass now sound through the bar and it gains a
 * light pulse, because the Funk style used to leave the bar ends nearly
 * silent). Every other Original section is byte-identical; only the garage
 * digests move. This is a product-owner decision, not a refresh to satisfy a
 * red test.
 */
const FROZEN: Readonly<Record<string, string>> = {
  "funk-normal": "75fab7a463036c2d6e4b5a1017184c9f417e902a1e6357b918a2be2d313d7fa9",
  "chip-normal": "e8758a6d6c40690e5c8007f341d652959128deec2057fa254ea947e071306c84",
  "fusion-normal": "6cd4cab538a66e60a63b09c05275e9fc23426560ab71a51a1e268dfaef50cf15",
  "neon-normal": "20c1c680bf6ac996fb57e11ba48978e61d9f968faaf4536dad5f4e17b4913abf",
  "funk-high-register": "5e109d380ee6e10d5100a6f78d8e68ea1001ab3ff3300d99e3659c402a7f86cf",
  "chip-high-bright": "42ca834351be21b95f32bf039a3044c215f847cc6087617cb1d2da6ba8b6a404",
  "fusion-max-energy": "a8784abe58797fcadc34f3083ab85301d0fe30916c5179b808b47952a666db21",
  "neon-low-bright": "af17bab78d7c10f34e742c56280099ca8e856e00eadcb54b29357f914bf00bbc",
  "funk-low-bright": "7e1c1087fd91ceda74f5463796b733c5f33aed79b0141167f2f0fae3fbf4f257",
  "chip-max-all": "5e0dfc77f425a0eb57f9d14aa734e01e56fdd19fd1fe3e81a43ff986d031ebba",
  "fusion-seed-b": "3fcf1ffb1643135cdf0fb6482a06769a6f141ac30e9e8c6a7ca940bca5586aa2",
  "neon-high-register": "4e56763448dec12566f202ff008ffce6be6ae48cf7b207e7c3624c5b0714cb11",
  "funk-native-default": "ce4b4e284d04968c476612cf76b3f68d7656f221afa8ff55d8f00e24d14cad0a",
  "chip-native-default": "eab7d084c7d08c18ea8242b6bf7717b5827590446d184232c2ae447248d3ad92",
  "fusion-native-default": "87af8d46864c1a7257d67f33db11e3c66b2f891ab608af465ff42ce186e1e50d",
  "neon-native-default": "6d6ef3f182fb3ff04d7b1cf83a1d4626bdfb9649a6da75ff485515b48051a56c",
};

const CASES: readonly RacingCase[] = [
  { name: "funk-normal", style: "funk", seed: "frozen-funk-normal", energy: 0.58, complexity: 0.62, brightness: 0.56, syncopation: 0.7 },
  { name: "chip-normal", style: "chip", seed: "frozen-chip-normal", energy: 0.6, complexity: 0.6, brightness: 0.6, syncopation: 0.6 },
  { name: "fusion-normal", style: "fusion", seed: "frozen-fusion-normal", energy: 0.55, complexity: 0.5, brightness: 0.5, syncopation: 0.55 },
  { name: "neon-normal", style: "neon", seed: "frozen-neon-normal", energy: 0.62, complexity: 0.5, brightness: 0.55, syncopation: 0.6 },
  { name: "funk-high-register", style: "funk", seed: "frozen-funk-high", energy: 0.95, complexity: 0.8, brightness: 0.98, syncopation: 0.9 },
  { name: "chip-high-bright", style: "chip", seed: "frozen-chip-bright", energy: 0.8, complexity: 0.7, brightness: 1.0, syncopation: 0.7 },
  { name: "fusion-max-energy", style: "fusion", seed: "frozen-fusion-energy", energy: 1.0, complexity: 0.6, brightness: 0.7, syncopation: 0.8 },
  { name: "neon-low-bright", style: "neon", seed: "frozen-neon-low", energy: 0.5, complexity: 0.4, brightness: 0.1, syncopation: 0.4 },
  { name: "funk-low-bright", style: "funk", seed: "frozen-funk-low", energy: 0.4, complexity: 0.5, brightness: 0.05, syncopation: 0.5 },
  { name: "chip-max-all", style: "chip", seed: "frozen-chip-max", energy: 1.0, complexity: 1.0, brightness: 1.0, syncopation: 1.0 },
  { name: "fusion-seed-b", style: "fusion", seed: "frozen-fusion-seedb", energy: 0.55, complexity: 0.5, brightness: 0.5, syncopation: 0.55 },
  { name: "neon-high-register", style: "neon", seed: "frozen-neon-high", energy: 0.9, complexity: 0.75, brightness: 0.92, syncopation: 0.85 },
  // Native-default setup: a non-empty project secret (Godot requires one) with
  // the lab's default traits, one case per racing style.
  { name: "funk-native-default", style: "funk", secret: "my-game", seed: "level-001", energy: 0.62, complexity: 0.6, brightness: 0.52, syncopation: 0.7 },
  { name: "chip-native-default", style: "chip", secret: "my-game", seed: "level-001", energy: 0.62, complexity: 0.6, brightness: 0.52, syncopation: 0.7 },
  { name: "fusion-native-default", style: "fusion", secret: "my-game", seed: "level-001", energy: 0.62, complexity: 0.6, brightness: 0.52, syncopation: 0.7 },
  { name: "neon-native-default", style: "neon", secret: "my-game", seed: "level-001", energy: 0.62, complexity: 0.6, brightness: 0.52, syncopation: 0.7 },
];

function generate(recipe: RacingCase, arrangement: "original" | "extended"): PortableScore {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    secret: recipe.secret ?? "",
    seed: recipe.seed,
    style: recipe.style,
    recipe: "racing",
    arrangement,
    autoplay: false,
    palette: { melody: "", harmony: "", drive: "", bass: "" },
    energy: recipe.energy,
    complexity: recipe.complexity,
    brightness: recipe.brightness,
    syncopation: recipe.syncopation,
  }));
  (exports.gamestruments_reset as () => void)();
  const ptr = (exports.gamestruments_alloc as (length: number) => number)(input.length);
  new Uint8Array(memory.buffer).set(input, ptr);
  const result = (exports.gamestruments_score_json as (ptr: number, length: number) => number)(ptr, input.length);
  const length = (exports.gamestruments_output_len as () => number)();
  const bytes = new Uint8Array(memory.buffer, result, length).slice();
  const text = new TextDecoder().decode(bytes);
  if ((exports.gamestruments_status as () => number)() !== 0) throw new Error(text);
  const score = JSON.parse(text) as PortableScore;
  validatePortableScore(score);
  return score;
}

/**
 * Canonicalizes only the musical material: BPM, timing, section length, and
 * every event's voice/pitch/velocity/onset/duration/lane. Non-musical metadata
 * (ids, titles, labels, schema/version fields, rules, forms) is deliberately
 * excluded so it never masks a real audible change.
 */
function canonicalMusic(score: PortableScore, sectionIds?: readonly string[]): unknown {
  const sections = sectionIds
    ? sectionIds.map((id) => score.sections.find((section) => section.id === id)!)
    : score.sections;
  return {
    bpm: score.bpm,
    beatsPerBar: score.beatsPerBar,
    ticksPerBeat: score.ticksPerBeat,
    sections: sections.map((section) => ({
      id: section.id,
      lengthTicks: section.lengthTicks,
      events: section.events.map((event) => {
        switch (event.kind) {
          case "note":
            return {
              lane: event.lane,
              startTick: event.startTick,
              durationTicks: event.durationTicks,
              velocity: event.velocity,
              voice: event.voice,
              kind: "note",
              pitch: event.pitch,
              role: event.role ?? null,
            };
          case "percussion":
            return {
              lane: event.lane,
              startTick: event.startTick,
              durationTicks: event.durationTicks,
              velocity: event.velocity,
              voice: event.voice,
              kind: "percussion",
            };
          case "stem":
            throw new Error("racing material must not contain stem events");
        }
      }),
    })),
  };
}

function digest(value: unknown): string {
  return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

describe("Racing Original material is frozen", () => {
  it("reproduces the frozen six-section material for every representative case", () => {
    for (const recipe of CASES) {
      const score = generate(recipe, "original");
      assert.deepEqual(
        score.sections.map((section) => section.id),
        [...ORIGINAL_ORDER],
        `${recipe.name} must keep the six-section order`,
      );
      assert.equal(score.form, undefined, `${recipe.name} autoplay must stay disabled`);
      assert.equal(
        digest(canonicalMusic(score)),
        FROZEN[recipe.name],
        `${recipe.name} original material drifted from the frozen baseline`,
      );
    }
  });

  it("keeps the six original Extended sections identical to the frozen Original material", () => {
    for (const recipe of CASES) {
      const extended = generate(recipe, "extended");
      assert.equal(
        digest(canonicalMusic(extended, ORIGINAL_ORDER)),
        FROZEN[recipe.name],
        `${recipe.name} Extended original sections drifted from the frozen baseline`,
      );
    }
  });
});
