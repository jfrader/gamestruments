import assert from "node:assert/strict";
import { describe, it } from "vitest";
import {
  AdaptiveTransport,
  eventsInRange,
  matchesCondition,
  SCORE_SCHEMA_VERSION,
  selectSection,
  validatePortableScore,
  type PortableScore,
} from "../packages/runtime/src/index.ts";

const score: PortableScore = {
  schemaVersion: SCORE_SCHEMA_VERSION,
  id: "test-score",
  title: "Test Score",
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
      events: [
        {
          id: "garage:note:0",
          section: "garage",
          lane: "notes",
          kind: "note",
          pitch: 60,
          voice: "warm",
          startTick: 0,
          durationTicks: 480,
          velocity: 0.5,
        },
        {
          id: "garage:note:1",
          section: "garage",
          lane: "notes",
          kind: "note",
          pitch: 64,
          voice: "warm",
          startTick: 1920,
          durationTicks: 480,
          velocity: 0.5,
        },
      ],
    },
    {
      id: "race",
      label: "Race",
      feeling: "flow",
      color: "#0ff",
      lengthTicks: 3840,
      events: [],
    },
    {
      id: "final",
      label: "Final Lap",
      feeling: "maximum",
      color: "#f00",
      lengthTicks: 3840,
      events: [],
    },
  ],
  rules: [
    {
      target: "final",
      priority: 100,
      when: { numeric: { finalLap: { min: 1 } } },
    },
    {
      target: "race",
      priority: 10,
      when: { categorical: { racePhase: "race" } },
    },
  ],
};

describe("adaptive conditions", () => {
  it("matches numeric ranges and categorical lists", () => {
    const state = {
      numeric: { intensity: 0.72 },
      categorical: { phase: "race" },
    };
    assert.equal(
      matchesCondition(
        {
          numeric: { intensity: { min: 0.5, max: 0.8 } },
          categorical: { phase: ["grid", "race"] },
        },
        state,
      ),
      true,
    );
    assert.equal(
      matchesCondition({ numeric: { intensity: { min: 0.9 } } }, state),
      false,
    );
  });

  it("rejects non-finite numeric state", () => {
    const condition = { numeric: { intensity: { min: 0, max: 1 } } };

    assert.equal(
      matchesCondition(condition, {
        numeric: { intensity: Number.NaN },
        categorical: {},
      }),
      false,
    );
    assert.equal(
      matchesCondition(condition, {
        numeric: { intensity: Number.POSITIVE_INFINITY },
        categorical: {},
      }),
      false,
    );
    assert.equal(
      matchesCondition(condition, {
        numeric: { intensity: Number.NEGATIVE_INFINITY },
        categorical: {},
      }),
      false,
    );
  });

  it("uses the highest-priority matching musical state", () => {
    assert.equal(
      selectSection(score, {
        numeric: { finalLap: 1 },
        categorical: { racePhase: "race" },
      }),
      "final",
    );
  });
});

