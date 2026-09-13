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
 * material, generated with the WASM committed at baseline e906504
 * ("Add Suspense theme arrangement for title music (#64)"). They guard the
 * untouched Original Racing material — especially the high register — against
 * unauthorized re-voicing or pitch changes. Never update these to the current
 * output to make a failing test pass.
 */
const FROZEN: Readonly<Record<string, string>> = {
  "funk-normal": "b523492aa8b52dcfa52e78ba11e64458a1466b217d86917e9be961682e55fcc0",
  "chip-normal": "2ded738e8d21525cfcdde9642fb7247c7302e5e1572371d9a7090cdf72f42697",
  "fusion-normal": "56938a17987c816cc3b87340759030a36357c7ab8811ecefe901cece34464499",
  "neon-normal": "51fb1bf48d860ce19bd9ecfd77b8e591ad0ff3e2f94baac71a16a69d67b1342a",
  "funk-high-register": "9573faf0c79b6507a471d3214bcb60b3ba1bf320187931610e78fb1c12153508",
  "chip-high-bright": "5f5a6cd97aa97e459a4e6b7671d2a8fcabebd4d3ed3502574319f4225bc508d4",
  "fusion-max-energy": "e17e7e85f10302c5cad7a006c5ce603271b99151d0b7d7a6aad9172ac5a9978f",
  "neon-low-bright": "87a1c39e6b6f2ca4414831abe4985f35c6559afded33600dc2373adb059a9035",
  "funk-low-bright": "85dfd58cf4258cc9df5615e934d32e045e3c99dc7b667bdf685522d8f17f1e14",
  "chip-max-all": "9fee6c9f0607595f379f623cdbc34fd6861e5e0e3f81db3914fb4602e20390fd",
  "fusion-seed-b": "ce566430399f8ad3ccb7c3bac3efea641e3e6571a93eee6b4aa6b20fd575ca2c",
  "neon-high-register": "54da0d8d3cf5b9d7ec82979fea8926a77b77b9b0305c3f53aeb063f4bb2ff191",
  "funk-native-default": "06cc8a97014cba8732ea56940f0bcd7768db6077327a9146d0c6f6f54523b367",
  "chip-native-default": "5fc1288710fda37a71cd0c455139e3ea542f28ef56a52a921656c7456be70b71",
  "fusion-native-default": "cd34aa9d3bad72e7c11f8e43c94350b5acd9e7d1b8a568cc0c7d6fa366db3756",
  "neon-native-default": "ea685a5b3066dc0a2c68f159766ce14b664b6df3d903d7b4a020be27117490bf",
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

describe("Racing Original material is frozen at baseline e906504", () => {
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
        `${recipe.name} original material drifted from baseline e906504`,
      );
    }
  });

  it("keeps the six original Extended sections identical to the frozen Original material", () => {
    for (const recipe of CASES) {
      const extended = generate(recipe, "extended");
      assert.equal(
        digest(canonicalMusic(extended, ORIGINAL_ORDER)),
        FROZEN[recipe.name],
        `${recipe.name} Extended original sections drifted from baseline e906504`,
      );
    }
  });
});
