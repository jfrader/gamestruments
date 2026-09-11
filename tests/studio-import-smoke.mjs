import assert from "node:assert/strict";
import {
  generateLanternTrailAdventure,
  generateRacingLevel,
  LANTERN_TRAIL_GENERATOR_VERSION,
  POCKET_CIRCUIT_GENERATOR_VERSION,
} from "@gamestruments/studio";

const pocketCircuit = generateRacingLevel({ seed: "node-import-smoke" });
assert.equal(pocketCircuit.generatorVersion, POCKET_CIRCUIT_GENERATOR_VERSION);
assert.equal(pocketCircuit.portableScore.sections.length, 6);

const lanternTrail = generateLanternTrailAdventure({ seed: "node-import-smoke" });
assert.equal(lanternTrail.generatorVersion, LANTERN_TRAIL_GENERATOR_VERSION);
assert.equal(lanternTrail.portableScore.sections.length, 6);
process.stdout.write("studio Node import smoke passed\n");
