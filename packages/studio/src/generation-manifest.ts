import { createHash } from "node:crypto";
import type {
  GeneratedLanternTrailAdventure,
  LanternTrailDomainSeeds,
  NormalizedTrailTraits,
} from "./lantern-trail-generator.js";
import type {
  GeneratedPocketCircuitLevel,
  NormalizedMusicTraits,
  PocketCircuitDomainSeeds,
  PocketCircuitStyle,
} from "./pocket-circuit-generator.js";

export const POCKET_CIRCUIT_MANIFEST_VERSION = 1 as const;
export const POCKET_CIRCUIT_CHECKSUM_ALGORITHM = "sha256" as const;
export const LANTERN_TRAIL_MANIFEST_VERSION = 1 as const;
export const LANTERN_TRAIL_CHECKSUM_ALGORITHM = "sha256" as const;

export interface PocketCircuitGenerationManifest {
  manifestVersion: typeof POCKET_CIRCUIT_MANIFEST_VERSION;
  recipe: "pocket-circuit";
  generatorVersion: GeneratedPocketCircuitLevel["generatorVersion"];
  seed: string;
  style: PocketCircuitStyle;
  traits: NormalizedMusicTraits;
  domainSeeds: PocketCircuitDomainSeeds;
  score: {
    schemaVersion: GeneratedPocketCircuitLevel["portableScore"]["schemaVersion"];
    id: string;
  };
  checksum: {
    algorithm: typeof POCKET_CIRCUIT_CHECKSUM_ALGORITHM;
    encoding: "hex";
    source: "compact-json";
    value: string;
  };
}

export interface LanternTrailGenerationManifest {
  manifestVersion: typeof LANTERN_TRAIL_MANIFEST_VERSION;
  recipe: "lantern-trail";
  generatorVersion: GeneratedLanternTrailAdventure["generatorVersion"];
  seed: string;
  traits: NormalizedTrailTraits;
  domainSeeds: LanternTrailDomainSeeds;
  score: {
    schemaVersion: GeneratedLanternTrailAdventure["portableScore"]["schemaVersion"];
    id: string;
  };
  checksum: {
    algorithm: typeof LANTERN_TRAIL_CHECKSUM_ALGORITHM;
    encoding: "hex";
    source: "compact-json";
    value: string;
  };
}

export function compactJson(value: unknown): string {
  const serialized = JSON.stringify(value);
  if (serialized === undefined) {
    throw new Error("Cannot serialize value as compact JSON");
  }
  return serialized;
}

export function checksumCompactJson(value: unknown): string {
  return createHash(POCKET_CIRCUIT_CHECKSUM_ALGORITHM)
    .update(compactJson(value), "utf8")
    .digest("hex");
}

export function createPocketCircuitGenerationManifest(
  generated: GeneratedPocketCircuitLevel,
): PocketCircuitGenerationManifest {
  return {
    manifestVersion: POCKET_CIRCUIT_MANIFEST_VERSION,
    recipe: "pocket-circuit",
    generatorVersion: generated.generatorVersion,
    seed: generated.levelSeed,
    style: generated.style,
    traits: generated.traits,
    domainSeeds: generated.domainSeeds,
    score: {
      schemaVersion: generated.portableScore.schemaVersion,
      id: generated.portableScore.id,
    },
    checksum: {
      algorithm: POCKET_CIRCUIT_CHECKSUM_ALGORITHM,
      encoding: "hex",
      source: "compact-json",
      value: checksumCompactJson(generated.portableScore),
    },
  };
}

export function createLanternTrailGenerationManifest(
  generated: GeneratedLanternTrailAdventure,
): LanternTrailGenerationManifest {
  return {
    manifestVersion: LANTERN_TRAIL_MANIFEST_VERSION,
    recipe: "lantern-trail",
    generatorVersion: generated.generatorVersion,
    seed: generated.levelSeed,
    traits: generated.traits,
    domainSeeds: generated.domainSeeds,
    score: {
      schemaVersion: generated.portableScore.schemaVersion,
      id: generated.portableScore.id,
    },
    checksum: {
      algorithm: LANTERN_TRAIL_CHECKSUM_ALGORITHM,
      encoding: "hex",
      source: "compact-json",
      value: checksumCompactJson(generated.portableScore),
    },
  };
}
