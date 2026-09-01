import assert from "node:assert/strict";
import { describe, it } from "vitest";
import {
  AdaptiveTransport,
  selectSection,
  validatePortableScore,
  type NoteEvent,
  type PortableScore,
} from "../packages/runtime/src/index.ts";
import {
  createLanternTrailMusicalDNA,
  DEFAULT_LANTERN_TRAIL_TRAITS,
  deriveLanternTrailDomainSeeds,
  deriveLanternTrailSubSeed,
  exportScore,
  generateLanternTrailAdventure,
  generateLanternTrailAuthoringScore,
  LANTERN_TRAIL_GENERATOR_DOMAINS,
  LANTERN_TRAIL_GENERATOR_VERSION,
  resolveLanternTrailSectionHarmony,
  type LanternTrailGeneratorInput,
  type NormalizedTrailTraits,
} from "../packages/studio/src/index.ts";

const BALANCED_TRAITS: NormalizedTrailTraits = {
  wonder: 0.55,
  danger: 0.5,
  mystery: 0.6,
  motion: 0.58,
};

function input(
  seed: LanternTrailGeneratorInput["seed"],
  traits: NormalizedTrailTraits = BALANCED_TRAITS,
): LanternTrailGeneratorInput {
  return { seed, traits };
}

function melodyEvents(score: PortableScore) {
  return score.sections.flatMap((section) =>
    section.events.flatMap((event) =>
      event.kind === "note" && event.role === "melody" ? [event] : [],
    ),
  );
}

function average(values: readonly number[]): number {
  return values.reduce((total, value) => total + value, 0) / values.length;
}

