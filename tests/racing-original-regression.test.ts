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

// Only Cruise/Attack were reworked for the audition. The independent v1.2.0
// baseline below protects every other Original section from rebaselining.
const FROZEN: Readonly<Record<string, string>> = {
  "funk-normal": "76ca2d985a3e5cfc6781037367d395d68830de3432265c2784bdca95fa5266f8",
  "chip-normal": "bc8a5c74b66195c500363fa63c5dd03575372ff124e054f29222880786959747",
  "fusion-normal": "b1015f42fb4d9b186b0dc3972e47407057ce8b78c04db906c7e653161cf6603a",
  "neon-normal": "1bfdde2271da67540922c3906638074a87e35243b29aff805141186add500e20",
  "funk-high-register": "b6b7d902580a126cc6173d48f1db6fec203d33faa40176275e29c7ed3ce7555d",
  "chip-high-bright": "0030a212de19ffcd24ca1524de60d790147c6b9bdf11011641a96435a195643b",
  "fusion-max-energy": "83a17fc3e433983e121ceebc1da92e1a98ce3624790f0121cf4e445c9d32b2e5",
  "neon-low-bright": "9b8c6335aba5703e9f7a913156d69a61400bc1834114c9958efdfa9f5a28834b",
  "funk-low-bright": "607bc329cbab9993213a21a09d6d575ef36e041a7075ea8170f1024ebc539f7d",
  "chip-max-all": "91ccea3660449da421368ce8fd23f13c791c61c1f5f6b62d3e7cce296f9d6400",
  "fusion-seed-b": "ac9f9f4547413251b909c4898dfe075413854e0837d81741e333ac2b093773a1",
  "neon-high-register": "349f15c9f5e7558f44938a37ef532d23af6674ca23c96b54d24a69a7e8c3a9d7",
  "funk-native-default": "3a2183d6563fbaa11e54c21cf7275ba4bf47e9d110ded42a447313f7261d0452",
  "chip-native-default": "b27168a73085058a38e8a0260264dd01dfdcba4ea1af0300cd341b409ca4daa5",
  "fusion-native-default": "db30a88e8a8bc16e7eb0049ccd3b93b7493c88479ac6355c5d01001bdb4a52fb",
  "neon-native-default": "64089c44e5930c88d316fbea9e4a59b2d90b3c49411792b78ce4a6733f953317",
};

const UNCHANGED_SECTIONS = ["garage", "grid", "final-lap", "victory"] as const;
const UNCHANGED_V1_2_0: Readonly<Record<string, string>> = {
  "funk-normal": "0fbc645ef03311a61f61d4b6099231517bbe88554d7835eab4983585f531f744",
  "chip-normal": "81f6aa4b5ebee565ca42119ec6390017550373c66c8154919dd243e0f9f17d62",
  "fusion-normal": "8c21bc8961e84ea4dd43d68526096bb659df626bad73ac1e598eced4b1dfc941",
  "neon-normal": "798565aa956d85557a680533718f22d27d485cc4ca3418c107854e19e249fab7",
  "funk-high-register": "e1cbebdf1b138aea3766c5c473af4fc8949f2176a316095645c7b6ffadeed040",
  "chip-high-bright": "fea8b7bd6271a2ce39ed244e5d45edcc089889be866358ca43c0e9ab3dc6463f",
  "fusion-max-energy": "0eb3e889080fe78aa7c6a64c0dd27dde1442f74f9fd383493d133cf78842b41c",
  "neon-low-bright": "a0469cb0d8305f8ce10185b38a9391c00de0a50ae9589cef21f34d8977f9baae",
  "funk-low-bright": "d114b871fba1f79339d0cbb49e755d4a40e333d71b22af8f1a5e60417641d816",
  "chip-max-all": "9b72b956f546278c6f0e9331f98ea23007cd163d142b8d0207d9de92bb2b77ae",
  "fusion-seed-b": "252bc4c183966c19d2a8ee1a4adce3ff74e3e374082b91f9f217cc8d1dbe7310",
  "neon-high-register": "29a9a013116059dc99ad334deea41cd55ff4bb7f788a89211bc50c80051a862d",
  "funk-native-default": "ff080ca9f74a2130436fc39455c7c423b3884533830b3b382a9ef49768da4f61",
  "chip-native-default": "1c409a0947da4a8851bcb24350287c1565c5513488c36ff5922f069355af1bb7",
  "fusion-native-default": "06b2dbd61e2006af8463c5234e71ece80f455abb0f8d4247e07769ba15d021ba",
  "neon-native-default": "b7e3a5db8a6efae1a0e3ac98d4e087b0653a36d9d68744dbe7f04b3e4638ddbe",
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
  it("identifies the new generator in Lab and native scores", () => {
    for (const recipe of CASES) {
      assert.match(generate(recipe, "original").id, /^racing-generated-v1-12-0-/);
    }
  });

  it("includes every normalized trait in Lab and native score identity", () => {
    for (const secret of ["", "my-game"]) {
      const recipe = { ...CASES[0]!, secret };
      const original = generate(recipe, "original");
      for (const trait of ["energy", "complexity", "brightness", "syncopation"] as const) {
        assert.notEqual(generate({ ...recipe, [trait]: 0.1 }, "original").id, original.id, trait);
      }
      assert.equal(
        generate({ ...recipe, energy: 2 }, "original").id,
        generate({ ...recipe, energy: 1 }, "original").id,
      );
    }
  });

  it("preserves the four untouched sections from v1.2.0", () => {
    for (const recipe of CASES) {
      for (const arrangement of ["original", "extended"] as const) {
        assert.equal(
          digest(canonicalMusic(generate(recipe, arrangement), UNCHANGED_SECTIONS)),
          UNCHANGED_V1_2_0[recipe.name],
          `${recipe.name}/${arrangement} changed an untouched section`,
        );
      }
    }
  });

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
