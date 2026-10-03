import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import { LAB_RECIPE_PROFILES } from "../apps/demo/src/recipes.ts";
import { WasmPlayer } from "../apps/demo/src/wasm-player.ts";
import { validatePortableScore, type PortableScore } from "../packages/runtime/src/index.ts";

const wasm = new Uint8Array(await readFile(
  new URL("../apps/demo/public/engine/gamestruments_engine.wasm", import.meta.url),
));
const encoder = new TextEncoder();
const decoder = new TextDecoder();
const preset = LAB_RECIPE_PROFILES.folklore.presets[0]!;

interface GenerationExports {
  memory: WebAssembly.Memory;
  gamestruments_reset(): void;
  gamestruments_alloc(size: number): number;
  gamestruments_score_json(ptr: number, length: number): number;
  gamestruments_root_pitch_class(ptr: number, length: number): number;
  gamestruments_output_len(): number;
  gamestruments_status(): number;
}

async function generate(overrides: Record<string, unknown> = {}) {
  const { instance } = await WebAssembly.instantiate(wasm);
  const exports = instance.exports as unknown as GenerationExports;
  const input = encoder.encode(JSON.stringify({
    recipe: "folklore", style: preset.style, secret: "", seed: "level-001",
    arrangement: "seeded", ...preset.traits, ...overrides,
  }));
  exports.gamestruments_reset();
  const ptr = exports.gamestruments_alloc(input.length);
  assert.notEqual(ptr, 0);
  new Uint8Array(exports.memory.buffer).set(input, ptr);
  const rootPitchClass = exports.gamestruments_root_pitch_class(ptr, input.length);
  const output = exports.gamestruments_score_json(ptr, input.length);
  const bytes = new Uint8Array(exports.memory.buffer, output, exports.gamestruments_output_len()).slice();
  const text = decoder.decode(bytes);
  if (exports.gamestruments_status() !== 0) throw new Error(text);
  const score = JSON.parse(text) as PortableScore;
  validatePortableScore(score);
  const tonicBass = score.sections[0]?.events.find((event) => event.kind === "note" && event.lane === "bass");
  assert.ok(tonicBass?.kind === "note");
  assert.equal(rootPitchClass, tonicBass.pitch % 12);
  return { instance, bytes, score, rootPitchClass };
}

describe("Folklore listening prototype through the committed WASM", () => {
  it("is deterministic, uses its own instruments, and leaves game state out of the score", async () => {
    const { bytes, score } = await generate();
    assert.deepEqual(bytes, (await generate()).bytes);
    assert.notDeepEqual(bytes, (await generate({ seed: "level-002" })).bytes);
    assert.notDeepEqual(bytes, (await generate({ reelIndex: 1 })).bytes);
    assert.deepEqual((await generate({ reelIndex: 1 })).bytes, (await generate({ reelIndex: 1 })).bytes);
    assert.equal(score.beatsPerBar, 3);
    assert.deepEqual(score.sections.map((section) => section.id), [
      "introduccion", "primera", "interludio", "segunda", "estribillo", "cierre",
    ]);
    const voices = new Set(score.sections.flatMap((section) => section.events
      .filter((event) => event.kind !== "stem")
      .map((event) => event.voice)));
    for (const voice of ["nylon-guitar", "bombo", "bombo-rim"] as const) assert.ok(voices.has(voice), voice);
    assert.deepEqual(score.rules, []);
    assert.ok(score.form);
    assert.ok(score.form.steps.length > score.sections.length);
  });

  it("tours each section once in All phases and rejects unsupported choices", async () => {
    const { score } = await generate({ arrangement: "all-phases" });
    assert.deepEqual(score.form?.steps.map((step) => step.section), score.sections.map((section) => section.id));
    await assert.rejects(generate({ style: "folk" }), /style/i);
    await assert.rejects(generate({ arrangement: "extended" }), /arrangement/i);
  });

  it("renders finite audio and accepts a section cue through the existing live player", async () => {
    const { instance, score, rootPitchClass } = await generate();
    const player = new WasmPlayer(instance, 48000);
    const command = (value: unknown) => {
      const response = player.command(encoder.encode(JSON.stringify(value)));
      const text = decoder.decode(response.bytes);
      assert.equal(response.ok, true, text);
      return JSON.parse(text);
    };
    command({ load: { score, seed: "level-001", recipe: "folklore", rootPitchClass, openingSection: null } });
    let peak = 0;
    for (let block = 0; block < 120; block++) {
      for (const sample of player.fill(512)) {
        assert.ok(Number.isFinite(sample));
        peak = Math.max(peak, Math.abs(sample));
      }
    }
    assert.ok(peak > 0.001 && peak <= 1, `audio peak ${peak}`);
    assert.deepEqual(command({ cue: { section: "estribillo" } }), { accepted: true });
    assert.equal(command("status").transition?.to, "estribillo");
  });
});
