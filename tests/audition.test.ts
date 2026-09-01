import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { eventMatchesSolo } from "../apps/demo/src/audio-engine.ts";
import type { MusicEvent } from "../packages/runtime/src/index.ts";

const melody: MusicEvent = {
  id: "section:melody:0",
  section: "section",
  lane: "melody",
  kind: "note",
  pitch: 72,
  voice: "epiano",
  role: "melody",
  startTick: 0,
  durationTicks: 480,
  velocity: 0.7,
};

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

describe("audition filtering", () => {
  it("separates melody from the rhythm and harmony backing", () => {
    assert.equal(eventMatchesSolo(melody, "full"), true);
    assert.equal(eventMatchesSolo(harmony, "full"), true);
    assert.equal(eventMatchesSolo(percussion, "full"), true);
    assert.equal(eventMatchesSolo(melody, "melody"), true);
    assert.equal(eventMatchesSolo(harmony, "melody"), false);
    assert.equal(eventMatchesSolo(percussion, "melody"), false);
    assert.equal(eventMatchesSolo(melody, "rhythm"), false);
    assert.equal(eventMatchesSolo(harmony, "rhythm"), true);
    assert.equal(eventMatchesSolo(percussion, "rhythm"), true);
  });
});
