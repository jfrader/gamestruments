#!/usr/bin/env node
// tools/package-public-docs.mjs
// Build the free public docs + example ZIP for the itch.io per-file demo
// download. This is a LOCAL DRAFT, not a release candidate: it contains the
// complete paid-kit documentation plus one public example script for
// inspection only — no addon, native binaries, runnable project, or engine.
//
// Usage:
//   node tools/package-public-docs.mjs [--out-dir DIR] [--project-root DIR]
//
// - Defaults to dist/gamestruments-docs-and-example.zip under the repo root.
// - --project-root points at a repository checkout (canonical docs, MIT
//   license, and the quickstart fence source). Defaults to this repo.
// - Uses an explicit allowlist: only the canonical docs, three example
//   walkthroughs, a generated README, the MIT license, and example.gd are
//   included. Anything else is never copied.
// - Deterministic zip: sorted entries + fixed mtime/permissions + zip -X.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { parseArgs } from "node:util";
import { extractGdscriptFences } from "./gdscript-snippets.mjs";

const ZIP_NAME = "gamestruments-docs-and-example.zip";
// Fixed timestamp (2024-01-01T00:00:00Z) so the zip is byte-for-byte
// reproducible across builds, matching the buyer-archive conventions.
const FIXED_MTIME_MS = Date.UTC(2024, 0, 1, 0, 0, 0);

// Prepend this to the paid-kit docs copies so a free downloader is never
// confused about what they received. Canonical files are not modified.
const NOTICE =
  "> This is documentation for the paid kit. This free download contains the docs and example.gd, not the addon.\n";

// Explicit allowlist of canonical documentation. Guides count as docs, not
// scripts; no code, scene, engine, addon, or runtime files are allowed here.
const DOC_FILES = [
  "kit/README.md",
  "kit/docs/README.md",
  "kit/docs/quickstart.md",
  "kit/docs/api.md",
  "kit/docs/limitations.md",
  "kit/docs/troubleshooting.md",
  "kit/examples/README.md",
  "kit/examples/01-playback/README.md",
  "kit/examples/02-game-signals/README.md",
  "kit/examples/03-song-form/README.md",
];

const LICENSE_SOURCE = "crates/LICENSE.md";
const PAID_EXAMPLE_FILES = new Set([
  "kit/examples/01-playback/playback.gd",
  "kit/examples/01-playback/playback.tscn",
  "kit/examples/02-game-signals/game_signals.gd",
  "kit/examples/02-game-signals/game_signals.tscn",
  "kit/examples/03-song-form/song_form.gd",
  "kit/examples/03-song-form/song_form.tscn",
]);

// Short, generated root README (<= 20 lines). No new docs tree is introduced.
const PUBLIC_README = [
  "# Gamestruments — Free Docs & Example",
  "",
  "This free download contains the complete paid-kit documentation and one",
  "public example script (`example.gd`), for inspection before purchase. It",
  "is not the addon, and the example will not run without the paid kit's",
  "`GamestrumentsPlayer`.",
  "",
  "## Contents",
  "",
  "- `kit/docs/` — quickstart, API, limitations, and troubleshooting.",
  "- `example.gd` — the complete Racing script from the quickstart.",
  "- `LICENSE.md` — MIT license terms.",
  "",
  "## Get started",
  "",
  "- Docs & guide: [`kit/docs/README.md`](kit/docs/README.md) and",
  "  [quickstart](kit/docs/quickstart.md).",
  "- Public example: [`example.gd`](example.gd).",
  "- Listen in your browser: <https://gamestruments.gurisitos.games>.",
  "- Buy the paid kit: <https://gurisitosgames.itch.io/gamestruments-godot>.",
].join("\n");

function splitFragment(target) {
  const hash = target.indexOf("#");
  if (hash === -1) {
    return { file: target, fragment: "" };
  }
  return { file: target.slice(0, hash), fragment: target.slice(hash + 1) };
}

function resolveBundlePath(fromFile, linkFile) {
  const dir = path.posix.dirname(fromFile);
  return path.posix.normalize(path.posix.join(dir === "." ? "" : dir, linkFile));
}

const INLINE_LINK_RE = /!?\[[^\]]*\]\(([^)\s]+)\)/g;

