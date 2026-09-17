import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import {
  AdaptiveTransport,
  validatePortableScore,
  type PortableScore,
  type SongForm,
  type TransitionPlan,
} from "../packages/runtime/src/index.ts";
import { playbackSectionOnScore } from "../apps/demo/src/playback-section.ts";

const EXTENDED_ORDER = [
  "garage",
  "ignition",
  "grid",
  "cruise",
  "slipstream",
  "attack",
  "redline",
  "final-lap",
  "victory",
  "cooldown",
] as const;

const ORIGINAL_ORDER = ["garage", "grid", "cruise", "attack", "final-lap", "victory"] as const;

const { instance } = await WebAssembly.instantiate(new Uint8Array(await readFile(
  new URL("../apps/demo/public/engine/gamestruments_engine.wasm", import.meta.url),
)));
const exports = instance.exports;
const memory = exports.memory;
assert.ok(memory instanceof WebAssembly.Memory);

interface RacingRequest {
  recipe?: string;
  style: string;
  arrangement?: string;
  autoplay?: boolean;
  seed?: string;
}

function generate(request: RacingRequest): { bytes: Uint8Array; score: PortableScore } {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    secret: "",
    seed: "racing-arrangement",
    energy: 0.58,
    complexity: 0.62,
    brightness: 0.56,
    syncopation: 0.7,
    palette: { melody: "", harmony: "", drive: "", bass: "" },
    ...request,
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
  return { bytes, score };
}

function formOf(score: PortableScore): SongForm {
  assert.ok(score.form, "autoplay score must have a form");
  return score.form;
}

function completeTransition(transport: AdaptiveTransport, boundary: number): TransitionPlan {
  const plan = transport.advance(boundary);
  assert.ok(plan, `expected an automatic transition at tick ${boundary}`);
  transport.advance(plan.endTick);
  return plan;
}

