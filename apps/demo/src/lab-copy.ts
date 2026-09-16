/**
 * The Audio Lab's one copy source for its two music axes.
 *
 * "Pieza" is the level seed — what a game saves per level. "Versión" is the
 * take: another performance of the same piece, derived from the seed. Both
 * words, and every label that names their controls, come from here instead of
 * being repeated as literals in `index.html`, `ui.ts` and `main.ts`. That is
 * the same one-name lesson as the phase names in `phase-names.ts`: one object,
 * one name, one source.
 */
export const PIECE_AXIS = "Pieza";
export const VERSION_AXIS = "Versión";
export const APPLY_PIECE = "Usar pieza";
export const NEW_PIECE = "Otra pieza";
export const NEW_VERSION = "Otra versión";

/** `level-001 · Versión 3` — the visible readout for the version axis. */
export function versionLabel(seed: string, version: number): string {
  return `${seed} · ${VERSION_AXIS} ${version}`;
}
