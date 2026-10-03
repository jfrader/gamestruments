import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { AdaptiveTransport, type PortableScore } from "../packages/runtime/src/index.ts";

const bar = 3840;
const length = 4 * bar;
const score: PortableScore = {
  schemaVersion: 1, id: "form-control", title: "Form control", bpm: 120, ticksPerBeat: 960,
  beatsPerBar: 4, crossfadeBars: 1, defaultSection: "scan", rules: [],
  sections: ["scan", "scan-ii", "breach", "breach-ii", "ending"].map((id) => ({ id, label: id, feeling: "", color: "#fff", lengthTicks: length, events: [] })),
  form: { origin: "transitionStart", steps: ["scan", "scan-ii", "breach", "breach-ii"].map((section) => ({ section })), loopFrom: 0 },
};
const repeated: PortableScore = {
  ...score,
  form: { origin: "transitionStart", steps: ["scan", "scan-ii", "scan", "breach"].map((section) => ({ section })) },
};

describe("game-controlled song form", () => {
  it("plays repeated sections in order and keeps Next at the current occurrence", () => {
    const transport = new AdaptiveTransport(repeated);
    for (const [index, target] of ["scan-ii", "scan", "breach"].entries()) {
      const plan = transport.advance((index + 1) * length + index * bar);
      assert.equal(plan?.to, target);
      assert.ok(plan);
      transport.advance(plan.endTick);
      assert.equal(transport.snapshot().currentSection, target);
      if (target === "scan") assert.equal(transport.nextFormSection(plan.endTick), "breach");
    }
    assert.equal(transport.nextFormSection(3 * length + 2 * bar), null);
  });

  it("cancels lookahead without skipping a repeated section, and resumes after a held cue", () => {
    const transport = new AdaptiveTransport(repeated);
    transport.jumpSection("scan-ii", 0);
    const scheduled = transport.advance(length - 100, 200);
    assert.equal(scheduled?.to, "scan");
    assert.deepEqual(transport.setFormHeld(true, length - 50), scheduled);
    assert.equal(transport.nextFormSection(length - 50), "scan");
    const cue = transport.advanceForm(length + 10);
    assert.equal(cue.status, "scheduled");
    if (cue.status !== "scheduled") return;
    transport.advance(cue.plan.endTick);
    assert.equal(transport.snapshot().currentSection, "scan");
    assert.equal(transport.nextFormSection(cue.plan.endTick), "breach");
    transport.setFormHeld(false, cue.plan.endTick);
    const plan = transport.advance(cue.plan.endTick + length);
    assert.equal(plan?.to, "breach");
  });

  it("syncs an explicit cue to the next matching occurrence", () => {
    const transport = new AdaptiveTransport(repeated);
    const first = transport.requestSection("scan-ii", 0);
    assert.equal(first.status, "scheduled");
    if (first.status !== "scheduled") return;
    transport.advance(first.plan.endTick);
    const second = transport.requestSection("scan", first.plan.endTick);
    assert.equal(second.status, "scheduled");
    if (second.status !== "scheduled") return;
    transport.advance(second.plan.endTick);
    assert.equal(transport.nextFormSection(second.plan.endTick), "breach");
  });

  it("returns to the loop occurrence rather than an earlier matching section", () => {
    const looping: PortableScore = { ...repeated, form: { ...repeated.form!, loopFrom: 2 } };
    for (const manual of [false, true]) {
      const transport = new AdaptiveTransport(looping);
      transport.setFormHeld(manual, 0);
      let tick = 0;
      for (const target of ["scan-ii", "scan", "breach", "scan", "breach"]) {
        tick += length;
        if (manual) transport.advanceForm(tick);
        else transport.advance(tick);
        const plan = transport.snapshot().transition;
        assert.ok(plan);
        assert.equal(plan.to, target);
        tick = plan.endTick;
        transport.advance(tick);
        if (target === "scan") assert.equal(transport.nextFormSection(tick), "breach");
      }
    }
  });

  it("skips adjacent identical steps on manual Next while autoplay keeps their duration", () => {
    const adjacent: PortableScore = {
      ...score,
      form: { origin: "transitionStart", steps: ["scan", "scan", "breach"].map((section) => ({ section })) },
    };
    const manual = new AdaptiveTransport(adjacent);
    assert.equal(manual.nextFormSection(0), "breach");
    const automatic = new AdaptiveTransport(adjacent);
    assert.equal(automatic.advance(length), null);
    assert.equal(automatic.snapshot().currentSection, "scan");
    assert.equal(automatic.advance(2 * length)?.to, "breach");
  });
  it("holds indefinitely, advances once on a bar and keeps the next section held", () => {
    const transport = new AdaptiveTransport(score);
    transport.setFormHeld(true, 0);
    assert.equal(transport.advance(5 * length), null);
    assert.equal(transport.snapshot().currentSection, "scan");
    const request = transport.advanceForm(5 * length + 100);
    assert.equal(request.status, "scheduled");
    if (request.status !== "scheduled") return;
    assert.equal(request.plan.to, "scan-ii");
    assert.equal(request.plan.startTick % bar, 0);
    transport.advance(request.plan.endTick);
    assert.equal(transport.formHeld, true);
    assert.equal(transport.advance(30 * length), null);
    assert.equal(transport.snapshot().currentSection, "scan-ii");
  });

  it("cancels an automatic lookahead cue when holding, then resumes at a future loop end", () => {
    const transport = new AdaptiveTransport(score);
    const automatic = transport.advance(length - 100, 200);
    assert.equal(automatic?.to, "scan-ii");
    assert.deepEqual(transport.setFormHeld(true, length - 50), automatic);
    assert.equal(transport.snapshot().transition, null);
    transport.advance(2 * length + bar);
    transport.setFormHeld(false, 2 * length + bar);
    assert.equal(transport.advance(2 * length + bar), null);
    assert.equal(transport.advance(3 * length - 1), null);
    const resumed = transport.advance(3 * length);
    assert.equal(resumed?.startTick, 3 * length);
    assert.equal(resumed?.to, "scan-ii");
  });

  it("holding never aborts an active blend or an explicit future cue", () => {
    const transport = new AdaptiveTransport(score);
    transport.requestSection("breach", 100);
    assert.equal(transport.setFormHeld(true, 200), null);
    assert.equal(transport.snapshot().transition?.to, "breach");
    transport.advance(2 * bar);
    transport.setFormHeld(false, 2 * bar);
    transport.requestSection("breach-ii", 2 * bar + 100);
    transport.advance(3 * bar + 10);
    assert.equal(transport.setFormHeld(true, 3 * bar + 10), null);
    assert.equal(transport.snapshot().transition?.to, "breach-ii");
    transport.advance(4 * bar);
    assert.equal(transport.snapshot().currentSection, "breach-ii");
    assert.equal(transport.advance(20 * length), null);
  });

  it("has no next form section for an ending or a score without a form", () => {
    const ending = new AdaptiveTransport(score, "ending");
    assert.equal(ending.nextFormSection(0), null);
    assert.equal(ending.advanceForm(0).status, "unchanged");
    const { form: _form, ...withoutForm } = score;
    const plain = new AdaptiveTransport(withoutForm);
    plain.setFormHeld(true, 0);
    assert.equal(plain.formHeld, false);
    assert.equal(plain.nextFormSection(0), null);
  });
});
