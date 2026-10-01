const VOLUME_KEY = "gamestruments-volume";

/** How long a master-volume change glides, in seconds. */
export const VOLUME_GLIDE_SECONDS = 0.05;

/** The listener's saved master volume (0..1), or full volume. */
export function savedVolume(): number {
  try {
    const parsed = parseFloat(localStorage.getItem(VOLUME_KEY) ?? "");
    if (!Number.isNaN(parsed) && parsed >= 0 && parsed <= 1) return parsed;
  } catch {
    // No localStorage here: keep the default.
  }
  return 1;
}

/** Save a master volume, clamped to 0..1, and return what was saved. */
export function saveVolume(value: number): number {
  const clamped = Math.max(0, Math.min(1, value));
  try {
    localStorage.setItem(VOLUME_KEY, clamped.toString());
  } catch {
    // No localStorage here: the volume still applies for this session.
  }
  return clamped;
}
