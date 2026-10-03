import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import { WasmPlayer } from "../apps/demo/src/wasm-player.ts";
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

const ORIGINAL_SECTIONS = [
  "camp", "explore", "town", "festival", "reunion", "dungeon", "skirmish",
  "combat", "chase", "boss", "assault", "sanctuary", "dawn", "victory",
];
const VARIANTS = ["explore-strings", "town-strings"];
const ORIGINAL_DIGESTS: Readonly<Record<string, string>> = {
  folk: "60e0fc8ca34a55258afa7b7264c21f741d1e69dd81180e2fdaae173ed034b537",
  dark: "933418cb4bb3e39a5116f84450a000b55fbbbcd9ad6d7fe78cf267c0d5e581e7",
  orchestral: "e73a9843a0c5a1397f8e2e42be754d5d6dd75b29e01aa3cd4b08753d012ebabf",
};

function generate(
  style: string,
  autoplay?: boolean,
  options: { seed?: string; arrangement?: "original" | "all-phases" | "seeded" } = {},
): { bytes: Uint8Array; score: PortableScore; rootPitchClass: number } {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    recipe: "adventure",
    secret: "",
    seed: options.seed ?? "trail-001",
    style,
    arrangement: options.arrangement,
    ...(autoplay === undefined ? {} : { autoplay }),
    energy: 0.5,
    complexity: 0.45,
    brightness: 0.68,
    syncopation: 0.5,
  }));
  (exports.gamestruments_reset as () => void)();
  const ptr = (exports.gamestruments_alloc as (length: number) => number)(input.length);
  new Uint8Array(memory.buffer).set(input, ptr);
  const rootPitchClass = (exports.gamestruments_root_pitch_class as (ptr: number, length: number) => number)(ptr, input.length);
  const result = (exports.gamestruments_score_json as (ptr: number, length: number) => number)(ptr, input.length);
  const length = (exports.gamestruments_output_len as () => number)();
  const bytes = new Uint8Array(memory.buffer, result, length).slice();
  const text = new TextDecoder().decode(bytes);
  if ((exports.gamestruments_status as () => number)() !== 0) {
    throw new Error(text);
  }
  const score = JSON.parse(text) as PortableScore;
  validatePortableScore(score);
  return { bytes, score, rootPitchClass };
}

