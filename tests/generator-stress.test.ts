import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { validatePortableScore } from "../packages/runtime/src/index.ts";
import {
  checksumCompactJson,
  createPocketCircuitGenerationManifest,
} from "../packages/studio/src/generation-manifest.ts";
import {
  generateRacingLevel,
  type PocketCircuitStyle,
} from "../packages/studio/src/index.ts";

const STYLES: readonly PocketCircuitStyle[] = [
  "fusion",
  "neon",
  "funk",
  "chip",
];

describe("Pocket Circuit many-seed stress generation", () => {
  it(
    "generates 256 deterministic valid scores across styles and trait ranges",
    () => {
      const scoreIds = new Set<string>();
      const checksums = new Set<string>();

      for (let index = 0; index < 256; index += 1) {
        const style = STYLES[index % STYLES.length];
        assert.ok(style);
        const input = {
          seed: `stress-${index}`,
          style,
          traits: {
            energy: (index % 17) / 16,
            complexity: (index % 11) / 10,
            brightness: (index % 13) / 12,
            syncopation: (index % 19) / 18,
          },
        } as const;
        const generated = generateRacingLevel(input);
        validatePortableScore(generated.portableScore);

        const manifest = createPocketCircuitGenerationManifest(generated);
        assert.match(manifest.checksum.value, /^[0-9a-f]{64}$/);
        assert.equal(
          manifest.checksum.value,
          checksumCompactJson(generated.portableScore),
        );
        scoreIds.add(generated.portableScore.id);
        checksums.add(manifest.checksum.value);

        if (index % 31 === 0) {
          assert.deepEqual(generateRacingLevel(input), generated);
        }
      }

      assert.equal(scoreIds.size, 256);
      assert.ok(checksums.size >= 250);
    },
    30_000,
  );
});
