#!/usr/bin/env node
// tools/package-kit.mjs
// Assemble the distributable Gamestruments Godot kit into
// dist-kit/gamestruments-<version>/ and zip it to dist-kit/gamestruments-<version>.zip.
//
// Contents: the Godot addon (addons/gamestruments/ with its .gdextension and,
// when a build exists, the built native library), plus kit/docs/, kit/examples/,
// and kit/README.md. Uses only Node's stdlib and the system `zip`.
//
// Usage:
//   node tools/package-kit.mjs

import { execFileSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
} from "node:fs";
import path from "node:path";
import process from "node:process";

const repoRoot = path.resolve(import.meta.dirname, "..");
const VERSION = JSON.parse(
  readFileSync(path.join(repoRoot, "package.json"), "utf8"),
).version;

const distKitDir = path.join(repoRoot, "dist-kit");
const outDir = path.join(distKitDir, `gamestruments-${VERSION}`);
const zipPath = path.join(distKitDir, `gamestruments-${VERSION}.zip`);

const addonSrc = path.join(repoRoot, "addons", "gamestruments");
const gdextensionSrc = path.join(
  repoRoot,
  "crates",
  "godot",
  "gamestruments.gdextension",
);
const nativeLibSrc = path.join(
  repoRoot,
  "target",
  "release",
  "libgamestruments_godot.so",
);

function fail(message) {
  console.error(message);
  process.exit(1);
}

// Copy a single file into outDir/<rel>, creating parent directories as needed.
function stageFile(src, rel) {
  const dest = path.join(outDir, rel);
  mkdirSync(path.dirname(dest), { recursive: true });
  cpSync(src, dest);
}

// Copy a directory tree into outDir/<rel>, skipping Godot editor caches
// (.godot/), any embedded addons/, and *.uid cache files.
function stageTree(srcDir, rel) {
  cpSync(srcDir, path.join(outDir, rel), {
    recursive: true,
    filter: (entry) => {
      const base = path.basename(entry);
      if (base === ".godot" || base === "addons") return false;
      if (base.endsWith(".uid")) return false;
      return true;
    },
  });
}

// Recursively collect every file under a directory as POSIX relative paths.
function collectFiles(dir, prefix = "") {
  const files = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) {
      files.push(...collectFiles(path.join(dir, entry.name), rel));
    } else if (entry.isFile()) {
      files.push(rel);
    }
  }
  return files;
}

// 1. Assemble the staging directory.
rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

if (!existsSync(addonSrc)) {
  fail(`addon source missing: ${addonSrc}`);
}
if (!existsSync(gdextensionSrc)) {
  fail(`gdextension source missing: ${gdextensionSrc}`);
}

stageTree(addonSrc, path.posix.join("addons", "gamestruments"));
stageFile(gdextensionSrc, path.posix.join("addons", "gamestruments", "gamestruments.gdextension"));

const libRel = path.posix.join("addons", "gamestruments", "bin", "libgamestruments_godot.so");
if (existsSync(nativeLibSrc)) {
  stageFile(nativeLibSrc, libRel);
} else {
  console.log(
    `Note: no built native library at target/release/libgamestruments_godot.so — shipping the addon without ${libRel}`,
  );
}

stageTree(path.join(repoRoot, "kit", "docs"), path.posix.join("kit", "docs"));
stageTree(path.join(repoRoot, "kit", "examples"), path.posix.join("kit", "examples"));
stageFile(path.join(repoRoot, "kit", "README.md"), path.posix.join("kit", "README.md"));

// 2. List what was packaged.
const files = collectFiles(outDir).sort();
console.log(`Packaged ${files.length} files into ${outDir}:`);
for (const file of files) {
  console.log(`  ${file}`);
}

// 3. Zip deterministically (sorted entries, -X for no extra fields).
rmSync(zipPath, { force: true });
const list = files.join("\n") + (files.length > 0 ? "\n" : "");
try {
  execFileSync("zip", ["-X", "-9", "-@", zipPath], {
    cwd: outDir,
    input: list,
    stdio: ["pipe", "ignore", "pipe"],
  });
} catch (zipError) {
  const detail =
    zipError && typeof zipError.stderr === "object"
      ? zipError.stderr.toString("utf8").trim()
      : "";
  fail(`zip failed: ${detail || zipError.message}`);
}

const bytes = statSync(zipPath).size;
console.log(`Zipped: ${zipPath}`);
console.log(`Zip size: ${bytes} bytes`);
