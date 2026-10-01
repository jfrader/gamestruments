import type { PortableScore, SectionGain, SectionId } from "../../../packages/runtime/src/index.ts";
import type { SoloMode } from "./audio-engine.ts";
import type { LabRecipeProfile, SignalReadings } from "./recipes.ts";

export interface PlaybackTransition {
  from: SectionId;
  to: SectionId;
  startTick: number;
  endTick: number;
}

export interface PlaybackSnapshot {
  currentSection: SectionId;
  /** The section queued behind the active transition. */
  pendingSection: SectionId | null;
  transition: PlaybackTransition | null;
}

/** What the Lab shows on one animation frame. */
export interface PlaybackFrame {
  /** The score sounding now; a new version waits for the bar. */
  score: PortableScore;
  /** The audible position, in ticks. */
  tick: number;
  snapshot: PlaybackSnapshot;
  mix: readonly SectionGain[];
  formHeld: boolean;
  nextFormSection: SectionId | null;
  /** The cue the listener asked for, while it waits to start. */
  requestedCue: SectionId | null;
  /** The audible position inside a section's own phrase. */
  sectionTick(section: SectionId): number;
}

/** One version of a piece, ready to play. */
export interface LabScore {
  score: PortableScore;
  seed: string;
  /** The key the score was generated in. */
  rootPitchClass: number;
  profile: LabRecipeProfile;
}

/** Plays scores in the Lab: the browser synth, or the engine games run. */
export interface LabPlayback {
  readonly running: boolean;
  volume: number;
  soloMode: SoloMode;
  start(): Promise<void>;
  /** Stop; the next start resumes on the section that was sounding. */
  stop(): Promise<void>;
  frame(): PlaybackFrame;
  cue(section: SectionId): void;
  /** Send the signal panel's game state. */
  request(phase: string, readings: SignalReadings): void;
  setHold(held: boolean): void;
  /** Move on to the next section of the song form. */
  advance(): void;
  cancelCue(): void;
  /** Play `next` from here on, from the section that is sounding; resolves
   *  to the playback that plays it. */
  switchTo(next: LabScore): Promise<LabPlayback>;
}

/** The section that is audible: the incoming one once a transition starts. */
export function soundingSection(snapshot: PlaybackSnapshot, tick: number): SectionId {
  const transition = snapshot.transition;
  return transition !== null && tick >= transition.startTick ? transition.to : snapshot.currentSection;
}
