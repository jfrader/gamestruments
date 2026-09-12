import { describe, expect, it } from "vitest";
import type { PortableSection } from "../packages/runtime/src/index.ts";
import { orbitFrameAt, orbitStyleAt } from "../apps/demo/src/orbit-visualizer.ts";

const section: PortableSection = {
  id: "x",
  label: "X",
  feeling: "test",
  color: "#d7ff3f",
  lengthTicks: 32,
  events: [
    {
      id: "m1",
      section: "x",
      lane: "x-melody",
      startTick: 0,
      durationTicks: 4,
      velocity: 0.9,
      kind: "note",
      pitch: 67,
      voice: "organ",
    },
    {
      id: "h1",
      section: "x",
      lane: "x-harmony",
      startTick: 8,
      durationTicks: 4,
      velocity: 0.7,
      kind: "note",
      pitch: 60,
      voice: "pluck",
    },
    {
      id: "b1",
      section: "x",
      lane: "x-bass",
      startTick: 0,
      durationTicks: 8,
      velocity: 0.95,
      kind: "note",
      pitch: 36,
      voice: "bass",
    },
    {
      id: "k1",
      section: "x",
      lane: "x-kit",
      startTick: 4,
      durationTicks: 1,
      velocity: 0.8,
      kind: "percussion",
      voice: "kick",
    },
  ],
};

describe("orbitFrameAt (per-part rings driven by real lanes)", () => {
  it("groups by stripped part id and derives labels + most-common instruments (kit for percussion)", () => {
    const frame = orbitFrameAt(section, 0, 0, 8, 4);
    expect(frame.parts).toHaveLength(4);
    expect(frame.parts.map((part) => part.id)).toEqual(["melody", "harmony", "bass", "kit"]);
    expect(frame.parts.map((part) => part.label)).toEqual(["Melody", "Harmony", "Bass", "Drums"]);
    expect(frame.parts.map((part) => part.instrument)).toEqual(["organ", "pluck", "bass", "kit"]);
  });

  it("computes part pulses from their own note/perc events (max wins)", () => {
    // at tick 0: melody + bass active (bass velocity higher), kit/harmony silent
    const start = orbitFrameAt(section, 0, 0, 8, 4);
    const bassPulse = start.parts.find((part) => part.id === "bass")!.pulse;
    const melodyPulse = start.parts.find((part) => part.id === "melody")!.pulse;
    expect(bassPulse).toBeGreaterThan(0);
    expect(melodyPulse).toBeGreaterThan(0);
    expect(bassPulse).toBeGreaterThan(melodyPulse);
    expect(start.parts.find((part) => part.id === "kit")!.pulse).toBe(0);
    expect(start.parts.find((part) => part.id === "harmony")!.pulse).toBe(0);

    // the kit fires at tick 4, harmony at tick 8
    expect(orbitFrameAt(section, 4, 4, 8, 4).parts.find((part) => part.id === "kit")!.pulse).toBeCloseTo(0.8);
    expect(orbitFrameAt(section, 8, 8, 8, 4).parts.find((part) => part.id === "harmony")!.pulse).toBeCloseTo(0.7);
  });

  it("pulses the whole orbit on beats and emphasizes the downbeat", () => {
    expect(orbitFrameAt(section, 0, 0, 8, 4).beatPulse).toBe(1);
    expect(orbitFrameAt(section, 8, 8, 8, 4).beatPulse).toBe(0.72);
    expect(orbitFrameAt(section, 4, 4, 8, 4).beatPulse).toBeLessThan(0.1);
  });

  it("part turns are independent and stateless", () => {
    const first = orbitFrameAt(section, 0, 0, 8, 4);
    const later = orbitFrameAt(section, 16, 16, 8, 4);
    const melodyFirst = first.parts.find((part) => part.id === "melody")!.turns;
    const bassFirst = first.parts.find((part) => part.id === "bass")!.turns;
    const melodyLater = later.parts.find((part) => part.id === "melody")!.turns;
    expect(melodyFirst).not.toBeCloseTo(bassFirst, 5);
    expect(melodyLater).not.toBeCloseTo(melodyFirst, 5);
  });
});

describe("orbitStyleAt", () => {
  it("serializes the container vars from a frame", () => {
    expect(orbitStyleAt({ beatPulse: 1, playheadTurns: 0.1875, glowPulse: 0.9, parts: [] })).toEqual({
      "--orbit-scale": "1.0152",
      "--orbit-glow-opacity": "0.558",
      "--orbit-glow-scale": "0.976",
      "--playhead-opacity": "0.94",
      "--playhead-rotation": "0.1875turn",
    });
  });
});
