import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { describe, it } from "vitest";
import type { PortableScore } from "../packages/runtime/src/index.ts";
import {
  createLanternTrailGenerationManifest,
  createPocketCircuitGenerationManifest,
  type LanternTrailGenerationManifest,
  type PocketCircuitGenerationManifest,
} from "../packages/studio/src/generation-manifest.ts";
import {
  generateLanternTrailAdventure,
  generatePocketCircuitLevel,
} from "../packages/studio/src/index.ts";

async function readFixture<T>(directory: string, name: string): Promise<T> {
  return JSON.parse(
    await readFile(
      new URL(`./fixtures/${directory}/${name}`, import.meta.url),
      "utf8",
    ),
  ) as T;
}

describe("Pocket Circuit golden generation fixtures", () => {
  it("matches the checked-in score and versioned manifest", async () => {
    const generated = generatePocketCircuitLevel({
      seed: "golden-v1",
      style: "neon",
      traits: {
        energy: 0.73,
        complexity: 0.67,
        brightness: 0.81,
        syncopation: 0.59,
      },
    });
    const [score, manifest] = await Promise.all([
      readFixture<PortableScore>("pocket-circuit", "golden.score.json"),
      readFixture<PocketCircuitGenerationManifest>(
        "pocket-circuit",
        "golden.manifest.json",
      ),
    ]);

    assert.deepEqual(generated.portableScore, score);
    assert.deepEqual(createPocketCircuitGenerationManifest(generated), manifest);
  });
});

describe("Lantern Trail golden generation fixtures", () => {
  it("matches the checked-in score and versioned manifest", async () => {
    const generated = generateLanternTrailAdventure({
      seed: "golden-v1",
      traits: {
        wonder: 0.78,
        danger: 0.64,
        mystery: 0.83,
        motion: 0.57,
      },
    });
    const [score, manifest] = await Promise.all([
      readFixture<PortableScore>("lantern-trail/v1", "golden.score.json"),
      readFixture<LanternTrailGenerationManifest>(
        "lantern-trail/v1",
        "golden.manifest.json",
      ),
    ]);

    assert.deepEqual(generated.portableScore, score);
    assert.deepEqual(createLanternTrailGenerationManifest(generated), manifest);
  });
});
