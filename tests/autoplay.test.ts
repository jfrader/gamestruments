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

const { instance } = await WebAssembly.instantiate(new Uint8Array(await readFile(
  new URL("../apps/demo/public/engine/gamestruments_engine.wasm", import.meta.url),
)));
const exports = instance.exports;
const memory = exports.memory;
assert.ok(memory instanceof WebAssembly.Memory);

interface GenerationRequest {
  recipe?: string;
  style: string;
  autoplay?: boolean;
}

function generate(request: GenerationRequest): { bytes: Uint8Array; score: PortableScore } {
  assert.ok(memory instanceof WebAssembly.Memory);
  const input = new TextEncoder().encode(JSON.stringify({
    secret: "",
    seed: "autoplay-tour",
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

describe("automatic recipe arrangements through the shipped WASM", () => {
  it("defaults game callers to unchanged state-driven racing output", () => {
    const omitted = generate({ recipe: "racing", style: "funk" });
    const disabled = generate({ recipe: "racing", style: "funk", autoplay: false });

    assert.equal(omitted.score.form, undefined);
    assert.equal(disabled.score.form, undefined);
    assert.deepEqual(disabled.bytes, omitted.bytes);
    assert.doesNotMatch(omitted.score.id, /autoplay/);
  });

  it("follows the Racing tour at the first authored boundary and through a full cycle", () => {
    const { score } = generate({ recipe: "racing", style: "funk", autoplay: true });
    const form = formOf(score);
    assert.deepEqual(form.steps.map(({ section, repeats = 1 }) => [section, repeats]), [
      ["garage", 1],
      ["grid", 1],
      ["cruise", 2],
      ["attack", 1],
      ["final-lap", 1],
      ["victory", 1],
    ]);
    assert.equal(form.loopFrom, 1);
    assert.equal(form.origin, "transitionStart");
    assert.match(score.id, /-autoplay-v1$/);

    const transport = new AdaptiveTransport(score);
    let boundary = score.sections.find((section) => section.id === "garage")!.lengthTicks;
    assert.equal(completeTransition(transport, boundary).to, "grid");
    for (let index = 1; index < form.steps.length; index++) {
      const step = form.steps[index]!;
      const section = score.sections.find((candidate) => candidate.id === step.section)!;
      boundary += section.lengthTicks * (step.repeats ?? 1);
      const expected: string = index + 1 < form.steps.length
        ? form.steps[index + 1]!.section
        : form.steps[form.loopFrom!]!.section;
      assert.equal(completeTransition(transport, boundary).to, expected);
    }
  });

  it("resumes automatic progress from a manual cue and respects hold across regeneration", () => {
    const first = generate({ recipe: "adventure", style: "folk", autoplay: true }).score;
    const transport = new AdaptiveTransport(first);
    const cue = transport.requestSection("combat", 1);
    assert.equal(cue.status, "scheduled");
    if (cue.status !== "scheduled") return;
    transport.advance(cue.plan.endTick);
    assert.equal(transport.snapshot().currentSection, "combat");
    const combat = first.sections.find((section) => section.id === "combat")!;
    const bossPlan = completeTransition(transport, cue.plan.startTick + combat.lengthTicks);
    assert.equal(bossPlan.to, "boss");

    transport.setFormHeld(true, bossPlan.endTick);
    const heldSection = transport.snapshot().currentSection;
    assert.equal(transport.advance(Number.MAX_SAFE_INTEGER), null);
    assert.equal(transport.snapshot().currentSection, heldSection);

    const regenerated = generate({ recipe: "adventure", style: "dark", autoplay: true }).score;
    const nextTransport = new AdaptiveTransport(regenerated, heldSection);
    nextTransport.setFormHeld(transport.formHeld, 0);
    assert.equal(nextTransport.formHeld, true);
    assert.equal(nextTransport.snapshot().currentSection, heldSection);
    assert.equal(nextTransport.advance(Number.MAX_SAFE_INTEGER), null);
  });

  it("follows the Adventure tour across the authored section lengths and loops back to explore", () => {
    const { score } = generate({ recipe: "adventure", style: "folk", autoplay: true });
    const form = formOf(score);
    assert.deepEqual(form.steps.map(({ section, repeats = 1 }) => [section, repeats]), [
      ["camp", 1],
      ["explore", 1],
      ["town", 1],
      ["dungeon", 1],
      ["combat", 1],
      ["boss", 1],
      ["sanctuary", 1],
      ["victory", 1],
    ]);
    assert.equal(form.loopFrom, 1);
    assert.equal(form.origin, "transitionStart");
    assert.match(score.id, /-autoplay-v1$/);

    const barTicks = score.beatsPerBar * score.ticksPerBeat;
    const authoredBars: Record<string, number> = {
      camp: 16,
      explore: 32,
      town: 32,
      dungeon: 16,
      combat: 32,
      boss: 16,
      sanctuary: 16,
      victory: 32,
    };
    for (const [id, bars] of Object.entries(authoredBars)) {
      const section = score.sections.find((candidate) => candidate.id === id);
      assert.ok(section, `missing ${id}`);
      assert.equal(section.lengthTicks, bars * barTicks, `${id} must use its authored length`);
    }

    const transport = new AdaptiveTransport(score);
    let boundary = score.sections.find((section) => section.id === "camp")!.lengthTicks;
    assert.equal(completeTransition(transport, boundary).to, "explore");
    for (let index = 1; index < form.steps.length; index++) {
      const step = form.steps[index]!;
      const section = score.sections.find((candidate) => candidate.id === step.section)!;
      boundary += section.lengthTicks * (step.repeats ?? 1);
      const expected: string = index + 1 < form.steps.length
        ? form.steps[index + 1]!.section
        : form.steps[form.loopFrom!]!.section;
      assert.equal(completeTransition(transport, boundary).to, expected);
    }
  });

  it("resumes the Adventure tour from a held section on the authored boundary", () => {
    const { score } = generate({ recipe: "adventure", style: "folk", autoplay: true });
    const transport = new AdaptiveTransport(score);
    const barTicks = score.beatsPerBar * score.ticksPerBeat;
    const camp = score.sections.find((section) => section.id === "camp")!;
    const explore = score.sections.find((section) => section.id === "explore")!;

    const toExplore = completeTransition(transport, camp.lengthTicks);
    assert.equal(toExplore.to, "explore");
    assert.equal(transport.snapshot().currentSection, "explore");

    transport.setFormHeld(true, camp.lengthTicks);
    assert.equal(transport.advance(Number.MAX_SAFE_INTEGER), null);
    assert.equal(transport.snapshot().currentSection, "explore");

    const resumeTick = camp.lengthTicks + 2 * barTicks;
    transport.setFormHeld(false, resumeTick);
    assert.equal(transport.advance(resumeTick), null);

    const toTown = transport.advance(camp.lengthTicks + explore.lengthTicks);
    assert.ok(toTown, "resume must schedule the explore→town boundary");
    assert.equal(toTown.to, "town");
    transport.advance(toTown.endTick);
    assert.equal(transport.snapshot().currentSection, "town");
  });

  it("rejects an unknown recipe instead of silently generating Racing", () => {
    assert.throws(
      () => generate({ recipe: "missing", style: "funk", autoplay: true }),
      /Unknown recipe: missing/,
    );
  });

  it("rejects the retired Medieval recipe instead of silently falling back", () => {
    assert.throws(
      () => generate({ recipe: "medieval", style: "folk", autoplay: true }),
      /Unknown recipe: medieval/,
    );
  });
});
