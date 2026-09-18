/**
 * The Audio Lab's one copy source for its two music axes.
 *
 * "Piece" is the level seed — what a game saves per level. "Version" is the
 * take: another performance of the same piece, derived from the seed. Both
 * words, and every label that names their controls, come from here instead of
 * being repeated as literals in `index.html`, `ui.ts` and `main.ts`. That is
 * the same one-name lesson as the phase names in `phase-names.ts`: one object,
 * one name, one source.
 */
export const PIECE_AXIS = "Piece";
export const VERSION_AXIS = "Version";
export const APPLY_PIECE = "Use piece";
export const NEW_PIECE = "New piece";
export const NEW_VERSION = "New version";

/** `level-001 · Version 3` — the visible readout for the version axis. */
export function versionLabel(seed: string, version: number): string {
  return `${seed} · ${VERSION_AXIS} ${version}`;
}
