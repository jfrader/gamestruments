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

interface GenerationExports {
  memory: WebAssembly.Memory;
  gamestruments_reset(): void;
  gamestruments_alloc(size: number): number;
  gamestruments_output_len(): number;
  gamestruments_score_json(ptr: number, length: number): number;
  gamestruments_root_pitch_class(ptr: number, length: number): number;
}

function write(instance: WebAssembly.Instance, input: unknown): { exports: GenerationExports; ptr: number; length: number } {
  const exports = instance.exports as unknown as GenerationExports;
  const bytes = encoder.encode(JSON.stringify(input));
  exports.gamestruments_reset();
  const ptr = exports.gamestruments_alloc(bytes.length);
  new Uint8Array(exports.memory.buffer).set(bytes, ptr);
  return { exports, ptr, length: bytes.length };
}

function suspenseScore(instance: WebAssembly.Instance): PortableScore {
  const { exports, ptr, length } = write(instance, {
    recipe: "suspense", secret: "", seed: "level-001", style: "terminal",
    tension: 0.62, heat: 0.48, mystery: 0.72, pulse: 0.55,
  });
  const output = exports.gamestruments_score_json(ptr, length);
  const outputLength = exports.gamestruments_output_len();
  return JSON.parse(decoder.decode(new Uint8Array(exports.memory.buffer, output, outputLength))) as PortableScore;
}

function rootPitchClass(instance: WebAssembly.Instance, input: unknown): number {
  const { exports, ptr, length } = write(instance, input);
  return exports.gamestruments_root_pitch_class(ptr, length);
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

  it("moves to the section a game state selects, and can name it without moving", async () => {
    const instance = await instantiate();
    const score = suspenseScore(instance);
    const player = new WasmPlayer(instance, 48000);
    run(player, { load: { score, seed: "level-001", recipe: "suspense", rootPitchClass: 0, openingSection: null } });
    player.fill(512);
    const alarm = { trace: { phase: "alert", heat: 0.9, focus: 0.5, progress: 0 } };
    const section = run(player, { sectionFor: alarm }) as string;
    assert.ok(score.sections.some((candidate) => candidate.id === section), `selects a real section: ${section}`);
    assert.notEqual(section, score.defaultSection);
    assert.equal((run(player, "status") as Status).transition, null, "naming the section does not move");
    assert.deepEqual(run(player, { update: alarm }), { accepted: true });
    assert.equal((run(player, "status") as Status).transition?.to, section);
  });

  it("knows the key a Racing seed sounds in, so a new seed can join it", async () => {
    const instance = await instantiate();
    const roots = ["level-001", "level-002", "level-003", "level-004"].map((seed) =>
      rootPitchClass(instance, { recipe: "racing", secret: "", seed, style: "funk" }),
    );
    for (const root of roots) assert.ok(Number.isInteger(root) && root >= 0 && root < 12, `root ${root}`);
    assert.ok(new Set(roots).size > 1, "different seeds land in different keys");
    assert.equal(rootPitchClass(instance, { recipe: "suspense", secret: "", seed: "a", style: "terminal" }), 0);
    assert.equal(rootPitchClass(instance, { recipe: "racing", secret: "", seed: "a", style: "polka" }), 0);
  });

  it("answers an unknown section or a malformed command with an error", async () => {
    const instance = await instantiate();
    const player = new WasmPlayer(instance, 48000);
    run(player, { load: { score: suspenseScore(instance), seed: "s", recipe: "suspense", rootPitchClass: 0, openingSection: null } });
    assert.throws(() => run(player, { cue: { section: "nowhere" } }), /unknown score section: nowhere/);
    assert.throws(() => run(player, { dance: true }), /player command must parse/);
  });
});