// Rewrites links to omitted .gd/.tscn files as plain inline code, and fails
// loudly on any other relative link that does not resolve to a bundled file.
function rewriteLinks(markdown, fromFile, includedFiles) {
  const errors = [];
  const output = markdown.replace(INLINE_LINK_RE, (whole, target) => {
    const { file } = splitFragment(target);
    if (
      file === "" ||
      file.startsWith("http://") ||
      file.startsWith("https://") ||
      file.startsWith("mailto:")
    ) {
      return whole; // anchor-only or external URL — leave untouched.
    }
    const resolved = resolveBundlePath(fromFile, file);
    if (includedFiles.has(resolved)) {
      return whole; // resolves to a bundled doc — keep the link.
    }
    if (PAID_EXAMPLE_FILES.has(resolved)) {
      return "`" + file + "` (included in the paid kit)";
    }
    errors.push(
      `${fromFile}: relative link "${target}" resolves to "${resolved}" which is not in the bundle`,
    );
    return whole;
  });
  return { output, errors };
}

function build({ repoRoot, outDir }) {
  const quickstartPath = path.join(repoRoot, "kit", "docs", "quickstart.md");
  const quickstart = readFileSync(quickstartPath, "utf8");
  const fences = extractGdscriptFences(quickstart);
  if (fences.length !== 2) {
    throw new Error(
      `kit/docs/quickstart.md must contain exactly two gdscript fences (found ${fences.length})`,
    );
  }
  if (!fences[0].startsWith("extends Node")) {
    throw new Error(
      'kit/docs/quickstart.md first gdscript fence must begin with "extends Node"',
    );
  }
  const exampleGd = fences[0];

  const includedFiles = new Set(DOC_FILES);
  includedFiles.add("README.md");
  includedFiles.add("LICENSE.md");
  includedFiles.add("example.gd");

  const files = new Map();
  const errors = [];
  for (const doc of DOC_FILES) {
    let content = readFileSync(path.join(repoRoot, doc), "utf8");
    if (doc === "kit/README.md" || doc === "kit/docs/README.md") {
      content = NOTICE + "\n" + content;
    }
    const { output, errors: linkErrors } = rewriteLinks(content, doc, includedFiles);
    errors.push(...linkErrors);
    files.set(doc, output);
  }
  files.set("example.gd", exampleGd);
  files.set("README.md", PUBLIC_README);

  const license = readFileSync(path.join(repoRoot, LICENSE_SOURCE), "utf8");
  if (
    !/MIT License/i.test(license) ||
    !/Permission is hereby granted, free of charge/.test(license)
  ) {
    throw new Error(
      `${LICENSE_SOURCE} is not the expected MIT license text; refusing to copy`,
    );
  }
  files.set("LICENSE.md", license);

  if (errors.length > 0) {
    throw new Error(errors.join("\n"));
  }

  const staging = mkdtempSync(path.join(os.tmpdir(), "gamestruments-public-docs-"));
  try {
    const sorted = [...files.keys()].sort();
    for (const rel of sorted) {
      const abs = path.join(staging, rel);
      mkdirSync(path.dirname(abs), { recursive: true });
      writeFileSync(abs, files.get(rel));
      chmodSync(abs, 0o644);
      utimesSync(abs, FIXED_MTIME_MS / 1000, FIXED_MTIME_MS / 1000);
    }

    mkdirSync(outDir, { recursive: true });
    const zipPath = path.join(outDir, ZIP_NAME);
    rmSync(zipPath, { force: true });
    const list = sorted.join("\n") + (sorted.length > 0 ? "\n" : "");
    try {
      execFileSync("zip", ["-X", "-9", "-@", zipPath], {
        cwd: staging,
        input: list,
        stdio: ["pipe", "ignore", "pipe"],
        env: { ...process.env, TZ: "UTC" },
      });
    } catch (zipError) {
      const detail =
        zipError && typeof zipError.stderr === "object"
          ? zipError.stderr.toString("utf8").trim()
          : "";
      throw new Error(`zip failed: ${detail || zipError.message}`);
    }

    const bytes = statSync(zipPath).size;
    const sha = createHash("sha256").update(readFileSync(zipPath)).digest("hex");
    console.log(`Public docs + example zip: ${zipPath}`);
    console.log(`Size: ${bytes} bytes`);
    console.log(`SHA-256: ${sha}`);
    console.log(
      "Status: local draft (not a release candidate) — docs plus one public example for inspection; no addon, binaries, or runnable project.",
    );
    return { zipPath, bytes, sha };
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

const { values } = parseArgs({
  options: {
    "out-dir": { type: "string" },
    "project-root": { type: "string" },
  },
  allowPositionals: false,
  strict: true,
});

const repoRoot = values["project-root"]
  ? path.resolve(values["project-root"])
  : path.resolve(import.meta.dirname, "..");
const outDir = values["out-dir"]
  ? path.resolve(values["out-dir"])
  : path.join(repoRoot, "dist");

try {
  build({ repoRoot, outDir });
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
