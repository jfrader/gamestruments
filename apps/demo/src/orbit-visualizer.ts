import type { MusicEvent, PortableSection } from "../../../packages/runtime/src/index.ts";

export interface OrbitPart {
  id: string;
  label: string;
  instrument: string;
  pulse: number;
  turns: number;
  colorIndex: number;
}

export interface OrbitFrame {
  beatPulse: number;
  playheadTurns: number;
  glowPulse: number;
  parts: readonly OrbitPart[];
}

export type OrbitStyle = Record<
  | "--orbit-scale"
  | "--orbit-glow-opacity"
  | "--orbit-glow-scale"
  | "--playhead-opacity"
  | "--playhead-rotation",
  string
>;

const PART_LABELS: Record<string, string> = {
  melody: "Melody",
  harmony: "Harmony",
  bass: "Bass",
  kit: "Drums",
};

const PART_RATES: Record<string, number> = { melody: 4, harmony: 6, bass: 10, kit: 7 };
const PART_DIRECTIONS: Record<string, number> = { melody: -1, harmony: 1, bass: -1, kit: 1 };

interface PartGroup {
  noteEvents: MusicEvent[];
  percEvents: MusicEvent[];
  voices: string[];
}

interface PartTemplate {
  id: string;
  label: string;
  instrument: string;
  colorIndex: number;
  noteEvents: MusicEvent[];
  percEvents: MusicEvent[];
}

const sectionPartsCache = new WeakMap<PortableSection, PartTemplate[]>();

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
): number {
  return events.reduce((strongest, event) => {
    const distance = loopDistance(sectionTick, event.startTick, sectionLength);
    if (distance >= windowTicks) {
      return strongest;
    }
    const decay = 1 - distance / windowTicks;
    return Math.max(strongest, event.velocity * decay * decay);
  }, 0);
}

function notePulseAt(
  events: readonly MusicEvent[],
  sectionTick: number,
  lengthTicks: number,
  ticksPerBeat: number,
): number {
  const releaseTicks = ticksPerBeat * 0.35;
  let pulse = 0;

  for (const event of events) {
    if (event.kind !== "note") continue;
    const distance = loopDistance(sectionTick, event.startTick, lengthTicks);
    const audibleTicks = event.durationTicks + releaseTicks;
    if (distance >= audibleTicks) continue;
    const onset = Math.max(0, 1 - distance / (ticksPerBeat * 0.3));
    const release = distance <= event.durationTicks
      ? 1
      : 1 - (distance - event.durationTicks) / releaseTicks;
    const value = event.velocity * release * (0.55 + onset * 0.45);
    if (value > pulse) pulse = value;
  }

  return pulse;
}

function percPulseAt(
  events: readonly MusicEvent[],
  sectionTick: number,
  lengthTicks: number,
  ticksPerBeat: number,
): number {
  return onsetPulse(events, sectionTick, lengthTicks, ticksPerBeat * 0.4);
}

function toTitleCase(id: string): string {
  return id
    .replace(/[-_]+/g, " ")
    .replace(/\b\w/g, (character) => character.toUpperCase())
    .trim();
}

function partIdFromLane(sectionId: string, lane: string): string {
  const prefix = `${sectionId}-`;
  return lane.startsWith(prefix) ? lane.slice(prefix.length) : lane;
}

function mostCommon(values: readonly string[]): string {
  const counts = new Map<string, number>();
  for (const value of values) {
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  let best = values[0] ?? "kit";
  let bestCount = -1;
  for (const [value, count] of counts) {
    if (count > bestCount) {
      bestCount = count;
      best = value;
    }
  }
  return best;
}

// Groups the section's events into one entry per musical part (lane), so each
// ring can be driven by that part's own notes. Cached per section object.
function partTemplates(section: PortableSection): PartTemplate[] {
  const cached = sectionPartsCache.get(section);
  if (cached !== undefined) {
    return cached;
  }

  const groups = new Map<string, PartGroup>();
  for (const event of section.events) {
    if (event.kind === "stem") continue;
    const id = partIdFromLane(section.id, event.lane);
    let group = groups.get(id);
    if (group === undefined) {
      group = { noteEvents: [], percEvents: [], voices: [] };
      groups.set(id, group);
    }
    if (event.kind === "note") {
      group.noteEvents.push(event);
      group.voices.push(event.voice);
    } else {
      group.percEvents.push(event);
      group.voices.push(event.voice);
    }
  }

  const priority = ["melody", "harmony", "bass", "kit"];
  const ordered: string[] = priority.filter((id) => groups.has(id));
  for (const id of groups.keys()) {
    if (!priority.includes(id)) {
      ordered.push(id);
    }
  }

  const templates = ordered.map((id, index) => {
    const group = groups.get(id)!;
    return {
      id,
      label: PART_LABELS[id] ?? toTitleCase(id),
      instrument: group.percEvents.length > 0 ? "kit" : mostCommon(group.voices),
      colorIndex: index % 6,
      noteEvents: group.noteEvents,
      percEvents: group.percEvents,
    };
  });

  sectionPartsCache.set(section, templates);
  return templates;
}

function partTurns(transportTick: number, ticksPerBeat: number, id: string, index: number, pulse: number): number {
  const rate = PART_RATES[id] ?? 5.5 + (index % 3);
  const direction = PART_DIRECTIONS[id] ?? (index % 2 === 0 ? 1 : -1);
  return (transportTick / (ticksPerBeat * rate)) * direction + pulse * 0.01;
}

export function orbitFrameAt(
  section: PortableSection,
  transportTick: number,
  sectionTick: number,
  ticksPerBeat: number,
  beatsPerBar: number,
): OrbitFrame {
  const barTicks = ticksPerBeat * beatsPerBar;
  const beatProgress = (transportTick % ticksPerBeat) / ticksPerBeat;
  const beatIndex = Math.floor((transportTick % barTicks) / ticksPerBeat);
  const beatWeight = beatIndex === 0 ? 1 : 0.72;

  let glowPulse = 0;
  const parts = partTemplates(section).map((template, index) => {
    const pulse = Math.max(
      notePulseAt(template.noteEvents, sectionTick, section.lengthTicks, ticksPerBeat),
      percPulseAt(template.percEvents, sectionTick, section.lengthTicks, ticksPerBeat),
    );
    if (pulse > glowPulse) glowPulse = pulse;
    return {
      id: template.id,
      label: template.label,
      instrument: template.instrument,
      pulse,
      turns: partTurns(transportTick, ticksPerBeat, template.id, index, pulse),
      colorIndex: template.colorIndex,
    };
  });

  return {
    beatPulse: Math.pow(1 - beatProgress, 4) * beatWeight,
    playheadTurns: transportTick / barTicks / 2,
    glowPulse,
    parts,
  };
}

export function orbitStyleAt(frame: OrbitFrame): OrbitStyle {
  return {
    "--orbit-scale": cssNumber(1 + frame.beatPulse * 0.008 + frame.glowPulse * 0.008),
    "--orbit-glow-opacity": cssNumber(0.3 + frame.beatPulse * 0.15 + frame.glowPulse * 0.12),
    "--orbit-glow-scale": cssNumber(0.94 + frame.glowPulse * 0.04),
    "--playhead-opacity": cssNumber(0.82 + frame.beatPulse * 0.12),
    "--playhead-rotation": `${cssNumber(frame.playheadTurns)}turn`,
  };
}
