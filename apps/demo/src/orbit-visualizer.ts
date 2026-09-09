import type { MusicEvent, PortableSection } from "../../../packages/runtime/src/index.ts";

export interface OrbitMotion {
  barProgress: number;
  beatPulse: number;
  melodyAngle: number;
  melodyPulse: number;
  phraseProgress: number;
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

function melodyMotion(
  section: PortableSection,
  sectionTick: number,
  ticksPerBeat: number,
): Pick<OrbitMotion, "melodyAngle" | "melodyPulse"> {
  const releaseTicks = ticksPerBeat * 0.35;
  let melodyAngle = 0;
  let melodyPulse = 0;
  let latestOnsetDistance = Number.POSITIVE_INFINITY;

  for (const event of section.events) {
    if (event.kind !== "note" || event.role !== "melody") {
      continue;
    }
    const distance = loopDistance(sectionTick, event.startTick, section.lengthTicks);
    if (distance < latestOnsetDistance) {
      latestOnsetDistance = distance;
      melodyAngle = ((event.pitch % 12) / 12) * 300 + 30;
    }
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

  return { melodyAngle, melodyPulse };
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
  const melody = melodyMotion(section, sectionTick, ticksPerBeat);

  return {
    barProgress: (transportTick % barTicks) / barTicks,
    beatPulse: Math.pow(1 - beatProgress, 4) * beatWeight,
    melodyAngle: melody.melodyAngle,
    melodyPulse: melody.melodyPulse,
    phraseProgress: sectionTick / section.lengthTicks,
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
    "--orbit-scale": cssNumber(1 + motion.beatPulse * 0.012 + motion.rhythmPulse * 0.018),
    "--orbit-glow-opacity": cssNumber(0.3 + motion.beatPulse * 0.36 + motion.melodyPulse * 0.24),
    "--orbit-glow-scale": cssNumber(0.9 + motion.melodyPulse * 0.12),
    "--outer-opacity": cssNumber(0.66 + motion.rhythmPulse * 0.34),
    "--outer-rotation": `${cssNumber(motion.phraseProgress)}turn`,
    "--outer-scale": cssNumber(1 + motion.rhythmPulse * 0.035),
    "--inner-opacity": cssNumber(0.72 + motion.melodyPulse * 0.28),
    "--inner-rotation": `${cssNumber(motion.melodyAngle)}deg`,
    "--inner-scale": cssNumber(1 + motion.melodyPulse * 0.075),
    "--playhead-opacity": cssNumber(0.74 + motion.beatPulse * 0.26),
    "--playhead-rotation": `${cssNumber(motion.barProgress)}turn`,
  };
}
