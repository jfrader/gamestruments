import { mini } from "@strudel/mini";
import {
  SCORE_SCHEMA_VERSION,
  type AdaptiveRule,
  type MusicEvent,
  type NoteEvent,
  type PercussionEvent,
  type PortableScore,
  type PortableSection,
  type SongForm,
  type StemEvent,
} from "@gamestruments/runtime";

interface NoteLane {
  kind: "note";
  id: string;
  pattern: string;
  voice: NoteEvent["voice"];
  role?: NoteEvent["role"];
  velocity?: number;
  gate?: number;
}

interface PercussionLane {
  kind: "percussion";
  id: string;
  pattern: string;
  velocity?: number;
}

type AuthoringLane = NoteLane | PercussionLane;

interface StemMarker {
  lane: string;
  asset: string;
  playable?: boolean;
}

export interface AuthoringSection {
  id: string;
  label: string;
  feeling: string;
  color: string;
  bars: number;
  lanes: readonly AuthoringLane[];
  stemMarkers?: readonly StemMarker[];
}

export interface AuthoringScore {
  id: string;
  title: string;
  bpm: number;
  beatsPerBar: number;
  ticksPerBeat: number;
  crossfadeBars: number;
  defaultSection: string;
  sections: readonly AuthoringSection[];
  rules: readonly AdaptiveRule[];
  form?: SongForm;
}

const NOTE_OFFSETS: Readonly<Record<string, number>> = {
  c: 0,
  d: 2,
  e: 4,
  f: 5,
  g: 7,
  a: 9,
  b: 11,
};

const PERCUSSION_VOICES = new Set<PercussionEvent["voice"]>([
  "kick",
  "snare",
  "hat",
  "tom",
]);

function hashSeed(seed: string): number {
  let hash = 0x811c9dc5;
  for (const character of seed) {
    hash ^= character.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 0x01000193);
  }
  return hash >>> 0;
}

function noteToMidi(note: string): number {
  const match = /^([a-gA-G])([#b]?)(-?\d)$/.exec(note);
  if (match === null) {
    throw new Error(`Unsupported note token: ${note}`);
  }

  const [, letter = "", accidental = "", octaveText = ""] = match;
  const base = NOTE_OFFSETS[letter.toLowerCase()];
  if (base === undefined) {
    throw new Error(`Unsupported note token: ${note}`);
  }
  const accidentalOffset = accidental === "#" ? 1 : accidental === "b" ? -1 : 0;
  const pitch = (Number(octaveText) + 1) * 12 + base + accidentalOffset;
  if (!Number.isInteger(pitch) || pitch < 0 || pitch > 127) {
    throw new Error(`Note is outside the MIDI range: ${note}`);
  }
  return pitch;
}

function toTick(cycles: number, barTicks: number): number {
  const tick = Math.round(cycles * barTicks);
  if (Math.abs(tick / barTicks - cycles) > 1e-7) {
    throw new Error(`Pattern time cannot be represented as integer ticks: ${cycles}`);
  }
  return tick;
}

function clampVelocity(value: number): number {
  return Math.max(0, Math.min(1, value));
}

function exportLane(
  section: AuthoringSection,
  lane: AuthoringLane,
  barTicks: number,
  seed: number,
): MusicEvent[] {
  let queried: { startTick: number; endTick: number; token: string }[];
  try {
    queried = mini(lane.pattern)
      .queryArc(0, section.bars, { randSeed: seed })
      .filter((hap) => hap.whole !== undefined && hap.hasOnset())
      .map((hap) => {
        const whole = hap.whole;
        if (whole === undefined) {
          throw new Error("Strudel returned an onset without a whole timespan");
        }
        return {
          startTick: toTick(Number(whole.begin.valueOf()), barTicks),
          endTick: toTick(Number(whole.end.valueOf()), barTicks),
          token: String(hap.value),
        };
      })
      .sort(
        (left, right) =>
          left.startTick - right.startTick ||
          left.endTick - right.endTick ||
          left.token.localeCompare(right.token),
      );
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Cannot export ${section.id}:${lane.id}: ${message}`);
  }

  return queried.map((event, index) => {
    const base = {
      id: `${section.id}:${lane.id}:${index}`,
      section: section.id,
      lane: lane.id,
      startTick: event.startTick,
      durationTicks: Math.max(
        1,
        Math.round(
          (event.endTick - event.startTick) *
            (lane.kind === "note" ? (lane.gate ?? 0.82) : 0.45),
        ),
      ),
      velocity: clampVelocity(lane.velocity ?? 0.76),
    };

    if (lane.kind === "note") {
      return {
        ...base,
        kind: "note",
        pitch: noteToMidi(event.token),
        voice: lane.voice,
        ...(lane.role === undefined ? {} : { role: lane.role }),
      } satisfies NoteEvent;
    }

    if (!PERCUSSION_VOICES.has(event.token as PercussionEvent["voice"])) {
      throw new Error(`Unsupported percussion token: ${event.token}`);
    }
    return {
      ...base,
      kind: "percussion",
      voice: event.token as PercussionEvent["voice"],
    } satisfies PercussionEvent;
  });
}

function exportSection(
  section: AuthoringSection,
  score: AuthoringScore,
  seed: string,
): PortableSection {
  const barTicks = score.beatsPerBar * score.ticksPerBeat;
  const lengthTicks = section.bars * barTicks;
  const patternEvents = section.lanes.flatMap((lane, index) =>
    exportLane(section, lane, barTicks, hashSeed(`${seed}:${section.id}:${index}`)),
  );
  const stemEvents: StemEvent[] = (section.stemMarkers ?? [])
    .filter((marker) => marker.playable === true)
    .map((marker, index) => ({
      id: `${section.id}:${marker.lane}:stem:${index}`,
      section: section.id,
      lane: marker.lane,
      kind: "stem",
      asset: marker.asset,
      action: "start",
      startTick: 0,
      durationTicks: lengthTicks,
      velocity: 1,
    }));

  return {
    id: section.id,
    label: section.label,
    feeling: section.feeling,
    color: section.color,
    lengthTicks,
    events: [...patternEvents, ...stemEvents].sort(
      (left, right) =>
        left.startTick - right.startTick || left.id.localeCompare(right.id),
    ),
  };
}

export function exportScore(
  score: AuthoringScore,
  seed = "gamestruments",
): PortableScore {
  const exported: PortableScore = {
    schemaVersion: SCORE_SCHEMA_VERSION,
    id: score.id,
    title: score.title,
    bpm: score.bpm,
    beatsPerBar: score.beatsPerBar,
    ticksPerBeat: score.ticksPerBeat,
    crossfadeBars: score.crossfadeBars,
    defaultSection: score.defaultSection,
    sections: score.sections.map((section) =>
      exportSection(section, score, seed),
    ),
    rules: score.rules,
    ...(score.form === undefined ? {} : { form: score.form }),
  };

  const ids = new Set(exported.sections.map((section) => section.id));
  if (!ids.has(exported.defaultSection)) {
    throw new Error(`Default section does not exist: ${exported.defaultSection}`);
  }
  return exported;
}
