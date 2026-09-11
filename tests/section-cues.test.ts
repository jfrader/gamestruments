import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { AdaptiveTransport, type PortableScore } from "../packages/runtime/src/index.ts";
import { cueView } from "../apps/demo/src/section-cues.ts";

const score: PortableScore = {
  schemaVersion: 1, id: "cue-test", title: "Cue test", bpm: 120,
  ticksPerBeat: 960, beatsPerBar: 4, crossfadeBars: 2, defaultSection: "intro", rules: [],
  sections: ["intro", "scan", "anomaly", "coda"].map((id) => ({ id, label: id, feeling: "", color: "#fff", lengthTicks: 30720, events: [] })),
  form: { origin: "transitionStart", steps: [{ section: "intro" }, { section: "scan" }, { section: "anomaly" }], loopFrom: 1 },
};

describe("musical section cues", () => {
  it("preselects a starting section without pretending audio is playing", () => {
    const transport = new AdaptiveTransport(score, "anomaly");
    const view = cueView(score, transport.snapshot(), 0, false, null);
    assert.equal(view.status, "Start with anomaly");
    assert.equal(view.cancellable, false);
  });

  it("waits for the next bar, allows replacement and cancels back to the current form", () => {
    const transport = new AdaptiveTransport(score);
    const cue = transport.requestSection("anomaly", 100);
    assert.equal(cue.status, "scheduled");
    assert.equal(transport.snapshot().currentSection, "intro");
    assert.equal(transport.snapshot().transition?.startTick, 3840);
    let view = cueView(score, transport.snapshot(), 100, true, "anomaly");
    assert.equal(view.current, "intro");
    assert.equal(view.status, "Cued: anomaly");
    assert.equal(view.cancellable, true);
    const replacement = transport.requestSection("coda", 200);
    assert.equal(replacement.status, "scheduled");
    assert.equal(transport.cancelPending(300)?.to, "coda");
    view = cueView(score, transport.snapshot(), 300, true, null);
    assert.equal(view.status, "Automatic progression");
    assert.equal(transport.advance(30720)?.to, "scan");
  });

  it("keeps an active blend intact and only queues the latest choice", () => {
    const transport = new AdaptiveTransport(score);
    transport.requestSection("scan", 100);
    assert.equal(transport.requestSection("anomaly", 4000).status, "queued");
    assert.equal(transport.requestSection("coda", 4100).status, "queued");
    const view = cueView(score, transport.snapshot(), 4100, true, "coda");
    assert.equal(view.current, "scan");
    assert.equal(view.status, "Queued: coda");
    assert.equal(transport.snapshot().transition?.to, "scan");
    assert.equal(transport.cancelPending(4200), null);
    assert.equal(transport.snapshot().transition?.to, "scan");
    assert.equal(transport.snapshot().pendingSection, null);
  });

  it("keeps an edge cue quiet while its section plays and re-arms once the form leaves", () => {
    const transport = new AdaptiveTransport({ ...score, rules: [{ target: "scan", hold: false, priority: 1, when: {} }] });
    transport.requestState({ numeric: {}, categorical: {} }, 0);
    transport.advance(7680);
    transport.advance(30720);
    transport.advance(38400);
    assert.equal(transport.snapshot().currentSection, "anomaly");
    assert.equal(transport.requestState({ numeric: {}, categorical: {} }, 38500).status, "scheduled");
  });

  it("re-arms an edge cue once the form leaves the cued section", () => {
    const transport = new AdaptiveTransport({
      ...score,
      rules: [{ target: "anomaly", hold: false, priority: 1, when: { numeric: { heat: { min: 0.75 } } } }],
    });
    const alert = { numeric: { heat: 0.8 }, categorical: {} };
    const cue = transport.requestState(alert, 0);
    assert.equal(cue.status, "scheduled");
    if (cue.status !== "scheduled") return;
    transport.advance(cue.plan.endTick);
    assert.equal(transport.snapshot().currentSection, "anomaly");
    assert.equal(transport.requestState(alert, cue.plan.endTick).status, "unchanged");
    transport.advance(cue.plan.endTick + 30720);
    transport.advance(cue.plan.endTick + 30720 + 7680);
    assert.equal(transport.snapshot().currentSection, "scan");
    const rearmed = transport.requestState(alert, cue.plan.endTick + 30720 + 7680);
    assert.equal(rearmed.status, "scheduled");
  });
});
