import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const [rootArgument, archiveName] = process.argv.slice(2);
if (!rootArgument || !archiveName) {
  throw new Error("Usage: node tests/verify-release-manifest.mjs <archive-root> <archive-name>");
}

const root = path.resolve(rootArgument);
const manifest = JSON.parse(await readFile(path.join(root, "RELEASE-MANIFEST.json"), "utf8"));
const expectedArchiveName = `gamestruments-${manifest.version}-godot4.zip`;
if (archiveName !== expectedArchiveName) {
  throw new Error(`Release manifest version expects ${expectedArchiveName}, got ${archiveName}`);
}
if (manifest.schemaVersion !== 1 || manifest.product !== "Gamestruments Godot 4 Kit") {
  throw new Error("Release manifest has an unsupported identity or schema");
}
if (!/^[0-9a-f]{40}$/.test(manifest.source?.commit ?? "")) {
  throw new Error("Release manifest source commit is not a full Git SHA");
}
if (typeof manifest.source?.ref !== "string" || manifest.source.ref.length === 0) {
  throw new Error("Release manifest source ref is missing");
}
if (typeof manifest.source?.dirty !== "boolean") {
  throw new Error("Release manifest source dirty flag is missing");
}
const provenanceValues = new Set(["release", "pull_request", "workflow_dispatch", "local"]);
if (!provenanceValues.has(manifest.provenance)) {
  throw new Error(`Release manifest has unsupported provenance: ${String(manifest.provenance)}`);
}
if (manifest.provenance === "release") {
  if (manifest.source.dirty || manifest.source.ref !== `v${manifest.version}`) {
    throw new Error("Tagged release provenance must be clean and match the archive version");
  }
  if (typeof manifest.workflowUrl !== "string" || manifest.workflowUrl.length === 0) {
    throw new Error("Tagged release provenance requires a workflow URL");
  }
}
if (manifest.workflowUrl !== null) {
  const workflowUrl = new URL(manifest.workflowUrl);
  if (workflowUrl.protocol !== "https:") {
    throw new Error("Release manifest workflow URL must use HTTPS");
  }
}

const engineSource = await readFile(
  path.join(root, "crates", "engine", "src", "racing.rs"),
  "utf8",
);
const generatorMatch = engineSource.match(/pub const GENERATOR_VERSION: &str = "([^"]+)";/);
if (!generatorMatch || manifest.generatorVersion !== generatorMatch[1]) {
  throw new Error("Release manifest generator version does not match shipped source");
}
const toolchainSource = await readFile(path.join(root, "rust-toolchain.toml"), "utf8");
const rustMatch = toolchainSource.match(/channel\s*=\s*"([^"]+)"/);
if (!rustMatch || manifest.toolchain?.rust !== rustMatch[1]) {
  throw new Error("Release manifest Rust version does not match shipped toolchain");
}
if (typeof manifest.toolchain?.godot !== "string" || manifest.toolchain.godot.length === 0) {
  throw new Error("Release manifest Godot version is missing");
}

const expectedLibraries = new Map([
  ["addons/gamestruments/bin/libgamestruments_godot.so", "x86_64-unknown-linux-gnu"],
  ["addons/gamestruments/bin/gamestruments_godot.dll", "x86_64-pc-windows-msvc"],
  ["addons/gamestruments/bin/libgamestruments_godot.dylib", "aarch64-apple-darwin+x86_64-apple-darwin"],
  ["addons/gamestruments/bin/gamestruments_godot.wasm", "wasm32-unknown-emscripten"],
]);
if (!Array.isArray(manifest.nativeLibraries) || manifest.nativeLibraries.length !== expectedLibraries.size) {
  throw new Error(`Release manifest must list exactly ${expectedLibraries.size} native libraries`);
}
for (const library of manifest.nativeLibraries) {
  if (expectedLibraries.get(library.path) !== library.target) {
    throw new Error(`Release manifest has an unexpected native library: ${String(library.path)}`);
  }
  const bytes = await readFile(path.join(root, library.path));
  const digest = createHash("sha256").update(bytes).digest("hex");
  if (digest !== library.sha256) {
    throw new Error(`Release manifest digest mismatch for ${library.path}`);
  }
  expectedLibraries.delete(library.path);
}
if (expectedLibraries.size !== 0) {
  throw new Error(`Release manifest is missing: ${[...expectedLibraries.keys()].join(", ")}`);
}

console.log("GAMESTRUMENTS_RELEASE_MANIFEST_VERIFY_PASS");
