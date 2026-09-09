import { describe, expect, it } from "vitest";
import type { PortableSection } from "../packages/runtime/src/index.ts";
import { orbitMotionAt, orbitStyleAt } from "../apps/demo/src/orbit-visualizer.ts";

const section: PortableSection = {
  id: "test",
  label: "Test",
  feeling: "Testing",
  color: "#d7ff3f",
  lengthTicks: 32,
  events: [
    {
      id: "kick",
      section: "test",
      lane: "drums",
      startTick: 0,
      durationTicks: 1,
      velocity: 0.8,
      kind: "percussion",
      voice: "kick",
    },
    {
      id: "lead",
      section: "test",
      lane: "lead",
      startTick: 8,
      durationTicks: 4,
      velocity: 0.9,
      kind: "note",
      pitch: 67,
      voice: "chip",
      role: "melody",
    },
  ],
};

describe("orbitMotionAt", () => {
  it("pulses the whole orbit on beats and emphasizes the downbeat", () => {
    expect(orbitMotionAt(section, 0, 0, 8, 4).beatPulse).toBe(1);
    expect(orbitMotionAt(section, 8, 8, 8, 4).beatPulse).toBe(0.72);
    expect(orbitMotionAt(section, 4, 4, 8, 4).beatPulse).toBeLessThan(0.1);
  });

  it("follows percussion onsets and decays between hits", () => {
    const onset = orbitMotionAt(section, 0, 0, 8, 4);
    const decay = orbitMotionAt(section, 2, 2, 8, 4);
    expect(onset.rhythmPulse).toBeCloseTo(0.8);
    expect(decay.rhythmPulse).toBeGreaterThan(0);
    expect(decay.rhythmPulse).toBeLessThan(onset.rhythmPulse);
  });

  it("maps active melody velocity to the inner ring pulse", () => {
    const silent = orbitMotionAt(section, 0, 0, 8, 4);
    const melody = orbitMotionAt(section, 8, 8, 8, 4);
    expect(silent.melodyPulse).toBe(0);
    expect(melody.melodyPulse).toBeCloseTo(0.9);
  });

  it("tracks smooth rotation progress", () => {
    const motion = orbitMotionAt(section, 12, 20, 8, 4);
    expect(motion.outerTurns).toBeCloseTo(0.046875);
    expect(motion.innerTurns).toBeCloseTo(-0.09375);
    expect(motion.playheadTurns).toBeCloseTo(0.1875);
  });

  it("keeps rotation continuous across bar boundaries", () => {
    const before = orbitMotionAt(section, 31.9, 31.9, 8, 4);
    const after = orbitMotionAt(section, 32.1, 0.1, 8, 4);
    expect(after.outerTurns).toBeGreaterThan(before.outerTurns);
    expect(after.playheadTurns).toBeGreaterThan(before.playheadTurns);
    expect(after.innerTurns).toBeLessThan(before.innerTurns);
  });

  it("serializes browser-ready values without typed CSS arithmetic", () => {
    const styles = orbitStyleAt({
      beatPulse: 1,
      innerTurns: -0.15625,
      melodyPulse: 0.9,
      outerTurns: 0.078125,
      playheadTurns: 0.1875,
      rhythmPulse: 0.8,
    });

    expect(styles).toEqual({
      "--orbit-scale": "1.0144",
      "--orbit-glow-opacity": "0.558",
      "--orbit-glow-scale": "0.976",
      "--outer-opacity": "0.844",
      "--outer-rotation": "0.078125turn",
      "--outer-scale": "1.0112",
      "--inner-opacity": "0.904",
      "--inner-rotation": "-0.15625turn",
      "--inner-scale": "1.027",
      "--playhead-opacity": "0.94",
      "--playhead-rotation": "0.1875turn",
    });
  });
});
