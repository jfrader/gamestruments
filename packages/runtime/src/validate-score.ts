import { SCORE_SCHEMA_VERSION, type PortableScore } from "./types.js";

const NOTE_VOICES = new Set([
  "warm",
  "glass",
  "pulse",
  "bass",
  "pluck",
  "chip",
  "epiano",
  "organ",
  "supersaw",
  "triangle",
  "felt",
  "dusk",
  "harp",
  "recorder",
  "vielle",
  "bell",
  "saw-bass",
  "trance-pad",
  "trance-lead",
  "nylon-guitar",
]);
const PERCUSSION_VOICES = new Set([
  "kick",
  "techno-kick",
  "clap",
  "snare",
  "hat",
  "tom",
  "reverse-cymbal",
  "air-impact",
  "frame-drum",
  "tambourine",
  "bombo",
  "bombo-rim",
]);

function requireValid(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(`Invalid portable score: ${message}`);
  }
}

function isPositiveSafeInteger(value: number): boolean {
  return Number.isSafeInteger(value) && value > 0;
}

export function validatePortableScore(score: PortableScore): void {
  requireValid(
    score.schemaVersion === SCORE_SCHEMA_VERSION,
    `unsupported schema version ${score.schemaVersion}`,
  );
  requireValid(score.id.length > 0, "score id is empty");
  requireValid(score.title.length > 0, "score title is empty");
  requireValid(Number.isFinite(score.bpm) && score.bpm > 0, "bpm must be positive");
  requireValid(isPositiveSafeInteger(score.beatsPerBar), "beatsPerBar must be a positive integer");
  requireValid(isPositiveSafeInteger(score.ticksPerBeat), "ticksPerBeat must be a positive integer");
  requireValid(Number.isFinite(score.crossfadeBars) && score.crossfadeBars > 0, "crossfadeBars must be positive");
  requireValid(score.sections.length > 0, "at least one section is required");

  const barTicks = score.beatsPerBar * score.ticksPerBeat;
  requireValid(Number.isSafeInteger(barTicks), "bar length exceeds integer timing bounds");
  requireValid(
    Number.isSafeInteger(score.crossfadeBars * barTicks),
    "crossfade duration must resolve to integer ticks",
  );

  const sectionIds = new Set<string>();
  const eventIds = new Set<string>();
  for (const section of score.sections) {
    requireValid(section.id.length > 0, "section id is empty");
    requireValid(!sectionIds.has(section.id), `duplicate section id ${section.id}`);
    sectionIds.add(section.id);
    requireValid(section.label.length > 0, `section ${section.id} has no label`);
    requireValid(isPositiveSafeInteger(section.lengthTicks), `section ${section.id} has invalid length`);

    for (const event of section.events) {
      requireValid(event.id.length > 0, `section ${section.id} contains an event without an id`);
      requireValid(!eventIds.has(event.id), `duplicate event id ${event.id}`);
      eventIds.add(event.id);
      requireValid(event.section === section.id, `event ${event.id} belongs to ${event.section}`);
      requireValid(event.lane.length > 0, `event ${event.id} has no lane`);
      requireValid(Number.isSafeInteger(event.startTick) && event.startTick >= 0, `event ${event.id} has invalid startTick`);
      requireValid(isPositiveSafeInteger(event.durationTicks), `event ${event.id} has invalid durationTicks`);
      requireValid(
        event.startTick + event.durationTicks <= section.lengthTicks,
        `event ${event.id} exceeds section ${section.id}`,
      );
      requireValid(
        Number.isFinite(event.velocity) && event.velocity >= 0 && event.velocity <= 1,
        `event ${event.id} has invalid velocity`,
      );

      if (event.kind === "note") {
        requireValid(
          Number.isSafeInteger(event.pitch) && event.pitch >= 0 && event.pitch <= 127,
          `note ${event.id} has invalid pitch`,
        );
        requireValid(NOTE_VOICES.has(event.voice), `note ${event.id} has unsupported voice`);
        requireValid(
          event.role === undefined || event.role === "melody",
          `note ${event.id} has unsupported role`,
        );
      } else if (event.kind === "percussion") {
        requireValid(
          PERCUSSION_VOICES.has(event.voice),
          `percussion ${event.id} has unsupported voice`,
        );
      } else if (event.kind === "stem") {
        requireValid(event.asset.length > 0, `stem ${event.id} has no asset`);
        requireValid(
          event.action === "start" || event.action === "stop",
          `stem ${event.id} has unsupported action`,
        );
      } else {
        const unknownEvent: never = event;
        throw new Error(`Invalid portable score: unsupported event ${String(unknownEvent)}`);
      }
    }
  }

  requireValid(sectionIds.has(score.defaultSection), `unknown default section ${score.defaultSection}`);
  if (score.form !== undefined) {
    requireValid(
      score.form.origin === undefined || score.form.origin === "transitionStart" || score.form.origin === "transitionEnd",
      "song form origin must be transitionStart or transitionEnd",
    );
    requireValid(score.form.steps.length > 0, "song form must contain at least one step");
    for (const [index, step] of score.form.steps.entries()) {
      requireValid(step.section.length > 0, `song form step ${index} has an empty section`);
      requireValid(
        sectionIds.has(step.section),
        `song form step ${index} targets unknown section ${step.section}`,
      );
      requireValid(
        step.repeats === undefined || (Number.isSafeInteger(step.repeats) && step.repeats > 0),
        `song form step ${index} repeats must be positive`,
      );
    }
    requireValid(
      score.form.loopFrom === undefined ||
        (Number.isSafeInteger(score.form.loopFrom) &&
          score.form.loopFrom >= 0 &&
          score.form.loopFrom < score.form.steps.length),
      `song form loopFrom ${score.form.loopFrom} is out of range`,
    );
  }
  for (const rule of score.rules) {
    requireValid(sectionIds.has(rule.target), `rule targets unknown section ${rule.target}`);
    requireValid(Number.isFinite(rule.priority), `rule for ${rule.target} has invalid priority`);
    for (const [name, range] of Object.entries(rule.when.numeric ?? {})) {
      requireValid(range.min === undefined || Number.isFinite(range.min), `${name} has invalid minimum`);
      requireValid(range.max === undefined || Number.isFinite(range.max), `${name} has invalid maximum`);
      requireValid(
        range.min === undefined || range.max === undefined || range.min <= range.max,
        `${name} has an inverted range`,
      );
    }
  }
}
