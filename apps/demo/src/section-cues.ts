import type { PortableScore, SectionId } from "../../../packages/runtime/src/index.ts";
import type { PlaybackSnapshot } from "./playback.ts";

export interface CueView {
  current: SectionId;
  target: SectionId | null;
  cancellable: boolean;
  status: string;
  detail: string;
  select: SectionId;
}

export function cueView(score: PortableScore, snapshot: PlaybackSnapshot, tick: number, running: boolean, requested: SectionId | null, held = false): CueView {
  const transition = snapshot.transition;
  const started = transition !== null && tick >= transition.startTick;
  const current = started ? transition.to : snapshot.currentSection;
  const target = snapshot.pendingSection ?? (transition !== null && !started ? transition.to : null);
  const label = (id: SectionId) => score.sections.find((section) => section.id === id)?.label ?? id;
  const cancellable = running && requested !== null && requested === target;
  if (!running) {
    return { current, target: null, cancellable: false, status: `Start with ${label(current)}`, detail: held ? "Play will hold this section. Next or Cue can move to another section." : "Choose any section, then press Play.", select: current };
  }
  if (snapshot.pendingSection !== null) {
    return { current, target, cancellable, status: `Queued: ${label(snapshot.pendingSection)}`, detail: "The current blend will finish first. The latest cue replaces the previous one.", select: snapshot.pendingSection };
  }
  if (transition !== null && !started) {
    const beats = Math.max(1, Math.ceil((transition.startTick - tick) / score.ticksPerBeat));
    const bar = Math.floor(transition.startTick / (score.beatsPerBar * score.ticksPerBeat)) + 1;
    return { current, target, cancellable, status: `${cancellable ? "Cued" : "Next"}: ${label(transition.to)}`,
      detail: `Starts on bar ${bar} · in ${beats} beat${beats === 1 ? "" : "s"} · ${score.crossfadeBars}-bar blend`, select: cancellable ? transition.to : current };
  }
  if (transition !== null && started) {
    return { current, target, cancellable: false, status: `Blending: ${label(transition.from)} → ${label(transition.to)}`,
      detail: "Choose another section to queue it after this blend.", select: current };
  }
  const inForm = score.form?.steps.some((step) => step.section === current);
  if (held && inForm) return { current, target: null, cancellable: false, status: `Holding: ${label(current)}`, detail: "Looping until you cue or advance. Resume automatic to finish this loop and continue.", select: current };
  return { current, target: null, cancellable: false, status: score.form === undefined ? "Following game signals" : inForm ? "Automatic progression" : "Holding this ending",
    detail: "Cue a section to enter on the next bar. No hard cuts.", select: current };
}
