import assert from "node:assert/strict";
import { describe, it } from "vitest";
import {
  exportScore,
  pocketCircuitExperiments,
  pocketCircuitScore,
  type AuthoringScore,
} from "../packages/studio/src/index.ts";

describe("Strudel authoring export", () => {
  it("exports deterministic, sorted, integer-timed instrumental events", () => {
    const first = exportScore(pocketCircuitScore, "same-seed");
    const second = exportScore(pocketCircuitScore, "same-seed");
    assert.deepEqual(first, second);
    assert.equal(first.sections.length, 6);

    for (const section of first.sections) {
      assert.ok(section.events.length > 0);
      assert.ok(
        section.events.every(
          (event) =>
            Number.isSafeInteger(event.startTick) &&
            Number.isSafeInteger(event.durationTicks) &&
            event.startTick >= 0 &&
            event.durationTicks > 0,
        ),
      );
      assert.deepEqual(
        section.events,
        [...section.events].sort(
          (left, right) =>
            left.startTick - right.startTick || left.id.localeCompare(right.id),
        ),
      );
    }
  });

  it("lets the authoring seed vary procedural melodic choices", () => {
    for (const experiment of pocketCircuitExperiments) {
      const signatures = Array.from({ length: 6 }, (_, index) =>
        exportScore(experiment.score, `${experiment.slug}:take-${index + 1}`),
      ).map((score) =>
        score.sections
          .flatMap((section) =>
            section.events.flatMap((event) =>
              event.kind === "note" && event.role === "melody"
                ? [event.pitch]
                : [],
            ),
          )
          .join(","),
      );
      assert.ok(new Set(signatures).size > 1, experiment.score.title);
    }
  });

  it("keeps the main hook stable while genres use distinct rhythms", () => {
    const rhythmSignatures: string[] = [];
    for (const experiment of pocketCircuitExperiments) {
      const takeSignatures = ["take-a", "take-b", "take-c"].map((seed) => {
        const score = exportScore(experiment.score, seed);
        const cruise = score.sections.find((section) => section.id === "cruise");
        assert.ok(cruise);
        const melody = cruise.events.flatMap((event) =>
          event.kind === "note" && event.role === "melody" ? [event] : [],
        );
        return melody.map((event) => `${event.startTick}:${event.pitch}`).join(",");
      });
      assert.equal(new Set(takeSignatures).size, 1, experiment.score.title);
      rhythmSignatures.push(
        takeSignatures[0]
          ?.split(",")
          .map((event) => event.split(":")[0])
          .join(",") ?? "",
      );
    }
    assert.equal(new Set(rhythmSignatures).size, pocketCircuitExperiments.length);
  });

  it("exports a developed four-bar melody in every adaptive section", () => {
    for (const experiment of pocketCircuitExperiments) {
      const score = exportScore(experiment.score, "melody-contract");
      const barTicks = score.beatsPerBar * score.ticksPerBeat;
      for (const section of score.sections) {
        const melody = section.events.flatMap((event) =>
          event.kind === "note" && event.role === "melody" ? [event] : [],
        );
        assert.ok(melody.length >= 8, `${score.title}: ${section.id}`);
        assert.ok(
          melody.some((event) => event.startTick >= barTicks * 3),
          `${score.title}: ${section.id} does not reach bar four`,
        );
        assert.ok(
          new Set(melody.map((event) => event.pitch)).size >= 4,
          `${score.title}: ${section.id} needs a wider melodic shape`,
        );
      }
    }
  });

  it("uses the dedicated electric piano for Countertop's syncopated hook", () => {
    const score = exportScore(pocketCircuitScore, "funk-piano-contract");
    for (const section of score.sections) {
      const melody = section.events.flatMap((event) =>
        event.kind === "note" && event.role === "melody" ? [event] : [],
      );
      assert.ok(melody.every((event) => event.voice === "epiano"));
      assert.ok(
        melody.some(
          (event) => event.startTick % score.ticksPerBeat === score.ticksPerBeat / 2,
        ),
        `${section.id} needs an off-beat piano note`,
      );
    }
  });

  it("keeps Strudel source and placeholder stems outside the portable bundle", () => {
    const portable = exportScore(pocketCircuitScore, "portable");
    const serialized = JSON.stringify(portable);
    assert.equal(serialized.includes("pattern"), false);
    assert.equal(serialized.includes("@strudel"), false);
    assert.equal(serialized.includes("queryArc"), false);
    assert.equal(serialized.includes('"kind":"stem"'), false);
  });

  it("exports only explicit playable stem markers as stem events", () => {
    const score: AuthoringScore = {
      id: "playable-stem-contract",
      title: "Playable Stem",
      bpm: 120,
      beatsPerBar: 4,
      ticksPerBeat: 960,
      crossfadeBars: 2,
      defaultSection: "section",
      sections: [
        {
          id: "section",
          label: "Section",
          feeling: "test",
          color: "#ffffff",
          bars: 4,
          lanes: [
            {
              kind: "percussion",
              id: "kit",
              pattern: "kick ~ snare ~",
              velocity: 0.5,
            },
          ],
          stemMarkers: [
            { lane: "hidden", asset: "hidden.ogg" },
            { lane: "real", asset: "real.ogg", playable: true },
          ],
        },
      ],
      rules: [],
    };
    const portable = exportScore(score, "playable-stem");
    const stems =
      portable.sections[0]?.events.filter((event) => event.kind === "stem") ?? [];
    assert.equal(stems.length, 1);
    assert.equal(stems[0]?.lane, "real");
    assert.equal(stems[0]?.asset, "real.ogg");
  });

  it("exports every racing sound world through the same portable contract", () => {
    assert.equal(pocketCircuitExperiments.length, 4);
    const exported = pocketCircuitExperiments.map((experiment) => ({
      slug: experiment.slug,
      genre: experiment.genre,
      score: exportScore(experiment.score, "catalog-smoke"),
    }));
    assert.equal(new Set(exported.map((item) => item.slug)).size, 4);
    assert.equal(new Set(exported.map((item) => item.genre)).size, 4);
    for (const item of exported) {
      assert.equal(item.score.sections.length, 6);
      assert.ok(item.score.sections.every((section) => section.events.length > 0));
      assert.ok(
        item.score.sections.every((section) =>
          section.events.every((event) => event.kind !== "stem"),
        ),
      );
    }
  });
});
