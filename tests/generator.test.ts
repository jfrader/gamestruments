import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { AdaptiveTransport, type NoteEvent, type PercussionEvent, type PortableScore } from "../packages/runtime/src/index.ts";
import {
  createPocketCircuitMusicalDNA,
  derivePocketCircuitDomainSeeds,
  derivePocketCircuitSubSeed,
  exportScore,
  arrangementGroove,
  generateRacingLevel,
  melodyDegreesForBar,
  POCKET_CIRCUIT_DNA_SEED_VERSION,
  POCKET_CIRCUIT_GENERATOR_DOMAINS,
  POCKET_CIRCUIT_GENERATOR_VERSION,
  POCKET_CIRCUIT_SECTION_PLANS,
  resolvePocketCircuitSectionHarmony,
  DEFAULT_POCKET_CIRCUIT_TRAITS,
  type NormalizedMusicTraits,
  type PocketCircuitGeneratorInput,
  type PocketCircuitStyle,
} from "../packages/studio/src/index.ts";

const BALANCED_TRAITS: NormalizedMusicTraits = {
  energy: 0.6,
  complexity: 0.58,
  brightness: 0.52,
  syncopation: 0.64,
};

function input(
  seed: PocketCircuitGeneratorInput["seed"],
  traits: NormalizedMusicTraits = BALANCED_TRAITS,
): PocketCircuitGeneratorInput {
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

function firstBarEvents(score: PortableScore, sectionId: string) {
  const section = score.sections.find((candidate) => candidate.id === sectionId);
  assert.ok(section);
  const barTicks = score.beatsPerBar * score.ticksPerBeat;
  return section.events.filter((event) => event.startTick < barTicks);
}

function firstBarStructuralSignature(score: PortableScore, sectionId: string): string {
  return JSON.stringify(
    firstBarEvents(score, sectionId).map((event) => ({
      kind: event.kind,
      lane: event.lane.slice(sectionId.length + 1),
      startTick: event.startTick,
      durationTicks: event.durationTicks,
      ...(event.kind === "note" ? { pitch: event.pitch } : {}),
    })),
  );
}

function firstBarPercussionSignature(score: PortableScore, sectionId: string): string {
  return JSON.stringify(
    firstBarEvents(score, sectionId).flatMap((event) =>
      event.kind === "percussion"
        ? [{ startTick: event.startTick, voice: event.voice }]
        : [],
    ),
  );
}

function firstBarLaneSignature(
  score: PortableScore,
  sectionId: string,
  laneSuffix: string,
): string {
  return JSON.stringify(
    firstBarEvents(score, sectionId)
      .filter(
        (event): event is NoteEvent =>
          event.kind === "note" && event.lane.endsWith(laneSuffix),
      )
      .map((event) => [event.startTick, event.durationTicks, event.pitch]),
  );
}

function firstBarDurationSignature(score: PortableScore, sectionId: string): string {
  return JSON.stringify(
    firstBarEvents(score, sectionId)
      .filter((event) => event.kind === "note")
      .map((event) => event.durationTicks)
      .sort((left, right) => left - right),
  );
}

function stripPresentation(value: unknown): unknown {
  return JSON.parse(
    JSON.stringify(value, (key, nestedValue: unknown) =>
      ["id", "title", "velocity", "voice"].includes(key)
        ? undefined
        : nestedValue,
    ),
  ) as unknown;
}

function directionSignature(values: readonly number[]): number[] {
  return values.slice(1).map((value, index) =>
    Math.sign(value - (values[index] ?? value)),
  );
}

function sectionPlan(sectionId: string) {
  const plan = POCKET_CIRCUIT_SECTION_PLANS.find(
    (candidate) => candidate.id === sectionId,
  );
  assert.ok(plan);
  return plan;
}

function hasThreeConsecutiveSteps(steps: readonly number[]): boolean {
  return steps.some(
    (step, index) =>
      steps[index + 1] === step + 1 && steps[index + 2] === step + 2,
  );
}

describe("Pocket Circuit deterministic generator", () => {
  it("provides stable defaults while accepting partial generation parameters", () => {
    assert.deepEqual(
      generateRacingLevel({ seed: "default-traits" }),
      generateRacingLevel({
        seed: "default-traits",
        traits: DEFAULT_POCKET_CIRCUIT_TRAITS,
      }),
    );
    assert.equal(
      generateRacingLevel({
        seed: "partial-traits",
        traits: { energy: 0.91 },
      }).traits.energy,
      0.91,
    );
  });

  it("reproduces the complete authoring and portable result for the same seed", () => {
    const first = generateRacingLevel(input("kitchen-track-17"));
    const second = generateRacingLevel(input("kitchen-track-17"));

    assert.deepEqual(first, second);
    assert.equal(first.generatorVersion, POCKET_CIRCUIT_GENERATOR_VERSION);
    assert.match(first.authoringScore.id, /generated-v1-10-1-[0-9a-f]{8}$/);
    assert.equal(first.portableScore.id, first.authoringScore.id);
    assert.deepEqual(
      first.portableScore,
      exportScore(
        first.authoringScore,
        `${POCKET_CIRCUIT_GENERATOR_VERSION}:${first.levelSeed}`,
      ),
    );
  });

  it("derives stable independent named domain seeds", () => {
    const seed = "named-domain-contract";
    const domainSeeds = derivePocketCircuitDomainSeeds(seed);
    const reversedDomains = [...POCKET_CIRCUIT_GENERATOR_DOMAINS].reverse();

    for (const domain of reversedDomains) {
      assert.equal(domainSeeds[domain], derivePocketCircuitSubSeed(seed, domain));
    }
    assert.equal(
      new Set(Object.values(domainSeeds)).size,
      POCKET_CIRCUIT_GENERATOR_DOMAINS.length,
    );
    assert.deepEqual(
      createPocketCircuitMusicalDNA(
        input(seed, {
          energy: 0,
          complexity: 0,
          brightness: 0,
          syncopation: 0,
        }),
      ).domainSeeds,
      createPocketCircuitMusicalDNA(
        input(seed, {
          energy: 1,
          complexity: 1,
          brightness: 1,
          syncopation: 1,
        }),
      ).domainSeeds,
    );
  });

  it("keeps the v1.1 DNA seed namespace while versioning v1.8 output", () => {
    const dna = createPocketCircuitMusicalDNA(input("dna-version-contract"));
    const { generatorVersion, ...versionIndependentDNA } = dna;

    assert.equal(POCKET_CIRCUIT_GENERATOR_VERSION, "1.10.1");
    assert.equal(POCKET_CIRCUIT_DNA_SEED_VERSION, "1.1.0");
    assert.equal(generatorVersion, "1.10.1");
    assert.deepEqual(versionIndependentDNA, {
      levelSeed: "string:dna-version-contract",
      style: "fusion",
      traits: BALANCED_TRAITS,
      domainSeeds: {
        harmony: 3645613445,
        motif: 2067650676,
        rhythm: 954024692,
        timbre: 2424834421,
        arrangement: 2945639774,
        ornaments: 26896888,
      },
      harmony: {
        key: "d#",
        rootPitchClass: 3,
        mode: "mixolydian",
        scaleIntervals: [0, 2, 4, 5, 7, 9, 10],
        progressionDegrees: [0, 4, 5, 3],
        chordSize: 3,
      },
      motif: {
        degrees: [0, 3, 2, 5, 4, 1, 1, 0],
        anchorLength: 8,
      },
      rhythm: {
        pulse: "eighth-note",
        stepsPerBar: 8,
        pulseTicks: 480,
        melodyOnsets: [1, 2, 3, 6, 7],
        kickOnsets: [0, 3],
        snareOnsets: [2, 7],
      },
      timbre: {
        harmonyVoice: "warm",
        driveHarmonyVoice: "organ",
        melodyVoice: "epiano",
        liftVoice: "epiano",
        bassVoice: "bass",
      },
      arrangement: {
        bpm: 144,
        bassApproach: "root-octave",
        chordGate: 0.8076000000000001,
        melodyGate: 0.528,
      },
      ornaments: {
        barRotations: [0, 2, 5, 5],
        passingOffsets: [0, -1, -1, 0],
        turnaroundStep: 3,
      },
    });
  });

  it("materially varies every musical domain across level seeds", () => {
    const generated = Array.from({ length: 18 }, (_, index) =>
      generateRacingLevel(input(`diversity-${index}`)),
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
          dna.timbre.liftVoice,
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
      assert.ok(["natural-minor", "dorian", "mixolydian", "lydian"].includes(dna.harmony.mode));
      assert.ok(portableScore.bpm >= 112 && portableScore.bpm <= 150);
      assert.equal(portableScore.beatsPerBar, 4);
      assert.equal(portableScore.ticksPerBeat, 960);
    }
  });

  it("uses style recipes as categorical generation parameters", () => {
    const styles = ["fusion", "neon", "funk", "chip"] as const;
    const generated = styles.map((style) =>
      generateRacingLevel({
        seed: "style-contract",
        style,
        traits: BALANCED_TRAITS,
      }),
    );
    assert.equal(new Set(generated.map(({ portableScore }) => portableScore.id)).size, 4);
    assert.equal(generated[0]?.dna.timbre.melodyVoice, "epiano");
    assert.equal(generated[1]?.dna.timbre.melodyVoice, "supersaw");
    assert.equal(generated[2]?.dna.timbre.melodyVoice, "pluck");
    assert.equal(generated[3]?.dna.timbre.melodyVoice, "chip");
    assert.equal(generated[0]?.dna.timbre.driveHarmonyVoice, "organ");
    assert.equal(generated[3]?.dna.timbre.bassVoice, "triangle");
    assert.equal(
      new Set(
        generated.map(
          ({ dna }) => `${dna.timbre.melodyVoice}:${dna.timbre.driveHarmonyVoice}`,
        ),
      ).size,
      4,
    );
  });

  it("keeps every generated onset on the shared eighth-note grid", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "funk", "chip"];

    for (const style of styles) {
      const { dna, portableScore } = generateRacingLevel({
        ...input(`pulse-grid-${style}`),
        style,
      });
      for (const section of portableScore.sections) {
        for (const event of section.events) {
          assert.equal(
            event.startTick % dna.rhythm.pulseTicks,
            0,
            `${style}:${section.id}:${event.id}`,
          );
        }
      }
    }
  });

  it("uses fusion seventh chords from Grid onward while Garage stays a triad", () => {
    const { portableScore } = generateRacingLevel({
      ...input("fusion-seventh-contract", {
        ...BALANCED_TRAITS,
        complexity: 0.2,
      }),
      style: "fusion",
    });

    for (const [sectionId, expectedPitchClasses] of [
      ["garage", 3],
      ["grid", 4],
      ["cruise", 4],
    ] as const) {
      const harmony = firstBarEvents(portableScore, sectionId).filter(
        (event): event is NoteEvent =>
          event.kind === "note" && event.lane === `${sectionId}-harmony`,
      );
      const chords = new Map<number, NoteEvent[]>();
      for (const event of harmony) {
        const chord = chords.get(event.startTick) ?? [];
        chord.push(event);
        chords.set(event.startTick, chord);
      }
      assert.ok(chords.size > 0, sectionId);
      for (const chord of chords.values()) {
        assert.equal(
          new Set(chord.map((event) => event.pitch % 12)).size,
          expectedPitchClasses,
          `${sectionId}:${chord[0]?.startTick}`,
        );
      }
    }
  });

  it("adds exactly 10 BPM to fusion over the same neon seed and traits", () => {
    const sharedInput = input("fusion-tempo-contract", {
      ...BALANCED_TRAITS,
      energy: 0.35,
    });
    const fusion = createPocketCircuitMusicalDNA({
      ...sharedInput,
      style: "fusion",
    });
    const neon = createPocketCircuitMusicalDNA({
      ...sharedInput,
      style: "neon",
    });

    assert.equal(fusion.arrangement.bpm, neon.arrangement.bpm + 10);
  });

  it("gives Race Flow a held-pad groove instead of stacked stabs", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "chip"];

    for (const style of styles) {
      const { dna, portableScore } = generateRacingLevel({
        ...input("race-groove-contract"),
        style,
      });
      const firstBar = firstBarEvents(portableScore, "cruise");
      const harmony = firstBar.filter(
        (event): event is NoteEvent =>
          event.kind === "note" && event.lane === "cruise-harmony",
      );
      const bass = firstBar.filter(
        (event): event is NoteEvent =>
          event.kind === "note" && event.lane === "cruise-bass",
      );
      const melody = firstBar.filter(
        (event): event is NoteEvent =>
          event.kind === "note" && event.role === "melody",
      );
      const kit = firstBar.filter((event) => event.kind === "percussion");
      const harmonyStarts = [...new Set(harmony.map((event) => event.startTick))];
      const kickSteps = kit
        .filter((event) => event.voice === "kick")
        .map((event) => event.startTick / dna.rhythm.pulseTicks);
      const snareSteps = kit
        .filter((event) => event.voice === "snare")
        .map((event) => event.startTick / dna.rhythm.pulseTicks);

      assert.deepEqual(harmonyStarts, [0], style);
      assert.ok(harmony[0] !== undefined && harmony[0].durationTicks > 2000, style);
      assert.deepEqual(
        bass.map((event) => event.startTick / dna.rhythm.pulseTicks),
        [0, 4],
        style,
      );
      assert.ok(melody.length >= 1 && melody.length <= 3, `${style}:melody`);
      assert.deepEqual(kickSteps, [0, 4], style);
      assert.deepEqual(snareSteps, [2, 6], style);
      assert.ok(harmony.every((event) => event.voice === dna.timbre.harmonyVoice));
      assert.ok(melody.every((event) => event.voice === dna.timbre.melodyVoice));
    }
  });

  it("moves the denser funk groove onto Race Flow", () => {
    const { portableScore } = generateRacingLevel({
      ...input("funk-race-groove-contract"),
      style: "funk",
    });
    const gridMelody = firstBarEvents(portableScore, "grid").filter(
      (event) => event.kind === "note" && event.role === "melody",
    );
    const raceMelody = firstBarEvents(portableScore, "cruise").filter(
      (event) => event.kind === "note" && event.role === "melody",
    );
    assert.ok(raceMelody.length > gridMelody.length);
  });

  it("gives Position Fight and Final Lap held pads and two-note hooks", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "funk", "chip"];

    for (const style of styles) {
      const { dna, portableScore } = generateRacingLevel({
        ...input("pressure-final-groove-contract"),
        style,
      });
      for (const sectionId of ["attack", "final-lap"] as const) {
        const firstBar = firstBarEvents(portableScore, sectionId);
        const harmony = firstBar.filter(
          (event): event is NoteEvent =>
            event.kind === "note" && event.lane.endsWith("-harmony"),
        );
        const bass = firstBar.filter(
          (event): event is NoteEvent =>
            event.kind === "note" && event.lane.endsWith("-bass"),
        );
        const melody = firstBar.filter(
          (event): event is NoteEvent =>
            event.kind === "note" && event.role === "melody",
        );
        const harmonyStarts = [...new Set(harmony.map((event) => event.startTick))];

        assert.deepEqual(harmonyStarts, [0], `${style}:${sectionId}`);
        assert.ok(
          harmony[0] !== undefined && harmony[0].durationTicks > 1400,
          `${style}:${sectionId}:held`,
        );
        assert.deepEqual(
          bass.map((event) => event.startTick / dna.rhythm.pulseTicks),
          [0, 4],
          `${style}:${sectionId}:bass`,
        );
        assert.ok(melody.length <= 3, `${style}:${sectionId}:melody`);
      }
    }
  });

  it("avoids adjacent repeated Race Flow melody pitches in every style", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "funk", "chip"];
    const seeds = ["level-001", "race-phrasing-a", "race-phrasing-b", 8042];

    for (const style of styles) {
      for (const seed of seeds) {
        const { portableScore } = generateRacingLevel({
          ...input(seed),
          style,
        });
        const cruise = portableScore.sections.find(
          (section) => section.id === "cruise",
        );
        assert.ok(cruise);
        const barTicks = portableScore.beatsPerBar * portableScore.ticksPerBeat;

        for (let barIndex = 0; barIndex < 4; barIndex += 1) {
          const barStart = barIndex * barTicks;
          const melody = cruise.events
            .filter(
              (event): event is NoteEvent =>
                event.kind === "note" &&
                event.role === "melody" &&
                event.startTick >= barStart &&
                event.startTick < barStart + barTicks,
            )
            .sort((left, right) => left.startTick - right.startTick);

          assert.ok(
            melody.slice(1).every(
              (event, index) => event.pitch !== melody[index]?.pitch,
            ),
            `${style}:${seed}:bar-${barIndex}`,
          );
        }
      }
    }
  });

  it("breaks three-eighth Race Flow melody runs before style phrasing", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "funk"];
    const seeds = ["level-001", "race-run-a", "race-run-b", 8042];
    const traitSets: readonly NormalizedMusicTraits[] = [
      BALANCED_TRAITS,
      { energy: 0.62, complexity: 0.68, brightness: 0.52, syncopation: 0.72 },
    ];

    for (const style of styles) {
      for (const seed of seeds) {
        for (const traits of traitSets) {
          const generated = generateRacingLevel({
            seed,
            style,
            traits,
          });
          const straight = exportScore(
            generated.authoringScore,
            `${POCKET_CIRCUIT_GENERATOR_VERSION}:${generated.levelSeed}`,
          );
          const cruise = straight.sections.find(
            (section) => section.id === "cruise",
          );
          assert.ok(cruise);
          const barTicks = straight.beatsPerBar * straight.ticksPerBeat;

          for (let barIndex = 0; barIndex < 4; barIndex += 1) {
            const barStart = barIndex * barTicks;
            const steps = cruise.events
              .filter(
                (event) =>
                  event.kind === "note" &&
                  event.role === "melody" &&
                  event.startTick >= barStart &&
                  event.startTick < barStart + barTicks,
              )
              .map(
                (event) =>
                  (event.startTick - barStart) / generated.dna.rhythm.pulseTicks,
              )
              .sort((left, right) => left - right);

            assert.equal(
              hasThreeConsecutiveSteps(steps),
              false,
              `${style}:${seed}:${JSON.stringify(traits)}:bar-${barIndex}:${steps.join(",")}`,
            );
          }
        }
      }
    }
  });

  it("keeps Grid snares on steps 2 and 6 even when DNA wants a step-6 kick", () => {
    for (const seed of ["grid-snare-a", "grid-snare-b", 8042, "level-001"]) {
      const { dna, portableScore } = generateRacingLevel(input(seed));
      const grid = portableScore.sections.find((section) => section.id === "grid");
      assert.ok(grid);
      const barTicks = portableScore.beatsPerBar * portableScore.ticksPerBeat;
      const lateBar = grid.events.filter(
        (event): event is PercussionEvent =>
          event.kind === "percussion" &&
          event.startTick >= barTicks * 2 &&
          event.startTick < barTicks * 3,
      );
      const snareSteps = lateBar
        .filter((event) => event.voice === "snare")
        .map((event) => (event.startTick % barTicks) / dna.rhythm.pulseTicks)
        .sort((left, right) => left - right);
      const kickSteps = lateBar
        .filter((event) => event.voice === "kick")
        .map((event) => (event.startTick % barTicks) / dna.rhythm.pulseTicks);

      assert.deepEqual(snareSteps, [2, 6], String(seed));
      assert.equal(kickSteps.includes(6), false, String(seed));
    }
  });

  it("keeps version strings from altering non-fusion events beyond the score id", () => {
    for (const style of ["neon", "funk", "chip"] as const) {
      const generated = generateRacingLevel({
        ...input(`v1-8-non-fusion-${style}`),
        style,
      });
      const legacyAuthoringScore = {
        ...generated.authoringScore,
        id: generated.authoringScore.id.replace("v1-10-1", "v1-9-0"),
      };
      const legacyScore = exportScore(
        legacyAuthoringScore,
        `1.9.0:${generated.levelSeed}`,
      );

      assert.equal(
        generated.portableScore.id.replace("v1-10-1", "v1-9-0"),
        legacyScore.id,
      );
      assert.deepEqual(
        { ...generated.portableScore, id: legacyScore.id },
        legacyScore,
        style,
      );
    }
  });

  it("adds a fusion-only four-bar victory sparkle lane", () => {
    for (const style of ["fusion", "neon", "funk", "chip"] as const) {
      const { dna, portableScore } = generateRacingLevel({
        ...input("victory-sparkle-contract"),
        style,
      });
      const victory = portableScore.sections.find((section) => section.id === "victory");
      assert.ok(victory);
      const sparkle = victory.events.filter(
        (event): event is NoteEvent =>
          event.kind === "note" && event.lane === "victory-sparkle",
      );

      if (style !== "fusion") {
        assert.equal(sparkle.length, 0, style);
        continue;
      }

      const barTicks = portableScore.beatsPerBar * portableScore.ticksPerBeat;
      const expectedPitches = [0, 4, 7, 12].map(
        (offset) => 72 + dna.harmony.rootPitchClass + offset,
      );
      assert.equal(sparkle.length, 16);
      assert.ok(sparkle.every((event) => event.voice === "glass"));
      assert.ok(sparkle.every((event) => event.durationTicks === 240));
      assert.ok(sparkle.every((event) => event.velocity === 0.5192));
      for (let barIndex = 0; barIndex < 4; barIndex += 1) {
        const bar = sparkle.filter(
          (event) =>
            event.startTick >= barIndex * barTicks &&
            event.startTick < (barIndex + 1) * barTicks,
        );
        assert.deepEqual(
          bar.map((event) => event.startTick - barIndex * barTicks),
          [0, 960, 1920, 2880],
        );
        assert.deepEqual(
          bar.map((event) => event.pitch),
          expectedPitches,
        );
      }
    }
  });

  it("raises only the fusion victory melody by one effective register", () => {
    const sharedInput = input("fusion-victory-register-contract");
    const fusion = generateRacingLevel({
      ...sharedInput,
      style: "fusion",
    }).portableScore;
    const neon = generateRacingLevel({
      ...sharedInput,
      style: "neon",
    }).portableScore;

    for (const sectionId of ["garage", "grid", "cruise", "attack", "final-lap"]) {
      const fusionPitches = firstBarEvents(fusion, sectionId)
        .filter((event): event is NoteEvent => event.kind === "note" && event.role === "melody")
        .map((event) => event.pitch);
      const neonPitches = firstBarEvents(neon, sectionId)
        .filter((event): event is NoteEvent => event.kind === "note" && event.role === "melody")
        .map((event) => event.pitch);
      assert.ok(fusionPitches.length > 0 && neonPitches.length > 0, sectionId);
      assert.ok(
        Math.abs(average(fusionPitches) - average(neonPitches)) < 12,
        `${sectionId}: fusion ${average(fusionPitches)} vs neon ${average(neonPitches)}`,
      );
    }

    const fusionVictory = firstBarEvents(fusion, "victory")
      .filter((event): event is NoteEvent => event.kind === "note" && event.role === "melody")
      .map((event) => event.pitch);
    const neonVictory = firstBarEvents(neon, "victory")
      .filter((event): event is NoteEvent => event.kind === "note" && event.role === "melody")
      .map((event) => event.pitch);
    assert.deepEqual(
      fusionVictory,
      neonVictory.map((pitch) => pitch + 12),
    );
  });

  it("gives every style six immediately distinct first-bar phase silhouettes", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "funk", "chip"];
    const sectionIds = ["garage", "grid", "cruise", "attack", "final-lap", "victory"];

    for (const style of styles) {
      for (const seed of ["phase-a", "phase-b", 8042]) {
        const { portableScore } = generateRacingLevel({
          ...input(seed),
          style,
        });
        const signatures = sectionIds.map((sectionId) =>
          firstBarStructuralSignature(portableScore, sectionId),
        );
        assert.equal(new Set(signatures).size, 6, `${style}:${seed}`);
      }
    }
  });

  it("makes Grid immediately denser and structurally distinct from Garage in every voice", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "chip"];

    for (const style of styles) {
      for (const seed of ["level-001", "grid-opening-a", "grid-opening-b", 8042]) {
        const { portableScore } = generateRacingLevel({
          ...input(seed),
          style,
        });
        const garage = firstBarEvents(portableScore, "garage");
        const grid = firstBarEvents(portableScore, "grid");
        const garageNotes = garage.filter((event) => event.kind === "note");
        const gridNotes = grid.filter((event) => event.kind === "note");
        const gridMelody = grid.filter(
          (event) => event.kind === "note" && event.role === "melody",
        );
        const gridHarmony = grid.filter(
          (event) => event.kind === "note" && event.lane === "grid-harmony",
        );
        const gridBass = grid.filter(
          (event) => event.kind === "note" && event.lane === "grid-bass",
        );
        const garageKit = garage.filter((event) => event.kind === "percussion");
        const gridKit = grid.filter((event) => event.kind === "percussion");
        const context = `${style}:${seed}`;
        const gridSection = portableScore.sections.find((section) => section.id === "grid");
        assert.ok(gridSection);
        const barTicks = portableScore.beatsPerBar * portableScore.ticksPerBeat;
        const gridBarDensities = Array.from({ length: 4 }, (_, barIndex) =>
          gridSection.events.filter(
            (event) =>
              event.kind === "note" &&
              event.startTick >= barIndex * barTicks &&
              event.startTick < (barIndex + 1) * barTicks,
          ).length,
        );

        assert.ok(gridNotes.length > garageNotes.length, `${context}:density`);
        assert.ok(
          gridBarDensities.slice(1).every(
            (density, index) => density > (gridBarDensities[index] ?? density),
          ),
          `${context}:bar-build`,
        );
        assert.ok(gridMelody.length >= 3, `${context}:melody-countdown`);
        assert.ok(gridMelody[0] && gridMelody.at(-1));
        assert.ok(gridMelody[0].startTick < 1920, `${context}:countdown-start`);
        assert.ok((gridMelody.at(-1)?.startTick ?? 0) >= 2880, `${context}:anticipation-end`);
        assert.ok(
          new Set(gridHarmony.map((event) => event.startTick)).size >= 2,
          `${context}:harmony-articulation`,
        );
        assert.ok(
          new Set(gridBass.map((event) => event.startTick)).size >= 2,
          `${context}:bass-articulation`,
        );
        assert.ok(gridKit.length > garageKit.length, `${context}:kit-density`);
        assert.ok(
          gridKit.some((event) => event.voice === "snare"),
          `${context}:backbeat`,
        );

        assert.notEqual(
          firstBarLaneSignature(portableScore, "garage", "-melody"),
          firstBarLaneSignature(portableScore, "grid", "-melody"),
          `${context}:melody-signature`,
        );
        assert.notEqual(
          firstBarLaneSignature(portableScore, "garage", "-harmony"),
          firstBarLaneSignature(portableScore, "grid", "-harmony"),
          `${context}:harmony-signature`,
        );
        assert.notEqual(
          firstBarLaneSignature(portableScore, "garage", "-bass"),
          firstBarLaneSignature(portableScore, "grid", "-bass"),
          `${context}:bass-signature`,
        );
        assert.notEqual(
          firstBarPercussionSignature(portableScore, "garage"),
          firstBarPercussionSignature(portableScore, "grid"),
          `${context}:percussion-signature`,
        );
        assert.notEqual(
          firstBarDurationSignature(portableScore, "garage"),
          firstBarDurationSignature(portableScore, "grid"),
          `${context}:duration-signature`,
        );
      }
    }
  });

  it("restores the sparse-to-groove-to-climax density and register arc", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "funk", "chip"];

    for (const style of styles) {
      for (const seed of ["phase-arc-a", "phase-arc-b"]) {
        const { portableScore } = generateRacingLevel({
          ...input(seed),
          style,
        });
        const phaseNotes = ["garage", "cruise", "final-lap"].map((sectionId) =>
          firstBarEvents(portableScore, sectionId).flatMap((event) =>
            event.kind === "note" ? [event] : [],
          ),
        );
        const phaseMelodies = phaseNotes.map((events) =>
          events.filter((event) => event.role === "melody"),
        );

        assert.ok(phaseNotes[0] && phaseNotes[1] && phaseNotes[2]);
        assert.ok(phaseNotes[0].length < phaseNotes[1].length, `${style}:${seed}:garage`);
        if (style !== "funk") {
          assert.ok(phaseNotes[1].length < phaseNotes[2].length, `${style}:${seed}:final-lap`);
        }
        assert.ok(phaseMelodies[0] && phaseMelodies[1] && phaseMelodies[2]);
        assert.ok(
          average(phaseMelodies[0].map((event) => event.pitch)) <
            average(phaseMelodies[1].map((event) => event.pitch)),
          `${style}:${seed}:cruise-register`,
        );
        assert.ok(
          average(phaseMelodies[1].map((event) => event.pitch)) <
            average(phaseMelodies[2].map((event) => event.pitch)),
          `${style}:${seed}:final-register`,
        );
      }
    }
  });

  it("uses distinct phase duration and percussion vocabularies", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "funk", "chip"];
    const sectionIds = ["garage", "grid", "cruise", "attack", "final-lap", "victory"];

    for (const style of styles) {
      const { portableScore } = generateRacingLevel({
        ...input("phase-rhythm-contract"),
        style,
      });
      const durationSignatures = sectionIds.map((sectionId) =>
        JSON.stringify(
          firstBarEvents(portableScore, sectionId)
            .filter((event) => event.kind === "note")
            .map((event) => event.durationTicks)
            .sort((left, right) => left - right),
        ),
      );
      const percussionSignatures = sectionIds.map((sectionId) =>
        firstBarPercussionSignature(portableScore, sectionId),
      );

      assert.ok(new Set(durationSignatures).size >= 4, style);
      assert.equal(new Set(percussionSignatures).size, 6, style);
    }
  });

  it("makes all four styles structurally distinct beyond presentation and voices", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "funk", "chip"];
    const structures = styles.map((style) =>
      JSON.stringify(
        stripPresentation(
          generateRacingLevel({
            ...input("style-structure-contract"),
            style,
          }).portableScore,
        ),
      ),
    );

    assert.equal(new Set(structures).size, 4);
  });

  it("keeps the motif contour recognizable and every style transformation in resolved harmony", () => {
    const styles: readonly PocketCircuitStyle[] = ["fusion", "neon", "funk", "chip"];

    for (const style of styles) {
      for (const seed of ["transformation-harmony-a", "transformation-harmony-b"]) {
        const { dna, portableScore } = generateRacingLevel({
          ...input(seed),
          style,
        });
        for (const section of portableScore.sections) {
          const resolved = resolvePocketCircuitSectionHarmony(dna, section.id);
          const allowedPitchClasses = new Set(
            resolved.scaleIntervals.map(
              (interval) => (dna.harmony.rootPitchClass + interval) % 12,
            ),
          );
          const firstBarMelody = firstBarEvents(portableScore, section.id).flatMap(
            (event) =>
              event.kind === "note" && event.role === "melody"
                ? [event.pitch]
                : [],
          );

          assert.ok(
            section.events.every(
              (event) => event.kind !== "note" || allowedPitchClasses.has(event.pitch % 12),
            ),
            `${style}:${seed}:${section.id}:harmony`,
          );
          assert.deepEqual(
            directionSignature(firstBarMelody),
            directionSignature(
              melodyDegreesForBar(
              dna,
              arrangementGroove(style, sectionPlan(section.id)),
              0,
            ),
            ),
            `${style}:${seed}:${section.id}:contour`,
          );
        }
      }
    }
  });

  it("gives each style a held final-lap lift layer instead of an arpeggio", () => {
    const signatures: string[] = [];

    for (const style of ["fusion", "neon", "funk", "chip"] as const) {
      const { portableScore } = generateRacingLevel({
        ...input("style-lift-contract"),
        style,
      });
      const lift = firstBarEvents(portableScore, "final-lap").filter(
        (event): event is NoteEvent =>
          event.kind === "note" && event.lane === "final-lap-lift",
      );
      assert.equal(lift.length, 1, style);
      assert.equal(lift[0]?.startTick, 0, style);
      assert.ok((lift[0]?.durationTicks ?? 0) > 2000, style);
      signatures.push(
        JSON.stringify(
          lift.map((event) => [event.startTick, event.durationTicks, event.pitch]),
        ),
      );
    }

    assert.equal(new Set(signatures).size, 4);
  });

  it("gives each normalized trait an observable deterministic effect", () => {
    const base = { ...BALANCED_TRAITS };
    const energyLow = generateRacingLevel(
      input("trait-contract", { ...base, energy: 0.1 }),
    );
    const energyHigh = generateRacingLevel(
      input("trait-contract", { ...base, energy: 0.9 }),
    );
    assert.ok(energyHigh.portableScore.bpm > energyLow.portableScore.bpm);
    assert.ok(
      average(energyHigh.portableScore.sections.flatMap((section) => section.events.map((event) => event.velocity))) >
        average(energyLow.portableScore.sections.flatMap((section) => section.events.map((event) => event.velocity))),
    );

    const complexityLow = generateRacingLevel(
      input("trait-contract", { ...base, complexity: 0.1 }),
    );
    const complexityHigh = generateRacingLevel(
      input("trait-contract", { ...base, complexity: 0.9 }),
    );
    assert.ok(
      melodyEvents(complexityHigh.portableScore).length >
        melodyEvents(complexityLow.portableScore).length,
    );
    assert.equal(complexityLow.dna.harmony.chordSize, 3);
    assert.equal(complexityHigh.dna.harmony.chordSize, 4);

    const brightnessLow = generateRacingLevel(
      input("trait-contract", { ...base, brightness: 0.1 }),
    );
    const brightnessHigh = generateRacingLevel(
      input("trait-contract", { ...base, brightness: 0.9 }),
    );
    assert.ok(
      average(melodyEvents(brightnessHigh.portableScore).map((event) => event.pitch)) >
        average(melodyEvents(brightnessLow.portableScore).map((event) => event.pitch)),
    );
    assert.ok(["natural-minor", "dorian"].includes(brightnessLow.dna.harmony.mode));
    assert.ok(["mixolydian", "lydian"].includes(brightnessHigh.dna.harmony.mode));

    const syncopationLow = createPocketCircuitMusicalDNA(
      input("trait-contract", { ...base, syncopation: 0.1 }),
    );
    const syncopationHigh = createPocketCircuitMusicalDNA(
      input("trait-contract", { ...base, syncopation: 0.9 }),
    );
    const lowOffbeats = syncopationLow.rhythm.melodyOnsets.filter(
      (step) => step % 2 === 1,
    ).length;
    const highOffbeats = syncopationHigh.rhythm.melodyOnsets.filter(
      (step) => step % 2 === 1,
    ).length;
    assert.ok(highOffbeats > lowOffbeats);
    assert.ok(
      syncopationHigh.arrangement.melodyGate <
        syncopationLow.arrangement.melodyGate,
    );

    const denseStraight = createPocketCircuitMusicalDNA(
      input("dense-syncopation", { ...base, complexity: 1, syncopation: 0 }),
    );
    const denseSyncopated = createPocketCircuitMusicalDNA(
      input("dense-syncopation", { ...base, complexity: 1, syncopation: 1 }),
    );
    assert.notDeepEqual(
      denseStraight.rhythm.melodyOnsets,
      denseSyncopated.rhythm.melodyOnsets,
    );
    assert.ok(
      denseSyncopated.rhythm.melodyOnsets.filter((step) => step % 2 === 1).length >
        denseStraight.rhythm.melodyOnsets.filter((step) => step % 2 === 1).length,
    );
  });

  it("keeps generated kick and snare onsets disjoint", () => {
    for (let index = 0; index < 64; index += 1) {
      const { rhythm } = createPocketCircuitMusicalDNA(input(`drum-overlap-${index}`));
      assert.equal(
        rhythm.kickOnsets.some((step) => rhythm.snareOnsets.includes(step)),
        false,
      );
    }
  });

  it("always anchors rhythm DNA with a downbeat and one distinct syncopated kick", () => {
    for (let index = 0; index < 512; index += 1) {
      const { rhythm } = createPocketCircuitMusicalDNA(
        input(`kick-contract-${index}`),
      );

      assert.equal(rhythm.kickOnsets.length, 2);
      assert.equal(rhythm.kickOnsets[0], 0);
      assert.ok([3, 4, 6].includes(rhythm.kickOnsets[1] ?? -1));
      assert.notEqual(rhythm.kickOnsets[0], rhythm.kickOnsets[1]);
    }
  });

  it("keeps BPM within 112..150 across broad seeds and energy extremes", () => {
    for (let index = 0; index < 256; index += 1) {
      for (const energy of [0, 1]) {
        const { arrangement } = createPocketCircuitMusicalDNA(
          input(`bpm-contract-${index}`, {
            ...BALANCED_TRAITS,
            energy,
          }),
        );
        assert.ok(arrangement.bpm >= 112, `${index}:${energy}:minimum`);
        assert.ok(arrangement.bpm <= 150, `${index}:${energy}:maximum`);
      }
    }
  });

  it("exports six non-empty sections with bounded sorted integer events", () => {
    const { portableScore } = generateRacingLevel(
      input(8042, {
        energy: 0.82,
        complexity: 0.76,
        brightness: 0.68,
        syncopation: 0.71,
      }),
    );

    assert.deepEqual(
      portableScore.sections.map((section) => section.id),
      ["garage", "grid", "cruise", "attack", "final-lap", "victory"],
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

  it("develops the recognizable shared motif through every phase without losing harmony or pulse", () => {
    const { dna, portableScore } = generateRacingLevel(
      input("shared-musical-dna"),
    );
    const barTicks = portableScore.beatsPerBar * portableScore.ticksPerBeat;

    for (const section of portableScore.sections) {
      const resolved = resolvePocketCircuitSectionHarmony(dna, section.id);
      const allowedPitchClasses = new Set(
        resolved.scaleIntervals.map(
          (interval) => (dna.harmony.rootPitchClass + interval) % 12,
        ),
      );
      const melody = section.events.flatMap((event) =>
        event.kind === "note" && event.role === "melody" ? [event] : [],
      );
      assert.ok(melody.length >= 8, section.id);
      assert.ok(melody.some((event) => event.startTick >= barTicks * 3), section.id);
      assert.ok(new Set(melody.map((event) => event.pitch)).size >= 2, section.id);
      assert.ok(
        melody.every((event) => {
          const pulseOffset = event.startTick % dna.rhythm.pulseTicks;
          return pulseOffset === 0;
        }),
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
      assert.ok(new Set(barSignatures).size >= 2, section.id);

      const firstBarPitches = melody
        .filter((event) => event.startTick < barTicks)
        .map((event) => event.pitch);
      assert.deepEqual(
        directionSignature(firstBarPitches),
        directionSignature(
          melodyDegreesForBar(dna, sectionPlan(section.id), 0),
        ),
        section.id,
      );
    }
  });

  it("applies section-specific harmony treatments while preserving tonic", () => {
    const { dna, portableScore } = generateRacingLevel(
      input("section-harmony", {
        ...BALANCED_TRAITS,
        brightness: 0.1,
        complexity: 0.3,
      }),
    );
    const rootPitchClass = dna.harmony.rootPitchClass;
    const sections = new Map(portableScore.sections.map((section) => [section.id, section]));

    // garage, grid, and cruise share one resolved harmony
    const shared = resolvePocketCircuitSectionHarmony(dna, "garage");
    assert.deepEqual(resolvePocketCircuitSectionHarmony(dna, "grid"), shared);
    assert.deepEqual(resolvePocketCircuitSectionHarmony(dna, "cruise"), shared);

    // attack raises the seventh (leading-tone pressure) but keeps the base progression
    const attack = resolvePocketCircuitSectionHarmony(dna, "attack");
    assert.notDeepEqual(attack.scaleIntervals, dna.harmony.scaleIntervals);
    assert.deepEqual(attack.progressionDegrees, dna.harmony.progressionDegrees);

    // final-lap adds dominant pressure: the raised leading tone (root + 11) is sounded
    const finalLap = sections.get("final-lap");
    assert.ok(finalLap);
    assert.ok(
      finalLap.events.some(
        (event) =>
          event.kind === "note" &&
          event.pitch % 12 === (rootPitchClass + 11) % 12,
      ),
    );

    // victory uses parallel major: the major third (root + 4) is the release color
    const victory = sections.get("victory");
    assert.ok(victory);
    assert.ok(
      victory.events.some(
        (event) =>
          event.kind === "note" &&
          event.pitch % 12 === (rootPitchClass + 4) % 12,
      ),
    );
  });

  it("does not wrap an already-raised seventh back to tonic", () => {
    const lydianDNA = Array.from({ length: 128 }, (_, index) =>
      createPocketCircuitMusicalDNA(
        input(`lydian-seventh-${index}`, {
          ...BALANCED_TRAITS,
          brightness: 1,
        }),
      ),
    ).find((dna) => dna.harmony.mode === "lydian");
    assert.ok(lydianDNA);

    const attack = resolvePocketCircuitSectionHarmony(lydianDNA, "attack");
    assert.equal(lydianDNA.harmony.scaleIntervals[6], 11);
    assert.equal(attack.scaleIntervals[6], 11);
  });

  it("preserves each base mode while adding final-lap dominant pressure", () => {
    const modes = ["natural-minor", "dorian", "mixolydian", "lydian"] as const;
    const examples = new Map<
      (typeof modes)[number],
      ReturnType<typeof createPocketCircuitMusicalDNA>
    >();

    for (const brightness of [0, 0.5, 1]) {
      for (let index = 0; index < 256; index += 1) {
        const dna = createPocketCircuitMusicalDNA(
          input(`final-lap-mode-${brightness}-${index}`, {
            ...BALANCED_TRAITS,
            brightness,
          }),
        );
        examples.set(dna.harmony.mode, dna);
      }
    }

    assert.deepEqual([...examples.keys()].sort(), [...modes].sort());
    for (const mode of modes) {
      const dna = examples.get(mode);
      assert.ok(dna);
      const finalLap = resolvePocketCircuitSectionHarmony(dna, "final-lap");

      assert.deepEqual(
        finalLap.scaleIntervals.slice(0, 6),
        dna.harmony.scaleIntervals.slice(0, 6),
        mode,
      );
      assert.equal(
        finalLap.scaleIntervals[6],
        Math.min(11, (dna.harmony.scaleIntervals[6] ?? 10) + 1),
        mode,
      );
      assert.deepEqual(
        finalLap.progressionDegrees.slice(0, 3),
        dna.harmony.progressionDegrees.slice(0, 3),
        mode,
      );
      assert.equal(finalLap.progressionDegrees[3], 4, mode);
      assert.equal(finalLap.chordSize, dna.harmony.chordSize, mode);
    }
  });

  it("keeps bass and chord roots on the same resolved progression", () => {
    const { portableScore } = generateRacingLevel(input("root-agreement"));
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

  it("replaces climactic placeholder stems with a synthesized lift lane", () => {
    const { portableScore } = generateRacingLevel(input("lift-contract"));
    const finalLap = portableScore.sections.find((section) => section.id === "final-lap");
    assert.ok(finalLap);
    const lift = finalLap.events.filter((event) => event.lane === "final-lap-lift");
    assert.ok(lift.length > 0);
    assert.ok(lift.every((event) => event.kind === "note"));
    assert.ok(finalLap.events.every((event) => event.kind !== "stem"));
  });

  it("emits no placeholder stem events in generated portable scores", () => {
    for (const seed of ["no-stems-a", "no-stems-b", 42]) {
      const { portableScore } = generateRacingLevel(input(seed));
      for (const section of portableScore.sections) {
        assert.ok(
          section.events.every((event) => event.kind !== "stem"),
          `${seed}:${section.id}`,
        );
      }
    }
  });

  it("produces a plain portable score accepted by the RNG-free runtime transport", () => {
    const generated = generateRacingLevel(input("runtime-boundary"));
    const serialized = JSON.stringify(generated.portableScore);
    const portable = JSON.parse(serialized) as PortableScore;

    assert.equal(serialized.includes("pattern"), false);
    assert.equal(serialized.includes("domainSeeds"), false);
    assert.equal(serialized.includes("traits"), false);
    assert.deepEqual(portable, generated.portableScore);
    for (const section of portable.sections) {
      const transport = new AdaptiveTransport(portable, section.id);
      const target = section.id === "victory" ? "garage" : "victory";
      const request = transport.requestSection(target, 1);
      assert.equal(request.status, "scheduled");
      if (request.status === "scheduled") {
        assert.equal(request.plan.startTick, 3840);
        assert.equal(request.plan.endTick, 11520);
      }
    }
  });
});
