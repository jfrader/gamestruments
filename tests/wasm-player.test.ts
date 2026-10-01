import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import type { PortableScore } from "../packages/runtime/src/index.ts";
import { WasmPlayer } from "../apps/demo/src/wasm-player.ts";

const wasm = new Uint8Array(await readFile(
  new URL("../apps/demo/public/engine/gamestruments_engine.wasm", import.meta.url),
));
const encoder = new TextEncoder();
const decoder = new TextDecoder();

async function instantiate(): Promise<WebAssembly.Instance> {
  return (await WebAssembly.instantiate(wasm)).instance;
}

function suspenseScore(instance: WebAssembly.Instance): PortableScore {
  const exports = instance.exports as Record<string, CallableFunction> & { memory: WebAssembly.Memory };
  const input = encoder.encode(JSON.stringify({
    recipe: "suspense", secret: "", seed: "level-001", style: "terminal",
    tension: 0.62, heat: 0.48, mystery: 0.72, pulse: 0.55,
  }));
  exports.gamestruments_reset();
  const ptr = exports.gamestruments_alloc(input.length) as number;
  new Uint8Array(exports.memory.buffer).set(input, ptr);
  const output = exports.gamestruments_score_json(ptr, input.length) as number;
  const length = exports.gamestruments_output_len() as number;
  return JSON.parse(decoder.decode(new Uint8Array(exports.memory.buffer, output, length))) as PortableScore;
}

function run(player: WasmPlayer, command: unknown): unknown {
  const response = player.command(encoder.encode(JSON.stringify(command)));
  const text = decoder.decode(response.bytes);
  if (!response.ok) throw new Error(text);
  return JSON.parse(text);
}

interface Status {
  scoreId: string;
  currentSection: string;
  transition: { to: string } | null;
  mix: { section: string; gain: number; origin: number }[];
}

describe("the live player through the shipped WASM", () => {
  it("plays a loaded score and reports what it sounds", async () => {
    const instance = await instantiate();
    const score = suspenseScore(instance);
    const player = new WasmPlayer(instance, 48000);
    assert.equal(run(player, "status"), null);
    assert.deepEqual(
      run(player, { load: { score, seed: "level-001", recipe: "suspense", rootPitchClass: 0, openingSection: null } }),
      { accepted: true },
    );
    let peak = 0;
    for (let block = 0; block < 100; block++) {
      for (const sample of player.fill(512)) peak = Math.max(peak, Math.abs(sample));
    }
    assert.ok(peak > 1e-3, "the player renders sound");
    const status = run(player, "status") as Status;
    assert.equal(status.scoreId, score.id);
    assert.equal(status.currentSection, score.defaultSection);
    assert.deepEqual(status.mix, [{ section: score.defaultSection, gain: 1, origin: 0 }]);
  });

  it("cues a section and cancels the cue before it starts", async () => {
    const instance = await instantiate();
    const score = suspenseScore(instance);
    const target = score.sections.find((section) => section.id !== score.defaultSection)!.id;
    const player = new WasmPlayer(instance, 48000);
    run(player, { load: { score, seed: "level-001", recipe: "suspense", rootPitchClass: 0, openingSection: null } });
    player.fill(512);
    assert.deepEqual(run(player, { cue: { section: target } }), { accepted: true });
    assert.equal((run(player, "status") as Status).transition?.to, target);
    assert.deepEqual(run(player, "cancel"), { accepted: true });
    assert.equal((run(player, "status") as Status).transition, null);
  });

  it("answers an unknown section or a malformed command with an error", async () => {
    const instance = await instantiate();
    const player = new WasmPlayer(instance, 48000);
    run(player, { load: { score: suspenseScore(instance), seed: "s", recipe: "suspense", rootPitchClass: 0, openingSection: null } });
    assert.throws(() => run(player, { cue: { section: "nowhere" } }), /unknown score section: nowhere/);
    assert.throws(() => run(player, { dance: true }), /player command must parse/);
  });
});