describe("Racing arrangements through the shipped WASM", () => {
  it("defaults omitted, explicit original, and disabled autoplay to the same unchanged bytes", () => {
    const omitted = generate({ recipe: "racing", style: "funk" });
    const explicit = generate({ recipe: "racing", style: "funk", arrangement: "original" });
    const disabled = generate({ recipe: "racing", style: "funk", autoplay: false });

    assert.deepEqual(explicit.bytes, omitted.bytes);
    assert.deepEqual(disabled.bytes, omitted.bytes);
    assert.equal(omitted.score.form, undefined);
    assert.doesNotMatch(omitted.score.id, /autoplay/);
    assert.doesNotMatch(omitted.score.id, /extended/);
  });

  it("Extended keeps the six original sections byte-identical and adds four new phases", () => {
    const original = generate({ recipe: "racing", style: "funk" }).score;
    const extended = generate({ recipe: "racing", style: "funk", arrangement: "extended" }).score;

    assert.equal(extended.form, undefined);
    assert.equal(extended.sections.length, 10);
    assert.deepEqual(extended.sections.map((section) => section.id), [...EXTENDED_ORDER]);
    for (const id of ORIGINAL_ORDER) {
      assert.deepEqual(
        extended.sections.find((section) => section.id === id),
        original.sections.find((section) => section.id === id),
        `original section ${id} must not change`,
      );
    }
    assert.match(extended.id, /-extended-v\d/);
    assert.notEqual(extended.id, original.id);
    assert.notEqual(extended.title, original.title);
  });

  it("Extended autoplay forms the ten-section tour with loopFrom grid", () => {
    const { score } = generate({ recipe: "racing", style: "funk", arrangement: "extended", autoplay: true });
    const form = formOf(score);

    assert.deepEqual(form.steps.map((step) => step.section), [...EXTENDED_ORDER]);
    assert.ok(form.steps.every((step) => (step.repeats ?? 1) === 1));
    assert.equal(form.loopFrom, 2);
    assert.equal(form.origin, "transitionStart");
    assert.match(score.id, /-extended-v\d.*-autoplay-v1$/);
  });

  it("Extended autoplay follows the authored lengths through all ten sections and loops back to grid", () => {
    const { score } = generate({ recipe: "racing", style: "funk", arrangement: "extended", autoplay: true });
    const form = formOf(score);
    const transport = new AdaptiveTransport(score);

    const garage = score.sections.find((section) => section.id === "garage")!;
    assert.equal(completeTransition(transport, garage.lengthTicks).to, "ignition");

    let boundary = garage.lengthTicks;
    for (let index = 1; index < form.steps.length; index++) {
      const step = form.steps[index]!;
      const section = score.sections.find((candidate) => candidate.id === step.section)!;
      boundary += section.lengthTicks * (step.repeats ?? 1);
      const expected = index + 1 < form.steps.length
        ? form.steps[index + 1]!.section
        : form.steps[form.loopFrom!]!.section;
      assert.equal(completeTransition(transport, boundary).to, expected);
    }
    assert.equal(form.steps[form.loopFrom!]!.section, "grid");
  });

  it("cues a new phase manually and resumes the tour from it", () => {
    const { score } = generate({ recipe: "racing", style: "funk", arrangement: "extended", autoplay: true });
    const transport = new AdaptiveTransport(score);

    const cue = transport.requestSection("slipstream", 1);
    assert.equal(cue.status, "scheduled");
    if (cue.status !== "scheduled") return;
    transport.advance(cue.plan.endTick);
    assert.equal(transport.snapshot().currentSection, "slipstream");

    const slipstream = score.sections.find((section) => section.id === "slipstream")!;
    assert.equal(
      completeTransition(transport, cue.plan.startTick + slipstream.lengthTicks).to,
      "attack",
    );
  });

  it("holds the extended tour and resumes it on the authored boundary", () => {
    const { score } = generate({ recipe: "racing", style: "funk", arrangement: "extended", autoplay: true });
    const transport = new AdaptiveTransport(score);
    const barTicks = score.beatsPerBar * score.ticksPerBeat;
    const garage = score.sections.find((section) => section.id === "garage")!;
    const ignition = score.sections.find((section) => section.id === "ignition")!;

    const toIgnition = completeTransition(transport, garage.lengthTicks);
    assert.equal(toIgnition.to, "ignition");
    assert.equal(transport.snapshot().currentSection, "ignition");

    transport.setFormHeld(true, garage.lengthTicks);
    assert.equal(transport.advance(Number.MAX_SAFE_INTEGER), null);
    assert.equal(transport.snapshot().currentSection, "ignition");

    const resumeTick = garage.lengthTicks + 2 * barTicks;
    transport.setFormHeld(false, resumeTick);
    assert.equal(transport.advance(resumeTick), null);

    const toGrid = transport.advance(garage.lengthTicks + ignition.lengthTicks);
    assert.ok(toGrid, "resume must schedule the ignition→grid boundary");
    assert.equal(toGrid.to, "grid");
  });

  it("rolling back to Original from an active new phase falls back to garage without a stale cue", () => {
    const extended = generate({ recipe: "racing", style: "funk", arrangement: "extended", autoplay: true }).score;
    const original = generate({ recipe: "racing", style: "funk", arrangement: "original", autoplay: true }).score;

    const extendedTransport = new AdaptiveTransport(extended);
    const cue = extendedTransport.requestSection("redline", 1);
    assert.equal(cue.status, "scheduled");
    if (cue.status !== "scheduled") return;
    extendedTransport.advance(cue.plan.endTick);
    assert.equal(extendedTransport.snapshot().currentSection, "redline");

    assert.equal(original.sections.some((section) => section.id === "redline"), false);
    const initial = playbackSectionOnScore(original, extendedTransport.snapshot().currentSection);
    assert.equal(initial, "garage");

    const nextTransport = new AdaptiveTransport(original, initial);
    assert.equal(nextTransport.snapshot().currentSection, "garage");
    assert.equal(nextTransport.snapshot().pendingSection, null);
  });

  it("keeps the extended arrangement across seed and style regeneration", () => {
    const funk = generate({ recipe: "racing", style: "funk", arrangement: "extended", autoplay: true }).score;
    const chip = generate({ recipe: "racing", style: "chip", arrangement: "extended", autoplay: true }).score;
    const reseeded = generate({ recipe: "racing", style: "funk", arrangement: "extended", autoplay: true, seed: "regen-seed" }).score;

    for (const score of [funk, chip, reseeded]) {
      assert.equal(score.sections.length, 10);
      assert.deepEqual(score.sections.map((section) => section.id), [...EXTENDED_ORDER]);
      assert.equal(formOf(score).loopFrom, 2);
    }
    assert.notEqual(reseeded.id, funk.id);
  });

  it("rejects an unknown racing arrangement and never treats Theme as Racing", () => {
    assert.throws(
      () => generate({ recipe: "racing", style: "funk", arrangement: "theme" }),
      /Unknown racing arrangement: theme/,
    );
    assert.throws(
      () => generate({ recipe: "racing", style: "funk", arrangement: "missing" }),
      /Unknown racing arrangement: missing/,
    );
  });

  it("Composed builds a song form over the six sections and loops to a groove", () => {
    const { score } = generate({ recipe: "racing", style: "funk", arrangement: "composed" });

    assert.match(score.id, /-composed-v\d/);
    assert.equal(score.sections.length, 6);
    const form = formOf(score);
    assert.ok(form.steps.length >= 6);
    for (const step of form.steps) {
      assert.ok(
        score.sections.some((section) => section.id === step.section),
        `composed step references unknown section ${step.section}`,
      );
    }
    assert.equal(form.steps[0]!.section, "garage");
    assert.equal(form.steps[form.steps.length - 1]!.section, "victory");
    assert.ok(form.loopFrom !== undefined, "composed form must have a loopFrom");
    assert.equal(form.steps[form.loopFrom!]!.section, "cruise");
  });

  it("Composed is deterministic and varies with the seed", () => {
    const first = generate({ recipe: "racing", style: "funk", arrangement: "composed" }).score;
    const repeat = generate({ recipe: "racing", style: "funk", arrangement: "composed" }).score;
    const reseeded = generate({
      recipe: "racing",
      style: "funk",
      arrangement: "composed",
      seed: "composed-other-seed",
    }).score;

    assert.deepEqual(first, repeat);
    assert.notEqual(reseeded.id, first.id);
    assert.deepEqual(
      reseeded.sections.map((section) => section.id),
      first.sections.map((section) => section.id),
    );
  });
});
