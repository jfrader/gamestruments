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

  it("maps active melody pitch and velocity to the inner ring", () => {
    const silent = orbitMotionAt(section, 0, 0, 8, 4);
    const melody = orbitMotionAt(section, 8, 8, 8, 4);
    expect(silent.melodyPulse).toBe(0);
    expect(melody.melodyPulse).toBeCloseTo(0.9);
    expect(melody.melodyAngle).toBe(205);
  });

  it("tracks actual bar and section progress", () => {
    const motion = orbitMotionAt(section, 12, 20, 8, 4);
    expect(motion.barProgress).toBe(0.375);
    expect(motion.phraseProgress).toBe(0.625);
  });

  it("serializes browser-ready values without typed CSS arithmetic", () => {
    const styles = orbitStyleAt({
      barProgress: 0.375,
      beatPulse: 1,
      melodyAngle: 205,
      melodyPulse: 0.9,
      phraseProgress: 0.625,
      rhythmPulse: 0.8,
    });

    expect(styles).toEqual({
      "--orbit-scale": "1.0264",
      "--orbit-glow-opacity": "0.876",
      "--orbit-glow-scale": "1.008",
      "--outer-opacity": "0.932",
      "--outer-rotation": "0.625turn",
      "--outer-scale": "1.028",
      "--inner-opacity": "0.972",
      "--inner-rotation": "205deg",
      "--inner-scale": "1.0675",
      "--playhead-opacity": "1",
      "--playhead-rotation": "0.375turn",
    });
  });
});
