import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import { AdaptiveTransport, eventsInRange, validatePortableScore, type PortableScore } from "../packages/runtime/src/index.ts";

const originalDigests = {
  terminal: "c7684ae96cb77bdf1653b4963591ed1171732b0a3c80a10ebb08f6b39ecdc040",
  cipher: "0ed060627fa75edef2bb27ba237096a5ec06953cdfb081df0592a72ed39346c5",
  noir: "87c745db5d9c82e7253c72f27bfdd2dbbda4668a8ad674d24553543670dfe0aa",
};
const withoutConfirmedBeep = {
  terminal: "0ded1d45d98044fb2f63dfb34e015dc6064d754c572c5b7d1bc3d25509b7e928",
  cipher: "8f4149f2c792f960a18cc6c64f9241fb0cb9c49159e1a0a2abbaa89f4c006611",
  noir: "27120987b59cb1062b2b3afdaace83c724fa28f4a8f8d6b68688efb548a74ed5",
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
  for (const style of Object.keys(originalDigests)) {
    it(`${style} Extended keeps the core notes and uninterrupted normal-section drums`, () => {
      const original = generate(style).score;
      const extended = generate(style, "extended").score;
      assert.equal(extended.form?.origin, "transitionStart");
      assert.equal(extended.crossfadeBars, original.crossfadeBars);
      for (const section of extended.sections) {
        const base = original.sections.find((item) => item.id === section.id);
        if (!base) assert.ok(["anomaly", "scan-ii", "breach-ii"].includes(section.id));
        for (const event of (base?.events ?? []).filter((e) => e.kind === "note")) {
          if (["solo", "bridge-b", "chorus-final"].includes(section.id) && event.voice === "glass" && event.lane === `${section.id}-cell`) continue;
          if (section.id === "bridge-b" && event.role === "melody" && event.startTick + event.durationTicks > 4 * 3840) continue;
          const retained = section.events.find((e) => e.id === event.id);
          assert.ok(retained?.kind === "note");
          assert.equal(retained.pitch, event.pitch);
          assert.equal(retained.voice, event.voice);
          assert.equal(retained.velocity, event.velocity);
          assert.equal(retained.startTick, event.startTick);
        }
        const drums = section.events.filter((e) => e.kind === "percussion");
        if (section.id === "intro") {
          assert.deepEqual(section, base);
        } else if (["break", "outro", "coda"].includes(section.id)) {
          assert.equal(drums.length, 0);
        } else {
          for (let bar = 0; bar < section.lengthTicks / 3840; bar++) {
            assert.deepEqual(drums.filter((e) => e.voice === "kick" && Math.floor(e.startTick / 3840) === bar).map((e) => e.startTick % 3840), [0, 1920]);
            assert.deepEqual(drums.filter((e) => e.voice === "hat" && Math.floor(e.startTick / 3840) === bar).map((e) => e.startTick % 3840), [480, 1440, 2400, 3360]);
          }
        }
      }
    });
  }

  it("Extended sustains both drum grids over two entire form cycles with only named breaks", () => {
    const score = generate("terminal", "extended").score;
    const transport = new AdaptiveTransport(score);
    const sections = new Map(score.sections.map((section) => [section.id, section]));
    const entrances = [{ section: score.defaultSection, tick: 0 }];
    for (let tick = 0; entrances.filter((entry) => entry.section === "verse").length < 3; tick += 37) {
      assert.ok(tick < 2000 * 3840, "form must keep advancing");
      const plan = transport.advance(tick, 367);
      if (plan) entrances.push({ section: plan.to, tick: plan.startTick });
    }
    let lastKick: number | undefined;
    let lastHat: number | undefined;
    let breaks = 0;
    for (let index = 0; index < entrances.length - 1; index++) {
      const entry = entrances[index]!;
      const next = entrances[index + 1]!;
      const section = sections.get(entry.section)!;
      assert.equal(next.tick - entry.tick, section.lengthTicks, `no repeated opening before leaving ${section.id}`);
      if (section.id === "intro" || section.id === "break") {
        if (section.id === "break") breaks++;
        lastKick = undefined;
        lastHat = undefined;
        continue;
      }
      for (const event of eventsInRange(section, entry.tick, next.tick, entry.tick)) {
        if (event.kind !== "percussion") continue;
        if (event.voice === "kick") {
          if (lastKick !== undefined) assert.equal(event.startTick - lastKick, 1920, `kick gap in ${section.id}`);
          lastKick = event.startTick;
        }
        if (event.voice === "hat") {
          if (lastHat !== undefined) assert.equal(event.startTick - lastHat, 960, `hat gap in ${section.id}`);
          lastHat = event.startTick;
        }
      }
    }
    assert.equal(breaks, 2);
  });
  for (const [style, digest] of Object.entries(originalDigests)) {
    it(`${style} keeps the approved Original score byte-for-byte`, () => {
      const baseline = generate(style);
      assert.equal(createHash("sha256").update(baseline.bytes).digest("hex"), digest);
      assert.deepEqual(generate(style, "original").bytes, baseline.bytes);
      generate(style, "extended");
      assert.deepEqual(generate(style, "original").bytes, baseline.bytes);
    });

    it(`${style} extends the main beds but preserves Handshake and the break`, () => {
      const original = generate(style).score;
      const extended = generate(style, "extended").score;
      assert.match(extended.id, /-extended-v2-1-1$/);
      assert.deepEqual(generate(style, "extended").score, extended);
      for (const section of extended.sections) {
        if (section.id === "anomaly") {
          assert.equal(section.lengthTicks, 8 * 3840);
          continue;
        }
        if (section.id === "scan-ii" || section.id === "breach-ii") {
          assert.equal(section.lengthTicks, 16 * 3840);
          continue;
        }
        const base = original.sections.find((candidate) => candidate.id === section.id);
        assert.ok(base);
        if (["verse", "verse-b", "chorus", "chorus-final", "bridge", "solo"].includes(section.id)) {
          assert.equal(section.lengthTicks, base.lengthTicks * 2);
          assert.ok(section.events.some((event) => event.lane.endsWith("-atmosphere")));
        } else {
          assert.equal(section.lengthTicks, base.lengthTicks);
        }
      }
      assert.deepEqual(extended.form?.steps.filter((step) => !["anomaly", "scan-ii", "breach-ii"].includes(step.section)), original.form?.steps);
      assert.equal(extended.bpm, original.bpm);
      assert.equal(extended.crossfadeBars, original.crossfadeBars);
    });
  }

  it("rejects an unknown arrangement instead of silently losing the selection", () => {
    assert.throws(() => generate("terminal", "missing"), /Unknown suspense arrangement/);
    assert.throws(() => generate("terminal", "flow"), /Unknown suspense arrangement/);
    assert.throws(() => generate("terminal", "featured"), /Unknown suspense arrangement/);
  });

  for (const [style, digest] of Object.entries(withoutConfirmedBeep)) {
    it(`${style} preserves the approved base material and Anomaly outside the independent variations`, () => {
      const { id, ...music } = generate(style, "extended").score;
      assert.match(id, /-extended-v2-1-1$/);
      const checkpoint = {
        ...music,
        sections: music.sections.filter((section) => !["scan-ii", "breach-ii"].includes(section.id)),
        form: { ...music.form, steps: music.form!.steps.filter((step) => !["scan-ii", "breach-ii"].includes(step.section)) },
      };
      assert.equal(createHash("sha256").update(JSON.stringify(checkpoint)).digest("hex"), digest);
      for (const section of music.sections) {
        if (["solo", "bridge-b", "chorus-final"].includes(section.id)) {
          assert.ok(!section.events.some((event) => event.kind === "note" && event.voice === "glass" && event.lane === `${section.id}-cell`));
        }
      }
    });
  }
});
