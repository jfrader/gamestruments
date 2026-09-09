import type { MusicEvent, PortableSection } from "../../../packages/runtime/src/index.ts";

export interface OrbitMotion {
  beatPulse: number;
  innerTurns: number;
  melodyPulse: number;
  outerTurns: number;
  playheadTurns: number;
  rhythmPulse: number;
}

export type OrbitStyle = Record<
  | "--orbit-scale"
  | "--orbit-glow-opacity"
  | "--orbit-glow-scale"
  | "--outer-opacity"
  | "--outer-rotation"
  | "--outer-scale"
  | "--inner-opacity"
  | "--inner-rotation"
  | "--inner-scale"
  | "--playhead-opacity"
  | "--playhead-rotation",
  string
>;

function cssNumber(value: number): string {
  return String(Math.round(value * 1_000_000) / 1_000_000);
}

function loopDistance(currentTick: number, eventTick: number, lengthTicks: number): number {
  return ((currentTick - eventTick) % lengthTicks + lengthTicks) % lengthTicks;
}

function onsetPulse(
  events: readonly MusicEvent[],
  sectionTick: number,
  sectionLength: number,
  windowTicks: number,
  matches: (event: MusicEvent) => boolean,
): number {
  return events.reduce((strongest, event) => {
    if (!matches(event)) {
      return strongest;
    }
    const distance = loopDistance(sectionTick, event.startTick, sectionLength);
    if (distance >= windowTicks) {
      return strongest;
    }
    const decay = 1 - distance / windowTicks;
    return Math.max(strongest, event.velocity * decay * decay);
  }, 0);
}

function melodyPulseAt(
  section: PortableSection,
  sectionTick: number,
  ticksPerBeat: number,
): number {
  const releaseTicks = ticksPerBeat * 0.35;
  let melodyPulse = 0;

  for (const event of section.events) {
    if (event.kind !== "note" || event.role !== "melody") {
      continue;
    }
    const distance = loopDistance(sectionTick, event.startTick, section.lengthTicks);
    const audibleTicks = event.durationTicks + releaseTicks;
    if (distance >= audibleTicks) {
      continue;
    }
    const onset = Math.max(0, 1 - distance / (ticksPerBeat * 0.3));
    const release = distance <= event.durationTicks
      ? 1
      : 1 - (distance - event.durationTicks) / releaseTicks;
    const pulse = event.velocity * release * (0.55 + onset * 0.45);
    if (pulse > melodyPulse) {
      melodyPulse = pulse;
    }
  }

  return melodyPulse;
}

export function orbitMotionAt(
  section: PortableSection,
  transportTick: number,
  sectionTick: number,
  ticksPerBeat: number,
  beatsPerBar: number,
): OrbitMotion {
  const barTicks = ticksPerBeat * beatsPerBar;
  const beatProgress = (transportTick % ticksPerBeat) / ticksPerBeat;
  const beatIndex = Math.floor((transportTick % barTicks) / ticksPerBeat);
  const beatWeight = beatIndex === 0 ? 1 : 0.72;
  const barPosition = transportTick / barTicks;
  const melodyPulse = melodyPulseAt(section, sectionTick, ticksPerBeat);

  return {
    beatPulse: Math.pow(1 - beatProgress, 4) * beatWeight,
    innerTurns: -barPosition / 4,
    melodyPulse,
    outerTurns: barPosition / 8,
    playheadTurns: barPosition / 2,
    rhythmPulse: onsetPulse(
      section.events,
      sectionTick,
      section.lengthTicks,
      ticksPerBeat * 0.4,
      (event) => event.kind === "percussion",
    ),
  };
}

export function orbitStyleAt(motion: OrbitMotion): OrbitStyle {
  return {
    "--orbit-scale": cssNumber(1 + motion.beatPulse * 0.008 + motion.rhythmPulse * 0.008),
    "--orbit-glow-opacity": cssNumber(0.3 + motion.beatPulse * 0.15 + motion.melodyPulse * 0.12),
    "--orbit-glow-scale": cssNumber(0.94 + motion.melodyPulse * 0.04),
    "--outer-opacity": cssNumber(0.7 + motion.rhythmPulse * 0.18),
    "--outer-rotation": `${cssNumber(motion.outerTurns)}turn`,
    "--outer-scale": cssNumber(1 + motion.rhythmPulse * 0.014),
    "--inner-opacity": cssNumber(0.76 + motion.melodyPulse * 0.16),
    "--inner-rotation": `${cssNumber(motion.innerTurns)}turn`,
    "--inner-scale": cssNumber(1 + motion.melodyPulse * 0.03),
    "--playhead-opacity": cssNumber(0.82 + motion.beatPulse * 0.12),
    "--playhead-rotation": `${cssNumber(motion.playheadTurns)}turn`,
  };
}
