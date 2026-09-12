import type { PortableScore, SectionId } from "../../../packages/runtime/src/index.ts";

export const SUSPENSE_PHASE_SECTIONS: Record<string, SectionId> = {
  boot: "intro",
  scan: "verse",
  exploit: "chorus",
  alert: "bridge",
  extract: "outro",
  complete: "coda",
};

export const ADVENTURE_SCENE_SECTIONS: Record<string, SectionId> = {
  camp: "camp",
  explore: "explore",
  town: "town",
  dungeon: "dungeon",
  combat: "combat",
  boss: "boss",
  sanctuary: "sanctuary",
  victory: "victory",
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
