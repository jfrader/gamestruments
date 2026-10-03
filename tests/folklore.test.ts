import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import { LAB_RECIPE_PROFILES } from "../apps/demo/src/recipes.ts";
import { WasmPlayer } from "../apps/demo/src/wasm-player.ts";
import {
  validatePortableScore,
  type NoteEvent,
  type PortableScore,
} from "../packages/runtime/src/index.ts";

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
  const style = String(overrides.style ?? preset.style);
  if (style === "carnavalito") {
    // The tonic sits at 48 + root; the lowest pitched note is the resolution.
    const pitches = score.sections.flatMap((section) => section.events)
      .filter((event): event is NoteEvent => event.kind === "note")
      .map((event) => event.pitch);
    assert.equal(rootPitchClass, Math.min(...pitches) % 12);
  } else {
    // Chacarera: the first walking-bass note (tick 0) is the tonic.
    const tonicBass = score.sections[0]?.events.find((event) => event.kind === "note" && event.lane === "bass");
    assert.ok(tonicBass?.kind === "note");
    assert.equal(rootPitchClass, tonicBass.pitch % 12);
  }
  return { instance, bytes, score, rootPitchClass };
}

describe("Folklore listening prototype through the committed WASM", () => {
  it("retains the original six Chacarera sections from the v1.2.0 WASM", async () => {
    // SHA256 of JSON.stringify({ bpm, beatsPerBar, ticksPerBeat, sections }),
    // using the v1.2.0 release WASM; each section has id, lengthTicks, and
    // events with only their id and section metadata removed.
    const cases = [
      { secret: "", seed: "level-001", energy: 0.58, complexity: 0.47, brightness: 0.61, syncopation: 0.64, digest: "3579dbcf69cd13a48651a9f14f6697488cb5427b9b06be06e03fdb8996b79e7a" },
      { secret: "my-game", seed: "level-001", energy: 0.62, complexity: 0.6, brightness: 0.52, syncopation: 0.7, digest: "7461b26a9695d028c49fcb7ba60a8365f025e99ff5c5886c8b04b89e97afd346" },
      { secret: "guitar", seed: "stable", energy: 1, complexity: 0, brightness: 1, syncopation: 1, digest: "aafb2c80801a6694e5e9b7b2257eb5b93fbbf395f87ba24eedbc20f185dd1f0c" },
    ];
    for (const { digest, ...input } of cases) {
      const { score } = await generate(input);
      if (input.secret === "" && input.seed === "level-001") assert.notEqual(score.id, "folklore-a2931eab");
      const canonical = {
        bpm: score.bpm, beatsPerBar: score.beatsPerBar, ticksPerBeat: score.ticksPerBeat,
        sections: score.sections.slice(0, 6).map((section) => ({
          id: section.id, lengthTicks: section.lengthTicks,
          events: section.events.map(({ id: _id, section: _section, ...music }) => music),
        })),
      };
      assert.equal(createHash("sha256").update(JSON.stringify(canonical)).digest("hex"), digest, input.seed);
    }
  });

  it("is deterministic, uses its own instruments, and leaves game state out of the score", async () => {
    const { bytes, score } = await generate();
    assert.deepEqual(bytes, (await generate()).bytes);
    assert.notDeepEqual(bytes, (await generate({ seed: "level-002" })).bytes);
    assert.notDeepEqual(bytes, (await generate({ reelIndex: 1 })).bytes);
    assert.deepEqual((await generate({ reelIndex: 1 })).bytes, (await generate({ reelIndex: 1 })).bytes);
    assert.equal(score.beatsPerBar, 3);
    assert.deepEqual(score.sections.map((section) => section.id), [
      "introduccion", "primera", "interludio", "segunda", "estribillo", "cierre",
      "punteo", "respiro", "pena",
    ]);
    const voices = new Set(score.sections.flatMap((section) => section.events
      .filter((event) => event.kind !== "stem")
      .map((event) => event.voice)));
    for (const voice of ["nylon-guitar", "bombo", "bombo-rim"] as const) assert.ok(voices.has(voice), voice);
    assert.deepEqual(score.rules, []);
    assert.ok(score.form);
    const form = score.form.steps.map((step) => step.section);
    for (const section of ["punteo", "respiro", "pena"] as const) assert.ok(form.includes(section), section);
  });

  it("tours each section once in All phases and rejects unsupported choices", async () => {
    const { score } = await generate({ arrangement: "all-phases" });
    assert.deepEqual(score.form?.steps.map((step) => step.section), score.sections.map((section) => section.id));
    await assert.rejects(generate({ style: "folk" }), /style/i);
    await assert.rejects(generate({ style: "zamba" }), /style/i);
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

  it("composes a binary, pentatonic Carnavalito with charango and quena voices", async () => {
    const { bytes, score } = await generate({ style: "carnavalito" });
    assert.deepEqual(bytes, (await generate({ style: "carnavalito" })).bytes);
    assert.notDeepEqual(bytes, (await generate({ style: "carnavalito", seed: "level-002" })).bytes);
    assert.equal(score.beatsPerBar, 2);
    assert.deepEqual(score.sections.map((section) => section.id), [
      "preludio", "copla", "respuesta", "estribillo", "cierre",
    ]);
    const voices = new Set(score.sections.flatMap((section) => section.events)
      .filter((event) => event.kind !== "stem")
      .map((event) => event.voice));
    for (const voice of ["charango", "quena", "bombo"] as const) assert.ok(voices.has(voice), voice);
    assert.ok(!voices.has("nylon-guitar"));
    assert.deepEqual(score.rules, []);
    assert.ok(score.form);
  });

  it("tours each Carnavalito section once in All phases", async () => {
    const { score } = await generate({ style: "carnavalito", arrangement: "all-phases" });
    assert.deepEqual(score.form?.steps.map((step) => step.section), score.sections.map((section) => section.id));
    await assert.rejects(generate({ style: "carnavalito", arrangement: "extended" }), /arrangement/i);
  });
});
