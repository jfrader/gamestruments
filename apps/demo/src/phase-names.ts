/**
 * The Audio Lab's one display-name source for game phases.
 *
 * Keys are the Lab phase ids (`data-phase` / the value the game sends). Values
 * are the names the Lab renders. They mirror the engine's section labels on
 * purpose: `PortableSection { id, label, … }` is serialised into the score, so
 * renaming an id or label in the engine would change the frozen takes. The Lab
 * therefore agrees with the engine instead of inventing a name from the raw id.
 *
 * Every Lab surface that renders a phase name goes through `phaseName`. The
 * section readouts (mood, section list, selector, engine button, cue text)
 * already read `PortableSection.label`, which these values match.
 *
 * Code name ↔ displayed name, for the record
 * (keep in sync with `kit/docs/api.md` and `docs/engine-boundary.md`):
 *   Racing:    garage↔Garage · grid↔Starting Grid · race↔Race Flow · finish↔Finish
 *   Suspense:  boot↔Handshake · scan↔Scan · exploit↔Breach · alert↔Complication ·
 *              extract↔Disconnect · complete↔Closed Session
 *   Adventure: camp↔Trailhead Camp · explore↔The Old Forest · town↔Hearth and Hall ·
 *              dungeon↔The Deep Halls · combat↔Steel and Shadow · boss↔No Retreat ·
 *              sanctuary↔The Hidden Glade · victory↔Lanterns at Dawn
 *   Cozy:      dawn↔First Light Coffee · morning↔Morning Chores · market↔Market Day ·
 *              noon↔Sunny Fields · rain↔Rain on the Roof · evening↔Golden Hour ·
 *              festival↔Harvest Festival · night↔Lanterns Out
 */
export const PHASE_NAMES: Readonly<Record<string, string>> = {
  // Racing game signals.
  garage: "Garage",
  grid: "Starting Grid",
  race: "Race Flow",
  finish: "Finish",
  // Suspense trace phases.
  boot: "Handshake",
  scan: "Scan",
  exploit: "Breach",
  alert: "Complication",
  extract: "Disconnect",
  complete: "Closed Session",
  // Adventure area phases.
  camp: "Trailhead Camp",
  explore: "The Old Forest",
  town: "Hearth and Hall",
  dungeon: "The Deep Halls",
  combat: "Steel and Shadow",
  boss: "No Retreat",
  sanctuary: "The Hidden Glade",
  victory: "Lanterns at Dawn",
  // Cozy times of day and places.
  dawn: "First Light Coffee",
  morning: "Morning Chores",
  market: "Market Day",
  noon: "Sunny Fields",
  rain: "Rain on the Roof",
  evening: "Golden Hour",
  festival: "Harvest Festival",
  night: "Lanterns Out",
};

/** Title-cases an unmapped id so a new phase never renders blank. */
function titleCasePhaseId(id: string): string {
  return id
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

/** The one place the Lab turns a phase id into text. */
export function phaseName(id: string): string {
  return PHASE_NAMES[id] ?? titleCasePhaseId(id);
}
