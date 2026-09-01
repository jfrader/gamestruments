import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

const directory = await mkdtemp(join(tmpdir(), "gamestruments-cli-"));
const cliPath = new URL("../packages/studio/dist/cli.js", import.meta.url);

function runCli(arguments_) {
  return spawnSync(process.execPath, [cliPath.pathname, ...arguments_], {
    encoding: "utf8",
  });
}

async function assertGeneratedFiles(scorePath, manifestPath, recipe) {
  const score = JSON.parse(await readFile(scorePath, "utf8"));
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  const checksum = createHash("sha256")
    .update(JSON.stringify(score), "utf8")
    .digest("hex");
  assert.equal(manifest.manifestVersion, 1);
  assert.equal(manifest.recipe, recipe);
  assert.equal(manifest.checksum.source, "compact-json");
  assert.equal(manifest.checksum.value, checksum);
  return manifest;
}

try {
  const pocketScorePath = join(directory, "pocket.score.json");
  const pocketManifestPath = join(directory, "pocket.manifest.json");
  const pocketResult = runCli([
    "--seed",
    "cli-smoke",
    "--style",
    "chip",
    "--energy",
    "0.8",
    "--complexity",
    "0.7",
    "--brightness",
    "0.75",
    "--syncopation",
    "0.65",
    "--output",
    pocketScorePath,
    "--manifest",
    pocketManifestPath,
    "--pretty",
  ]);
  assert.equal(pocketResult.status, 0, pocketResult.stderr);
  const pocketManifest = await assertGeneratedFiles(
    pocketScorePath,
    pocketManifestPath,
    "pocket-circuit",
  );
  assert.equal(pocketManifest.style, "chip");

  const lanternScorePath = join(directory, "lantern.score.json");
  const lanternManifestPath = join(directory, "lantern.manifest.json");
  const lanternResult = runCli([
    "--recipe",
    "lantern-trail",
    "--seed",
    "cli-smoke",
    "--wonder",
    "0.82",
    "--danger",
    "0.61",
    "--mystery",
    "0.74",
    "--motion",
    "0.57",
    "--output",
    lanternScorePath,
    "--manifest",
    lanternManifestPath,
    "--pretty",
  ]);
  assert.equal(lanternResult.status, 0, lanternResult.stderr);
  const lanternManifest = await assertGeneratedFiles(
    lanternScorePath,
    lanternManifestPath,
    "lantern-trail",
  );
  assert.deepEqual(lanternManifest.traits, {
    wonder: 0.82,
    danger: 0.61,
    mystery: 0.74,
    motion: 0.57,
  });

  const invalidPocket = runCli([
    "--seed",
    "bad",
    "--wonder",
    "0.5",
  ]);
  assert.notEqual(invalidPocket.status, 0);
  assert.match(
    invalidPocket.stderr,
    /^gamestruments-generate: --wonder is only valid with --recipe lantern-trail/,
  );

  const invalidLantern = runCli([
    "--recipe",
    "lantern-trail",
    "--seed",
    "bad",
    "--style",
    "neon",
  ]);
  assert.notEqual(invalidLantern.status, 0);
  assert.match(
    invalidLantern.stderr,
    /^gamestruments-generate: --style is only valid with --recipe pocket-circuit/,
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}

process.stdout.write("generation CLI smoke passed\n");
