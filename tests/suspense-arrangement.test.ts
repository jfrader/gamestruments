import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import { validatePortableScore, type PortableScore } from "../packages/runtime/src/index.ts";

// The retired Original/Extended/Theme presets are gone. A Suspense score is now
// always the phase pool: `all-phases` or the seeded composer. Every legacy name
// (`""`, `original`, `extended`, `theme`) resolves to the seeded default, which
// these digests freeze byte-for-byte.
const seededDigests = {
  terminal: "945dbb08d3807331ffc2ec6ac1535731907b364277103e399c7fac7874dbf7c6",
  cipher: "57178ea1162261e0b0c14c5e81792fd5f6f47566c7158572b21fe2cf804f4f2f",
  noir: "436f44746f095b925bd49791046cd37571ea111cf463b73d20e7b12ba1353ab5",
};

const { instance } = await WebAssembly.instantiate(new Uint8Array(await readFile(
  new URL("../apps/demo/public/engine/gamestruments_engine.wasm", import.meta.url),
)));
const exports = instance.exports;
const memory = exports.memory;
assert.ok(memory instanceof WebAssembly.Memory);

function generate(style: string, arrangement?: string): { bytes: Uint8Array; score: PortableScore } {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    recipe: "suspense", secret: "", seed: "level-001", style,
    tension: 0.62, heat: 0.48, mystery: 0.72, pulse: 0.55,
    ...(arrangement === undefined ? {} : { arrangement }),
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

describe("Suspense arrangements through the shipped WASM", () => {
  for (const [style, digest] of Object.entries(seededDigests)) {
    it(`${style} freezes the seeded pool default`, () => {
      const baseline = generate(style);
      assert.equal(createHash("sha256").update(baseline.bytes).digest("hex"), digest);
      assert.match(baseline.score.id, /-seeded-arc$/);
      assert.deepEqual(generate(style, "seeded").bytes, baseline.bytes);
    });

    it(`${style} resolves every retired preset name to the pool default`, () => {
      const baseline = generate(style).bytes;
      for (const legacy of ["", "original", "extended", "theme"]) {
        assert.deepEqual(generate(style, legacy).bytes, baseline, `${style} ${legacy}`);
      }
    });
  }

  it("rejects an unknown arrangement instead of silently losing the selection", () => {
    assert.throws(() => generate("terminal", "missing"), /Unknown suspense arrangement/);
    assert.throws(() => generate("terminal", "flow"), /Unknown suspense arrangement/);
    assert.throws(() => generate("terminal", "featured"), /Unknown suspense arrangement/);
  });
});

function generateWith(overrides: Record<string, unknown>): { bytes: Uint8Array; score: PortableScore } {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    recipe: "suspense", secret: "", seed: "level-001", style: "terminal",
    tension: 0.62, heat: 0.48, mystery: 0.72, pulse: 0.55,
    ...overrides,
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

describe("Suspense pool arrangements through the shipped WASM", () => {
  it("all-phases plays the full pool once and loops from the first groove", () => {
    const { score } = generateWith({ arrangement: "all-phases" });
    assert.equal(score.sections.length, 27);
    const steps = score.form!.steps;
    assert.equal(steps.length, 27);
    assert.equal(new Set(steps.map((step) => step.section)).size, 27);
    assert.equal(steps[score.form!.loopFrom!]!.section, "verse");
    assert.match(score.id, /-all-phases$/);
  });

  it("seeded honours the reelIndex and stays deterministic per take", () => {
    const base = generateWith({ arrangement: "seeded", intent: "arc" }).score;
    const take7 = generateWith({ arrangement: "seeded", intent: "arc", reelIndex: 7 }).score;
    assert.deepEqual(
      generateWith({ arrangement: "seeded", intent: "arc", reelIndex: 7 }).score,
      take7,
    );
    assert.notDeepEqual(base.form, take7.form);
    assert.notDeepEqual(
      generateWith({ arrangement: "seeded", intent: "arc", reelIndex: 6 }).score.form,
      take7.form,
    );
    // A retired preset name is the same pool default, so the reel still moves it.
    assert.notDeepEqual(
      generateWith({ arrangement: "original", reelIndex: 7 }).score.form,
      generateWith({ arrangement: "extended" }).score.form,
    );
    assert.deepEqual(
      generateWith({ arrangement: "original" }).score,
      generateWith({ arrangement: "theme" }).score,
    );
  });

  it("seeded is deterministic and honours the intent field", () => {
    const arc = generateWith({ arrangement: "seeded", intent: "arc" }).score;
    assert.deepEqual(generateWith({ arrangement: "seeded", intent: "arc" }).score, arc);
    const loop = generateWith({ arrangement: "seeded", intent: "loop" }).score;
    assert.ok(loop.form!.steps.length >= 5 && loop.form!.steps.length <= 7);
    const long = generateWith({ arrangement: "seeded", intent: "long" }).score;
    assert.ok(long.form!.steps.length >= 11 && long.form!.steps.length <= 14);
    assert.notDeepEqual(loop.form, arc.form);
    assert.match(arc.id, /-seeded-arc$/);
  });

  it("rejects an unknown intent instead of silently defaulting", () => {
    assert.throws(() => generateWith({ arrangement: "seeded", intent: "verse" }), /Unknown suspense intent/);
  });
});
