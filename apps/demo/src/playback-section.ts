import type { PortableScore, SectionId } from "../../../packages/runtime/src/index.ts";

export const SUSPENSE_PHASE_SECTIONS: Record<string, SectionId> = {
  boot: "intro",
  scan: "verse",
  exploit: "chorus",
  alert: "bridge",
  extract: "outro",
  complete: "coda",
};

export function playbackSectionOnScore(
  score: PortableScore,
  requested: SectionId | null,
): SectionId {
  if (
    requested !== null &&
    score.sections.some((section) => section.id === requested)
  ) {
    return requested;
  }
  return score.defaultSection;
}
