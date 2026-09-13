import assert from "node:assert/strict";
import { describe, it } from "vitest";
import {
  playbackSectionOnScore,
  ADVENTURE_SCENE_SECTIONS,
  SUSPENSE_PHASE_SECTIONS,
} from "../apps/demo/src/playback-section.ts";
import {
  SCORE_SCHEMA_VERSION,
  type PortableScore,
} from "../packages/runtime/src/index.ts";

const racing: PortableScore = {
  schemaVersion: SCORE_SCHEMA_VERSION,
  id: "race",
  title: "Race",
  bpm: 120,
  beatsPerBar: 4,
  ticksPerBeat: 960,
  crossfadeBars: 2,
  defaultSection: "garage",
  sections: [
    {
      id: "garage",
      label: "Garage",
      feeling: "calm",
      color: "#fff",
      lengthTicks: 3840,
      events: [],
    },
  ],
  rules: [],
};

const suspense: PortableScore = {
  ...racing,
  id: "suspense",
  title: "Suspense",
  defaultSection: "intro",
  sections: [
    {
      id: "intro",
      label: "Handshake",
      feeling: "cold",
      color: "#666",
      lengthTicks: 7680,
      events: [],
    },
  ],
};

describe("SUSPENSE_PHASE_SECTIONS", () => {
  it("maps every lab phase button to a real bed", () => {
    assert.deepEqual(SUSPENSE_PHASE_SECTIONS, {
      boot: "intro",
      scan: "verse",
      exploit: "chorus",
      alert: "bridge",
      extract: "outro",
      complete: "coda",
    });
  });
});

describe("ADVENTURE_SCENE_SECTIONS", () => {
  it("maps every area phase to its adaptive section", () => {
    assert.deepEqual(ADVENTURE_SCENE_SECTIONS, {
      camp: "camp",
      explore: "explore",
      town: "town",
      dungeon: "dungeon",
      combat: "combat",
      boss: "boss",
      sanctuary: "sanctuary",
      victory: "victory",
    });
  });
});

describe("playbackSectionOnScore", () => {
  it("keeps the current section when the next score still has it", () => {
    assert.equal(playbackSectionOnScore(racing, "garage"), "garage");
  });

  it("falls back to the next score default when switching recipes", () => {
    assert.equal(playbackSectionOnScore(suspense, "garage"), "intro");
    assert.equal(playbackSectionOnScore(suspense, null), "intro");
  });
});
