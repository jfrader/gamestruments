export const SCORE_SCHEMA_VERSION = 1 as const;

export type SectionId = string;

export interface NumericRange {
  min?: number;
  max?: number;
}

export interface AdaptiveCondition {
  numeric?: Record<string, NumericRange>;
  categorical?: Record<string, string | readonly string[]>;
}

export interface AdaptiveRule {
  target: SectionId;
  priority: number;
  when: AdaptiveCondition;
  hold?: boolean;
}

export interface SongFormStep {
  section: SectionId;
  repeats?: number;
}

export interface SongForm {
  steps: readonly SongFormStep[];
  loopFrom?: number;
  origin?: "transitionStart" | "transitionEnd";
}

export interface GameState {
  numeric: Record<string, number>;
  categorical: Record<string, string>;
}

interface EventBase {
  id: string;
  section: SectionId;
  lane: string;
  startTick: number;
  durationTicks: number;
  velocity: number;
}

export interface NoteEvent extends EventBase {
  kind: "note";
  pitch: number;
  voice:
    | "warm"
    | "glass"
    | "pulse"
    | "bass"
    | "pluck"
    | "chip"
    | "epiano"
    | "organ"
    | "supersaw"
    | "triangle"
    | "felt"
    | "dusk"
    | "harp"
    | "recorder"
    | "vielle"
    | "bell"
    | "saw-bass"
    | "trance-pad"
    | "trance-lead"
    | "nylon-guitar";
  role?: "melody";
}

export interface PercussionEvent extends EventBase {
  kind: "percussion";
  voice:
    | "kick"
    | "techno-kick"
    | "clap"
    | "snare"
    | "hat"
    | "tom"
    | "reverse-cymbal"
    | "air-impact"
    | "frame-drum"
    | "tambourine"
    | "bombo"
    | "bombo-rim";
}

export interface StemEvent extends EventBase {
  kind: "stem";
  asset: string;
  action: "start" | "stop";
}

export type MusicEvent = NoteEvent | PercussionEvent | StemEvent;

export interface PortableSection {
  id: SectionId;
  label: string;
  feeling: string;
  color: string;
  lengthTicks: number;
  events: readonly MusicEvent[];
}

export interface PortableScore {
  schemaVersion: typeof SCORE_SCHEMA_VERSION;
  id: string;
  title: string;
  bpm: number;
  beatsPerBar: number;
  ticksPerBeat: number;
  crossfadeBars: number;
  defaultSection: SectionId;
  sections: readonly PortableSection[];
  rules: readonly AdaptiveRule[];
  form?: SongForm;
}

export interface TransitionPlan {
  from: SectionId;
  to: SectionId;
  requestedAtTick: number;
  startTick: number;
  endTick: number;
}

export interface SectionGain {
  section: SectionId;
  gain: number;
}

export interface TransportSnapshot {
  currentSection: SectionId;
  pendingSection: SectionId | null;
  transition: TransitionPlan | null;
}

export type TransitionRequest =
  | { status: "unchanged" }
  | { status: "queued"; target: SectionId }
  | { status: "cancelled"; plan: TransitionPlan }
  | {
      status: "scheduled";
      plan: TransitionPlan;
      replacedPlan?: TransitionPlan;
    };