describe("adaptive transport", () => {
  it("rejects malformed portable score data at the runtime boundary", () => {
    assert.throws(
      () => validatePortableScore({ ...score, bpm: 0 }),
      /bpm must be positive/,
    );
    assert.throws(
      () =>
        validatePortableScore({
          ...score,
          schemaVersion: 999 as typeof SCORE_SCHEMA_VERSION,
        }),
      /unsupported schema version/,
    );
    assert.throws(
      () =>
        validatePortableScore({
          ...score,
          sections: [...score.sections, score.sections[0]!],
        }),
      /duplicate section id/,
    );
    assert.throws(
      () => new AdaptiveTransport({ ...score, crossfadeBars: -1 }),
      /crossfadeBars must be positive/,
    );
  });

  it("quantizes and equal-power crossfades over complete bars", () => {
    const transport = new AdaptiveTransport(score);
    const request = transport.requestSection("race", 100);
    assert.equal(request.status, "scheduled");
    if (request.status !== "scheduled") {
      return;
    }
    assert.deepEqual(request.plan, {
      from: "garage",
      to: "race",
      requestedAtTick: 100,
      startTick: 3840,
      endTick: 11520,
    });

    assert.deepEqual(transport.mixAt(3839), [{ section: "garage", gain: 1 }]);
    const midpoint = transport.mixAt(7680);
    assert.equal(midpoint.length, 2);
    assert.ok(Math.abs((midpoint[0]?.gain ?? 0) - Math.SQRT1_2) < 1e-12);
    assert.ok(Math.abs((midpoint[1]?.gain ?? 0) - Math.SQRT1_2) < 1e-12);
  });

  it("queues rapid state changes until the active crossover completes", () => {
    const transport = new AdaptiveTransport(score);
    transport.requestSection("race", 100);
    assert.deepEqual(transport.requestSection("final", 5000), {
      status: "queued",
      target: "final",
    });
    const queuedPlan = transport.advance(11520);
    assert.deepEqual(queuedPlan, {
      from: "race",
      to: "final",
      requestedAtTick: 11520,
      startTick: 11520,
      endTick: 19200,
    });
  });

  it("replaces or cancels a crossover before its quantized start", () => {
    const transport = new AdaptiveTransport(score);
    const raceRequest = transport.requestSection("race", 100);
    assert.equal(raceRequest.status, "scheduled");
    const replacement = transport.requestSection("final", 200);
    assert.equal(replacement.status, "scheduled");
    if (replacement.status === "scheduled") {
      assert.equal(replacement.replacedPlan?.to, "race");
      assert.equal(replacement.plan.to, "final");
      assert.equal(replacement.plan.startTick, 3840);
    }

    const cancellation = transport.requestSection("garage", 300);
    assert.equal(cancellation.status, "cancelled");
    assert.deepEqual(transport.snapshot(), {
      currentSection: "garage",
      pendingSection: null,
      transition: null,
    });
  });

  it("rejects fractional runtime ticks", () => {
    const transport = new AdaptiveTransport(score);
    assert.throws(() => transport.requestSection("race", 1.5), /safe integer/);
  });

  it("jumps directly to a section and discards queued transitions", () => {
    const transport = new AdaptiveTransport(score);
    transport.requestSection("race", 100);
    transport.requestSection("final", 5000);
    transport.jumpSection("final", 6000);
    assert.deepEqual(transport.snapshot(), {
      currentSection: "final",
      pendingSection: null,
      transition: null,
    });
    assert.deepEqual(transport.mixAt(6000), [{ section: "final", gain: 1 }]);
    assert.equal(transport.advance(20000), null);
  });

  it("rejects jumps to unknown sections", () => {
    const transport = new AdaptiveTransport(score);
    assert.throws(
      () => transport.jumpSection("missing", 0),
      /Unknown target section/,
    );
  });

  it("auto-advances a song form and only cues hold:false once", () => {
    const formed: PortableScore = {
      ...score,
      defaultSection: "garage",
      form: {
        steps: [
          { section: "garage" },
          { section: "race" },
          { section: "final" },
        ],
        loopFrom: 1,
      },
      rules: [
        {
          target: "final",
          priority: 80,
          hold: false,
          when: { numeric: { heat: { min: 0.75 } } },
        },
      ],
    };
    const transport = new AdaptiveTransport(formed);
    const first = transport.advance(3840);
    assert.equal(first?.to, "race");
    transport.advance(3840 + 7680);
    assert.equal(transport.snapshot().currentSection, "race");

    const cue = transport.requestState(
      { numeric: { heat: 0.8 }, categorical: {} },
      3840 + 7680,
    );
    assert.equal(cue.status, "scheduled");
    const again = transport.requestState(
      { numeric: { heat: 0.8 }, categorical: {} },
      3840 + 7680,
    );
    assert.equal(again.status, "unchanged");
  });
});

describe("event query", () => {
  it("is invariant when the same window is partitioned", () => {
    const section = score.sections[0];
    assert.ok(section);
    const whole = eventsInRange(section, 0, 11520);
    const partitioned = [
      ...eventsInRange(section, 0, 3840),
      ...eventsInRange(section, 3840, 7680),
      ...eventsInRange(section, 7680, 11520),
    ];
    assert.deepEqual(partitioned, whole);
    assert.deepEqual(
      whole.map((event) => event.startTick),
      [0, 1920, 3840, 5760, 7680, 9600],
    );
  });

  it("starts a section loop at a nonzero phrase origin", () => {
    const section = score.sections[0];
    assert.ok(section);
    const origin = 5760;
    const events = eventsInRange(section, 0, origin + section.lengthTicks, origin);

    assert.equal(events[0]?.startTick, origin);
    assert.ok(events.every((event) => event.startTick >= origin));
    assert.deepEqual(
      events.map((event) => event.startTick),
      [origin, origin + 1920],
    );
  });
});