describe("Adventure recipe through the shipped WASM", () => {
  it("keeps game-mode generation state-driven and gives every quest phase room to develop", () => {
    const { score } = generate("folk");
    assert.deepEqual(new Set(score.sections.map((section) => section.id)), new Set([...ORIGINAL_SECTIONS, ...VARIANTS]));
    assert.equal(score.sections.length, ORIGINAL_SECTIONS.length + VARIANTS.length);
    assert.equal(score.defaultSection, "camp");
    assert.equal(score.form, undefined);
    const barTicks = score.beatsPerBar * score.ticksPerBeat;
    for (const section of score.sections) {
      const bars = section.lengthTicks / barTicks;
      assert.equal(Number.isInteger(bars), true, `${section.id} must end on a bar`);
      assert.equal(bars % 8, 0, `${section.id} must contain complete movements`);
      assert.ok(bars >= 16, `${section.id} should have at least two movements`);
      assert.ok(section.events.length > 8, `${section.id} should carry events`);
    }
  });

  it("preserves the fourteen original sections and game rules while adding guitar variants", () => {
    for (const style of Object.keys(ORIGINAL_DIGESTS)) {
      const { score } = generate(style);
      const original = {
        bpm: score.bpm,
        beatsPerBar: score.beatsPerBar,
        ticksPerBeat: score.ticksPerBeat,
        crossfadeBars: score.crossfadeBars,
        defaultSection: score.defaultSection,
        sections: ORIGINAL_SECTIONS.map((id) => score.sections.find((section) => section.id === id)),
        rules: score.rules,
      };
      assert.equal(createHash("sha256").update(JSON.stringify(original)).digest("hex"), ORIGINAL_DIGESTS[style], style);
      for (const id of VARIANTS) {
        const variant = score.sections.find((section) => section.id === id);
        assert.ok(variant, id);
        const voices = new Set(variant.events.filter((event) => event.kind !== "stem").map((event) => event.voice));
        assert.ok(voices.has("nylon-guitar"), `${style}/${id} has guitar`);
        assert.ok(voices.has("bombo"), `${style}/${id} has bombo skin`);
        assert.ok(voices.has("bombo-rim"), `${style}/${id} has rim accents`);
        assert.ok(variant.events.some((event) => event.kind === "note" && event.role === "melody"));
        assert.equal(score.rules.some((rule) => rule.target === id), false, "variants do not add game states");
      }
    }
  });

  it("carries both variants in every arrangement and cues them without replacing Explore or Town", () => {
    for (const arrangement of ["original", "all-phases", "seeded"] as const) {
      const { score } = generate("folk", undefined, { arrangement });
      for (const id of VARIANTS) {
        assert.ok(score.sections.some((section) => section.id === id));
        const cue = new AdaptiveTransport(score).requestSection(id, 0);
        assert.equal(cue.status, "scheduled");
        if (cue.status === "scheduled") assert.equal(cue.plan.to, id);
      }
      if (arrangement === "all-phases") {
        assert.deepEqual(score.form?.steps.map((step) => step.section), score.sections.map((section) => section.id));
      }
    }
    const first = generate("folk", undefined, { seed: "variant-a" }).score;
    const second = generate("folk", undefined, { seed: "variant-b" }).score;
    for (const id of VARIANTS) {
      assert.notDeepEqual(first.sections.find((section) => section.id === id)?.events, second.sections.find((section) => section.id === id)?.events);
    }
  });

  it("is deterministic and gives all three styles distinct voices", () => {
    const first = generate("folk");
    const repeated = generate("folk");
    assert.deepEqual(first.bytes, repeated.bytes);
    const scores = [first.score, generate("dark").score, generate("orchestral").score];
    assert.equal(new Set(scores.map((score) => score.id)).size, 3);
    assert.equal(new Set(scores.map((score) => score.title)).size, 3);
    const voiceArrangements = scores.map((score) => score.sections.map((section) =>
      section.events
        .filter((event) => event.kind === "note" || event.kind === "percussion")
        .map((event) => `${event.lane}:${event.voice}`)
        .join("|"),
    ).join("/"));
    assert.equal(new Set(voiceArrangements).size, 3);
  });

  it("plays both guitar variants through the live player in every style", () => {
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    const sampleRate = 48000;
    for (const style of Object.keys(ORIGINAL_DIGESTS)) {
      const { score, rootPitchClass } = generate(style);
      for (const openingSection of VARIANTS) {
        const player = new WasmPlayer(instance, sampleRate);
        const response = player.command(encoder.encode(JSON.stringify({
          load: { score, rootPitchClass, seed: "trail-001", recipe: "adventure", openingSection },
        })));
        assert.ok(response.ok, decoder.decode(response.bytes));
        let peak = 0;
        for (let frame = 0; frame < sampleRate * 2; frame += 256) {
          for (const sample of player.fill(256)) {
            assert.ok(Number.isFinite(sample), `${style}/${openingSection}`);
            peak = Math.max(peak, Math.abs(sample));
          }
        }
        assert.ok(peak > 0.001 && peak <= 1, `${style}/${openingSection} peak ${peak}`);
      }
    }
  });

  it("maps area phase, discovery, threat, and quest progress to sections", () => {
    const { score } = generate("dark");
    const request = (state: GameState) => new AdaptiveTransport(score).requestState(state, 0);
    const phase = (areaPhase: string) => ({ numeric: {}, categorical: { areaPhase } });

    assert.equal(request(phase("camp")).status, "unchanged");
    for (const id of ["explore", "town", "dungeon", "boss", "sanctuary", "victory"]) {
      const result = request(phase(id));
      assert.equal(result.status, "scheduled", `${id} should schedule`);
      if (result.status === "scheduled") assert.equal(result.plan.to, id);
    }

    const sanctuary = request({ numeric: { discovery: 0.9 }, categorical: {} });
    assert.equal(sanctuary.status, "scheduled");
    if (sanctuary.status === "scheduled") assert.equal(sanctuary.plan.to, "sanctuary");

    const combat = request({ numeric: { threat: 0.8 }, categorical: {} });
    assert.equal(combat.status, "scheduled");
    if (combat.status === "scheduled") assert.equal(combat.plan.to, "combat");

    const boss = request({ numeric: { threat: 0.9 }, categorical: { areaPhase: "combat" } });
    assert.equal(boss.status, "scheduled");
    if (boss.status === "scheduled") assert.equal(boss.plan.to, "boss");

    const complete = request({ numeric: { questComplete: 1 }, categorical: {} });
    assert.equal(complete.status, "scheduled");
    if (complete.status === "scheduled") assert.equal(complete.plan.to, "victory");
  });

  it("supersedes an in-flight manual form step with quest_complete and drops queued cues", () => {
    const { score } = generate("folk", true);
    const bar = score.beatsPerBar * score.ticksPerBeat;
    const transport = new AdaptiveTransport(score);
    assert.equal(transport.snapshot().currentSection, "camp");

    // A manual form step commits at the next bar boundary.
    const step = transport.advanceForm(bar / 2);
    assert.equal(step.status, "scheduled");
    if (step.status !== "scheduled") return;
    assert.equal(step.plan.to, "explore");
    assert.equal(step.plan.startTick, bar);

    // The form step is now in flight: past its start tick but not complete.
    transport.advance(step.plan.startTick);
    assert.equal(transport.snapshot().currentSection, "camp");

    // A direct cue during the in-flight form step defers (queues) behind it.
    const cue = transport.requestSection("town", step.plan.startTick);
    assert.equal(cue.status, "queued");

    // quest_complete arrives mid-transition and must supersede the stale form step.
    const victory = transport.requestState({ numeric: { questComplete: 1 }, categorical: {} }, step.plan.startTick);
    assert.equal(victory.status, "scheduled");
    if (victory.status !== "scheduled") return;
    assert.equal(victory.plan.to, "victory");

    // The superseded transition reaches victory.
    transport.advance(victory.plan.endTick);
    assert.equal(transport.snapshot().currentSection, "victory");
    assert.equal(transport.snapshot().transition, null);
    assert.equal(transport.snapshot().pendingSection, null);
  });

  it("supersedes an in-flight automatic form transition with quest_complete", () => {
    const { score } = generate("folk", true);
    const campLen = score.sections.find(s => s.id === "camp")!.lengthTicks;
    const transport = new AdaptiveTransport(score);
    assert.equal(transport.snapshot().currentSection, "camp");

    // Let the form auto-advance camp -> explore.
    transport.advance(campLen);
    const automatic = transport.snapshot().transition;
    assert.ok(automatic);
    assert.equal(automatic!.to, "explore");

    // quest_complete mid-flight must supersede the automatic progression.
    const victory = transport.requestState({ numeric: { questComplete: 1 }, categorical: {} }, automatic!.startTick + 1);
    assert.equal(victory.status, "scheduled");
    if (victory.status !== "scheduled") return;
    assert.equal(victory.plan.to, "victory");

    transport.advance(victory.plan.endTick);
    assert.equal(transport.snapshot().currentSection, "victory");
  });

  it("a held cue still defers to the active cue after the supersede change", () => {
    const { score } = generate("folk", true);
    const transport = new AdaptiveTransport(score);

    const cue = transport.requestSection("explore", 0);
    assert.equal(cue.status, "scheduled");
    if (cue.status !== "scheduled") return;

    transport.advance(cue.plan.startTick);

    // quest_complete during an explicit held cue defers (queues) behind it.
    const victory = transport.requestState({ numeric: { questComplete: 1 }, categorical: {} }, cue.plan.startTick + 1);
    assert.equal(victory.status, "queued");
    assert.equal(transport.snapshot().currentSection, "camp");
  });
});
