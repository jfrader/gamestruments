import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { parseArgs } from "node:util";
import path from "node:path";

const nativeLibraries = [
  {
    filename: "libgamestruments_godot.so",
    target: "x86_64-unknown-linux-gnu",
  },
  {
    filename: "gamestruments_godot.dll",
    target: "x86_64-pc-windows-msvc",
  },
  {
    filename: "libgamestruments_godot.dylib",
    target: "aarch64-apple-darwin+x86_64-apple-darwin",
  },
];

function requiredValue(values, name) {
  const value = values[name];
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`--${name} is required`);
  }
  return value;
}

function extractQuotedValue(source, pattern, label) {
  const match = source.match(pattern);
  if (!match) {
    throw new Error(`Could not read ${label}`);
  }
  return match[1];
}

async function sha256(file) {
  return createHash("sha256").update(await readFile(file)).digest("hex");
}

const { values } = parseArgs({
  options: {
    out: { type: "string" },
    "repo-root": { type: "string" },
    "assets-dir": { type: "string" },
    version: { type: "string" },
    commit: { type: "string" },
    ref: { type: "string" },
    provenance: { type: "string" },
    dirty: { type: "string" },
    "godot-version": { type: "string" },
    "workflow-url": { type: "string" },
  },
  strict: true,
});

const repoRoot = path.resolve(requiredValue(values, "repo-root"));
const assetsDir = path.resolve(requiredValue(values, "assets-dir"));
const version = requiredValue(values, "version");
const provenance = requiredValue(values, "provenance");
const allowedProvenance = new Set([
  "release",
  "pull_request",
  "workflow_dispatch",
  "local",
]);
if (!allowedProvenance.has(provenance)) {
  throw new Error(`Unsupported provenance: ${provenance}`);
}

const dirtyValue = requiredValue(values, "dirty");
if (dirtyValue !== "true" && dirtyValue !== "false") {
  throw new Error("--dirty must be true or false");
}

const engineSource = await readFile(
  path.join(repoRoot, "crates", "engine", "src", "pocket_circuit.rs"),
  "utf8",
);
const toolchainSource = await readFile(
  path.join(repoRoot, "rust-toolchain.toml"),
  "utf8",
);
const generatorVersion = extractQuotedValue(
  engineSource,
  /pub const GENERATOR_VERSION: &str = "([^"]+)";/,
  "generator version",
);
const rustVersion = extractQuotedValue(
  toolchainSource,
  /channel\s*=\s*"([^"]+)"/,
  "Rust toolchain version",
);

const libraries = await Promise.all(
  nativeLibraries.map(async ({ filename, target }) => ({
    path: `addons/gamestruments/bin/${filename}`,
    target,
    sha256: await sha256(path.join(assetsDir, filename)),
  })),
);

const workflowUrl = values["workflow-url"] || null;
const manifest = {
  schemaVersion: 1,
  product: "Gamestruments Godot 4 Kit",
  version,
  generatorVersion,
  source: {
    commit: requiredValue(values, "commit"),
    ref: requiredValue(values, "ref"),
    dirty: dirtyValue === "true",
  },
  provenance,
  toolchain: {
    rust: rustVersion,
    godot: requiredValue(values, "godot-version"),
  },
  workflowUrl,
  nativeLibraries: libraries,
};

await writeFile(
  path.resolve(requiredValue(values, "out")),
  `${JSON.stringify(manifest, null, 2)}\n`,
);
