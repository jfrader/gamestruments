import assert from "node:assert/strict";
import { describe, it } from "vitest";
import {
  createTransitionCurve,
  schedulingStartTick,
  sectionSchedulingState,
  transitionCurveForEvent,
  transitionGainAt,
} from "../apps/demo/src/audio-engine.ts";
import type { MusicEvent } from "../packages/runtime/src/index.ts";

const harmony: MusicEvent = {
  id: "section:harmony:0",
  section: "section",
  lane: "harmony",
  kind: "note",
  pitch: 60,
  voice: "warm",
  startTick: 0,
  durationTicks: 960,
  velocity: 0.5,
};

const melody: MusicEvent = {
  ...harmony,
  id: "section:melody:0",
  lane: "melody",
  role: "melody",
};

const bass: MusicEvent = {
  ...harmony,
  id: "section:bass:0",
  lane: "bass",
  pitch: 36,
  voice: "bass",
};

const percussion: MusicEvent = {
  id: "section:kit:0",
  section: "section",
  lane: "kit",
  kind: "percussion",
  voice: "kick",
  startTick: 0,
  durationTicks: 120,
  velocity: 0.8,
};

describe("audio transition curves", () => {
  it("uses linear gains for correlated harmony and bass material", () => {
    assert.equal(transitionCurveForEvent(harmony), "linear");
    assert.equal(transitionCurveForEvent(bass), "linear");
    const fadeOut = transitionGainAt(1, 0, 0.5, "linear");
    const fadeIn = transitionGainAt(0, 1, 0.5, "linear");
    assert.equal(fadeOut, 0.5);
    assert.equal(fadeIn, 0.5);
    assert.equal(fadeOut + fadeIn, 1);
  });

  it("keeps equal-power crossfades for melody and percussion", () => {
    assert.equal(transitionCurveForEvent(melody), "equalPower");
    assert.equal(transitionCurveForEvent(percussion), "equalPower");
    const fadeOut = transitionGainAt(1, 0, 0.5, "equalPower");
    const fadeIn = transitionGainAt(0, 1, 0.5, "equalPower");
    assert.ok(Math.abs(fadeOut ** 2 + fadeIn ** 2 - 1) < 1e-12);
    assert.ok(fadeOut + fadeIn > 1.4);
  });

  it("builds curves that preserve held replacement levels", () => {
    const curve = createTransitionCurve(0.35, 1, "equalPower", 5);
    assert.ok(Math.abs((curve[0] ?? 0) - 0.35) < 1e-6);
    assert.equal(curve.at(-1), 1);
  });
});

describe("audio scheduling horizons", () => {
  it("preserves a future high-water mark when a section is reactivated", () => {
    assert.equal(schedulingStartTick(100, 140), 140);
  });

  it("catches a stale or missing horizon up to the captured tick", () => {
    assert.equal(schedulingStartTick(160, 140), 160);
    assert.equal(schedulingStartTick(160, undefined), 160);
  });

  it("keeps a new phrase origin while honoring the previous scheduling high-water", () => {
    assert.deepEqual(sectionSchedulingState(100, 140, 120), {
      fromTick: 140,
      loopOrigin: 120,
    });
  });
});