describe("Lantern Trail deterministic generator", () => {
  it("provides stable defaults while accepting partial generation parameters", () => {
    assert.deepEqual(
      generateLanternTrailAdventure({ seed: "default-traits" }),
      generateLanternTrailAdventure({
        seed: "default-traits",
        traits: DEFAULT_LANTERN_TRAIL_TRAITS,
      }),
    );
    assert.equal(
      generateLanternTrailAdventure({
        seed: "partial-traits",
        traits: { wonder: 0.91 },
      }).traits.wonder,
      0.91,
    );
  });

  it("reproduces the complete authoring and portable result for the same seed", () => {
    const first = generateLanternTrailAdventure(input("glade-17"));
    const second = generateLanternTrailAdventure(input("glade-17"));

    assert.deepEqual(first, second);
    assert.equal(first.generatorVersion, LANTERN_TRAIL_GENERATOR_VERSION);
    assert.match(first.authoringScore.id, /generated-v1-1-0-[0-9a-f]{8}$/);
    assert.equal(first.portableScore.id, first.authoringScore.id);
    assert.deepEqual(
      first.portableScore,
      exportScore(
        first.authoringScore,
        `${LANTERN_TRAIL_GENERATOR_VERSION}:${first.levelSeed}`,
      ),
    );
  });

  it("keeps string and numeric seeds in separate deterministic namespaces", () => {
    const fromString = generateLanternTrailAdventure(input("8042"));
    const fromNumber = generateLanternTrailAdventure(input(8042));
    assert.notDeepEqual(fromString.dna, fromNumber.dna);
    assert.deepEqual(fromNumber, generateLanternTrailAdventure(input(8042)));
  });

  it("derives stable independent named domain seeds", () => {
    const seed = "named-domain-contract";
    const domainSeeds = deriveLanternTrailDomainSeeds(seed);
    const reversedDomains = [...LANTERN_TRAIL_GENERATOR_DOMAINS].reverse();

    for (const domain of reversedDomains) {
      assert.equal(domainSeeds[domain], deriveLanternTrailSubSeed(seed, domain));
    }
    assert.equal(
      new Set(Object.values(domainSeeds)).size,
      LANTERN_TRAIL_GENERATOR_DOMAINS.length,
    );
    assert.deepEqual(
      createLanternTrailMusicalDNA(
        input(seed, { wonder: 0, danger: 0, mystery: 0, motion: 0 }),
      ).domainSeeds,
      createLanternTrailMusicalDNA(
        input(seed, { wonder: 1, danger: 1, mystery: 1, motion: 1 }),
      ).domainSeeds,
    );
  });

  it("materially varies every musical domain across adventure seeds", () => {
    const generated = Array.from({ length: 18 }, (_, index) =>
      generateLanternTrailAdventure(input(`diversity-${index}`)),
    );
    const signatures = {
      harmony: generated.map(({ dna }) =>
        JSON.stringify([
          dna.harmony.key,
          dna.harmony.mode,
          dna.harmony.progressionDegrees,
        ]),
      ),
      motif: generated.map(({ dna }) => JSON.stringify(dna.motif.degrees)),
      rhythm: generated.map(({ dna }) =>
        JSON.stringify([
          dna.rhythm.melodyOnsets,
          dna.rhythm.kickOnsets,
          dna.rhythm.snareOnsets,
        ]),
      ),
      timbre: generated.map(({ dna }) =>
        JSON.stringify([
          dna.timbre.harmonyVoice,
          dna.timbre.melodyVoice,
          dna.arrangement.bassApproach,
        ]),
      ),
    };

    assert.ok(new Set(signatures.harmony).size >= 8);
    assert.ok(new Set(signatures.motif).size >= 6);
    assert.ok(new Set(signatures.rhythm).size >= 6);
    assert.ok(new Set(signatures.timbre).size >= 5);
    assert.ok(
      new Set(generated.map(({ portableScore }) => JSON.stringify(portableScore))).size >= 16,
    );
    for (const { dna, portableScore } of generated) {
      assert.ok(
        ["aeolian", "dorian", "mixolydian", "lydian", "phrygian"].includes(
          dna.harmony.mode,
        ),
      );
      assert.ok(portableScore.bpm >= 82 && portableScore.bpm <= 130);
      assert.equal(portableScore.beatsPerBar, 4);
      assert.equal(portableScore.ticksPerBeat, 960);
    }
  });

  it("gives each lantern trait an observable deterministic effect", () => {
    const base = { ...BALANCED_TRAITS };

    const wonderLow = generateLanternTrailAdventure(
      input("trait-contract", { ...base, wonder: 0.1 }),
    );
    const wonderHigh = generateLanternTrailAdventure(
      input("trait-contract", { ...base, wonder: 0.9 }),
    );
    assert.ok(
      average(melodyEvents(wonderHigh.portableScore).map((event) => event.pitch)) >
        average(melodyEvents(wonderLow.portableScore).map((event) => event.pitch)),
    );
    assert.equal(wonderHigh.dna.timbre.melodyVoice, "glass");
    assert.ok(["lydian", "mixolydian"].includes(wonderHigh.dna.harmony.mode));

    const dangerLow = generateLanternTrailAdventure(
      input("trait-contract", { ...base, danger: 0.1 }),
    );
    const dangerHigh = generateLanternTrailAdventure(
      input("trait-contract", { ...base, danger: 0.9 }),
    );
    assert.ok(dangerHigh.portableScore.bpm > dangerLow.portableScore.bpm);
    assert.ok(
      average(dangerHigh.portableScore.sections.flatMap((section) => section.events.map((event) => event.velocity))) >
        average(dangerLow.portableScore.sections.flatMap((section) => section.events.map((event) => event.velocity))),
    );

    const mysteryLow = generateLanternTrailAdventure(
      input("trait-contract", { ...base, mystery: 0.1 }),
    );
    const mysteryHigh = generateLanternTrailAdventure(
      input("trait-contract", { ...base, mystery: 0.9 }),
    );
    assert.equal(mysteryLow.dna.harmony.chordSize, 3);
    assert.equal(mysteryHigh.dna.harmony.chordSize, 4);
    assert.ok(["phrygian", "aeolian"].includes(mysteryHigh.dna.harmony.mode));
    assert.ok(mysteryHigh.dna.arrangement.melodyGate < mysteryLow.dna.arrangement.melodyGate);

    const motionLow = generateLanternTrailAdventure(
      input("trait-contract", { ...base, motion: 0.1 }),
    );
    const motionHigh = generateLanternTrailAdventure(
      input("trait-contract", { ...base, motion: 0.9 }),
    );
    assert.ok(
      melodyEvents(motionHigh.portableScore).length >
        melodyEvents(motionLow.portableScore).length,
    );
  });

  it("keeps generated kick and snare onsets disjoint", () => {
    for (let index = 0; index < 64; index += 1) {
      const { rhythm } = createLanternTrailMusicalDNA(input(`drum-overlap-${index}`));
      assert.equal(
        rhythm.kickOnsets.some((step) => rhythm.snareOnsets.includes(step)),
        false,
      );
    }
  });

  it("exports six non-empty sections with bounded sorted integer events", () => {
    const { portableScore } = generateLanternTrailAdventure(
      input(8042, { wonder: 0.82, danger: 0.76, mystery: 0.68, motion: 0.71 }),
    );

    assert.deepEqual(
      portableScore.sections.map((section) => section.id),
      ["camp", "explore", "clue", "danger", "sanctuary", "quest-complete"],
    );
    for (const section of portableScore.sections) {
      assert.equal(section.lengthTicks, 4 * 4 * 960);
      assert.ok(section.events.length > 0);
      assert.deepEqual(
        section.events,
        [...section.events].sort(
          (left, right) =>
            left.startTick - right.startTick || left.id.localeCompare(right.id),
        ),
      );
      for (const event of section.events) {
        assert.ok(Number.isSafeInteger(event.startTick));
        assert.ok(Number.isSafeInteger(event.durationTicks));
        assert.ok(event.startTick >= 0 && event.startTick < section.lengthTicks);
        assert.ok(event.durationTicks > 0);
        assert.ok(event.startTick + event.durationTicks <= section.lengthTicks);
        assert.ok(event.velocity >= 0 && event.velocity <= 1);
        if (event.kind === "note") {
          assert.ok(Number.isSafeInteger(event.pitch));
          assert.ok(event.pitch >= 0 && event.pitch <= 127);
        }
      }
    }
  });

  it("develops one shared musical DNA through every section", () => {
    const { dna, portableScore } = generateLanternTrailAdventure(
      input("shared-musical-dna"),
    );
    const barTicks = portableScore.beatsPerBar * portableScore.ticksPerBeat;
    const firstBarAnchors: string[] = [];

    for (const section of portableScore.sections) {
      const resolved = resolveLanternTrailSectionHarmony(dna, section.id);
      const allowedPitchClasses = new Set(
        resolved.scaleIntervals.map(
          (interval) => (dna.harmony.rootPitchClass + interval) % 12,
        ),
      );
      const melody = section.events.flatMap((event) =>
        event.kind === "note" && event.role === "melody" ? [event] : [],
      );
      assert.ok(melody.length >= 12, section.id);
      assert.ok(melody.some((event) => event.startTick >= barTicks * 3), section.id);
      assert.ok(new Set(melody.map((event) => event.pitch)).size >= 4, section.id);
      assert.ok(
        melody.every((event) => event.startTick % dna.rhythm.pulseTicks === 0),
        section.id,
      );
      assert.ok(
        section.events.every(
          (event) => event.kind !== "note" || allowedPitchClasses.has(event.pitch % 12),
        ),
        section.id,
      );
      const barSignatures = Array.from({ length: 4 }, (_, barIndex) =>
        melody
          .filter(
            (event) =>
              event.startTick >= barIndex * barTicks &&
              event.startTick < (barIndex + 1) * barTicks,
          )
          .map((event) => `${event.startTick % barTicks}:${event.pitch}`)
          .join(","),
      );
      assert.ok(new Set(barSignatures).size >= 3, section.id);
      firstBarAnchors.push(barSignatures[0] ?? "");
    }
    // camp, explore, and clue share one resolved harmony, so their motif opening matches
    assert.equal(firstBarAnchors[0], firstBarAnchors[1]);
    assert.equal(firstBarAnchors[0], firstBarAnchors[2]);
  });

  it("applies section-specific harmony treatments while preserving tonic", () => {
    const { dna, portableScore } = generateLanternTrailAdventure(
      input("trail-harmony", { wonder: 0.3, danger: 0.5, mystery: 0.9, motion: 0.5 }),
    );
    const rootPitchClass = dna.harmony.rootPitchClass;
    const sections = new Map(portableScore.sections.map((section) => [section.id, section]));

    // camp, explore, and clue share one resolved harmony
    const shared = resolveLanternTrailSectionHarmony(dna, "camp");
    assert.deepEqual(resolveLanternTrailSectionHarmony(dna, "explore"), shared);
    assert.deepEqual(resolveLanternTrailSectionHarmony(dna, "clue"), shared);

    // danger raises the seventh into a leading tone for menace
    const danger = resolveLanternTrailSectionHarmony(dna, "danger");
    assert.notDeepEqual(danger.scaleIntervals, dna.harmony.scaleIntervals);
    assert.deepEqual(danger.progressionDegrees, dna.harmony.progressionDegrees);
    assert.ok(danger.scaleIntervals.includes(11));
    const dangerSection = sections.get("danger");
    assert.ok(dangerSection);
    assert.ok(
      dangerSection.events.some(
        (event) =>
          event.kind === "note" &&
          event.pitch % 12 === (rootPitchClass + 11) % 12,
      ),
    );

    // sanctuary turns lydian (raised fourth) for radiant arrival
    const sanctuary = resolveLanternTrailSectionHarmony(dna, "sanctuary");
    assert.ok(sanctuary.scaleIntervals.includes(6));

    // quest-complete resolves to parallel major (major third = release color)
    const questComplete = resolveLanternTrailSectionHarmony(dna, "quest-complete");
    assert.ok(questComplete.scaleIntervals.includes(4));
    assert.equal(questComplete.progressionDegrees[3], 0);
    const questSection = sections.get("quest-complete");
    assert.ok(questSection);
    assert.ok(
      questSection.events.some(
        (event) =>
          event.kind === "note" &&
          event.pitch % 12 === (rootPitchClass + 4) % 12,
      ),
    );
  });

  it("preserves an already-raised Lydian seventh in the danger treatment", () => {
    const lydianDNA = Array.from({ length: 256 }, (_, index) =>
      createLanternTrailMusicalDNA(
        input(`lydian-danger-${index}`, {
          wonder: 1,
          danger: 1,
          mystery: 0,
          motion: 0.5,
        }),
      ),
    ).find((dna) => dna.harmony.mode === "lydian");
    assert.ok(lydianDNA);

    const danger = resolveLanternTrailSectionHarmony(lydianDNA, "danger");
    assert.equal(lydianDNA.harmony.scaleIntervals[6], 11);
    assert.equal(danger.scaleIntervals[6], 11);
    assert.deepEqual(
      danger.scaleIntervals.slice(0, 6),
      lydianDNA.harmony.scaleIntervals.slice(0, 6),
    );
  });

  it("keeps bass and chord roots on the same resolved progression", () => {
    const { portableScore } = generateLanternTrailAdventure(input("root-agreement"));
    const barTicks = portableScore.beatsPerBar * portableScore.ticksPerBeat;

    for (const section of portableScore.sections) {
      const harmony = section.events.filter(
        (event): event is NoteEvent =>
          event.kind === "note" && event.lane.endsWith("-harmony"),
      );
      const bass = section.events.filter(
        (event): event is NoteEvent =>
          event.kind === "note" && event.lane.endsWith("-bass"),
      );
      for (let bar = 0; bar < 4; bar += 1) {
        const barStart = bar * barTicks;
        const chordPitches = harmony
          .filter((event) => event.startTick === barStart)
          .map((event) => event.pitch);
        const bassPitchClasses = bass
          .filter((event) => event.startTick >= barStart && event.startTick < barStart + barTicks)
          .map((event) => event.pitch % 12);
        assert.ok(chordPitches.length > 0, `${section.id} bar ${bar} has no chord`);
        assert.ok(bassPitchClasses.length > 0, `${section.id} bar ${bar} has no bass`);
        assert.ok(
          bassPitchClasses.includes(Math.min(...chordPitches) % 12),
          `${section.id} bar ${bar} bass misses the chord root`,
        );
      }
    }
  });

  it("replaces climactic placeholder stems with a synthesized glow lane", () => {
    const { portableScore } = generateLanternTrailAdventure(input("glow-contract"));
    const sanctuary = portableScore.sections.find((section) => section.id === "sanctuary");
    assert.ok(sanctuary);
    const glow = sanctuary.events.filter((event) => event.lane === "sanctuary-glow");
    assert.ok(glow.length > 0);
    assert.ok(glow.every((event) => event.kind === "note"));
    assert.ok(sanctuary.events.every((event) => event.kind !== "stem"));
  });

  it("emits no placeholder stem events in generated portable scores", () => {
    for (const seed of ["no-stems-a", "no-stems-b", 42]) {
      const { portableScore } = generateLanternTrailAdventure(input(seed));
      for (const section of portableScore.sections) {
        assert.ok(
          section.events.every((event) => event.kind !== "stem"),
          `${seed}:${section.id}`,
        );
      }
    }
  });

  it("produces a plain portable score accepted by the RNG-free runtime transport", () => {
    const generated = generateLanternTrailAdventure(input("runtime-boundary"));
    const serialized = JSON.stringify(generated.portableScore);
    const portable = JSON.parse(serialized) as PortableScore;

    assert.equal(serialized.includes("pattern"), false);
    assert.equal(serialized.includes("domainSeeds"), false);
    assert.equal(serialized.includes("traits"), false);
    assert.equal(serialized.includes("@strudel"), false);
    assert.equal(serialized.includes("queryArc"), false);
    assert.deepEqual(portable, generated.portableScore);
    validatePortableScore(portable);
    for (const section of portable.sections) {
      const transport = new AdaptiveTransport(portable, section.id);
      const target = section.id === "quest-complete" ? "camp" : "quest-complete";
      const request = transport.requestSection(target, 1);
      assert.equal(request.status, "scheduled");
      if (request.status === "scheduled") {
        assert.equal(request.plan.startTick, 3840);
        assert.equal(request.plan.endTick, 11520);
      }
    }
  });

  it("selects the matching section from adventure game state", () => {
    const score = generateLanternTrailAdventure(input("adaptive-rules")).portableScore;

    const pick = (numeric: Record<string, number>, categorical: Record<string, string>) =>
      selectSection(score, { numeric, categorical });

    assert.equal(pick({}, { areaPhase: "camp" }), "camp");
    assert.equal(pick({}, { areaPhase: "explore" }), "explore");
    assert.equal(pick({}, { areaPhase: "clue" }), "clue");
    assert.equal(pick({}, { areaPhase: "danger" }), "danger");
    assert.equal(pick({}, { areaPhase: "complete" }), "quest-complete");

    assert.equal(pick({ threat: 0.9 }, { areaPhase: "explore" }), "danger");
    assert.equal(pick({ discovery: 0.5 }, { areaPhase: "camp" }), "clue");
    assert.equal(pick({ discovery: 0.95 }, { areaPhase: "explore" }), "sanctuary");
    assert.equal(pick({ questComplete: 1 }, { areaPhase: "danger" }), "quest-complete");

    const transport = new AdaptiveTransport(score);
    const request = transport.requestState(
      { numeric: { threat: 0.9 }, categorical: { areaPhase: "explore" } },
      0,
    );
    assert.equal(request.status, "scheduled");
    if (request.status === "scheduled") {
      assert.equal(request.plan.to, "danger");
    }
  });

  it("shares the authoring and portable score APIs across both recipes", () => {
    const authoring = generateLanternTrailAuthoringScore(input("authoring-api"));
    assert.equal(authoring.defaultSection, "camp");
    assert.equal(authoring.sections.length, 6);
    assert.equal(authoring.rules.length, 9);
    const portable = exportScore(authoring, `${LANTERN_TRAIL_GENERATOR_VERSION}:string:authoring-api`);
    assert.equal(portable.sections.length, 6);
    const sanctuary = portable.sections.find((section) => section.id === "sanctuary");
    assert.ok(sanctuary);
    assert.ok(
      sanctuary.events.some(
        (event) => event.kind === "note" && event.lane === "sanctuary-glow",
      ),
    );
    assert.ok(sanctuary.events.every((event) => event.kind !== "stem"));
  });
});
