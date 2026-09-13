import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { afterAll, describe, expect, it } from "vitest";

const SCRIPT = fileURLToPath(new URL("../tools/package-public-docs.mjs", import.meta.url));
const REPO_ROOT = fileURLToPath(new URL("..", import.meta.url));
const ZIP_NAME = "gamestruments-docs-and-example.zip";

const NOTICE =
  "> This is documentation for the paid kit. This free download contains the docs and example.gd, not the addon.";

// Exact allowlist the bundle must contain — nothing more, nothing less.
const EXPECTED_FILES = [
  "README.md",
  "LICENSE.md",
  "example.gd",
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
].sort();

const FORBIDDEN_EXTENSIONS = [
  ".tscn",
  ".so",
  ".dll",
  ".dylib",
  ".gd.uid",
  ".godot",
  ".js",
  ".mjs",
  ".ts",
  ".tsx",
];

interface RunResult {
  status: number | null;
  output: string;
}

function runPackage(args: string[], env: NodeJS.ProcessEnv = {}): RunResult {
  const result = spawnSync(process.execPath, [SCRIPT, ...args], {
    encoding: "utf8",
    env: { ...process.env, ...env },
  });
  return {
    status: result.status,
    output: `${result.stdout ?? ""}\n${result.stderr ?? ""}`,
  };
}

function extractZip(zipPath: string, into: string): void {
  execFileSync("unzip", ["-q", "-o", zipPath, "-d", into], { stdio: ["ignore", "ignore", "pipe"] });
}

function walkFiles(root: string, dir: string = root): string[] {
  const result: string[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      result.push(...walkFiles(root, full));
    } else {
      result.push(path.relative(root, full).split(path.sep).join("/"));
    }
  }
  return result.sort();
}

function firstGdscriptFence(markdown: string): string {
  const match = /```gdscript\r?\n([\s\S]*?)```/.exec(markdown);
  if (match === null || match[1] === undefined) {
    throw new Error("no gdscript fence found");
  }
  return match[1];
}

// Relative markdown links (excluding URLs and bare anchors) present in a file.
function relativeMarkdownLinks(content: string): string[] {
  const links: string[] = [];
  const linkPattern = /!?\[[^\]]*\]\(([^)\s]+)\)/g;
  let match: RegExpExecArray | null;
  while ((match = linkPattern.exec(content)) !== null) {
    const target = match[1] ?? "";
    if (
      target.startsWith("http://") ||
      target.startsWith("https://") ||
      target.startsWith("mailto:") ||
      target.startsWith("#")
    ) {
      continue;
    }
    const filePart = target.split("#")[0] ?? "";
    if (filePart !== "") {
      links.push(filePart);
    }
  }
  return links;
}

function sha256(filePath: string): string {
  return createHash("sha256").update(readFileSync(filePath)).digest("hex");
}

interface FixtureOptions {
  quickstartFences?: number;
  firstFenceNotExtendsNode?: boolean;
  brokenLink?: boolean;
  license?: string;
}

async function writeFixture(root: string, options: FixtureOptions = {}): Promise<void> {
  const fenceCount = options.quickstartFences ?? 2;
  const firstBody = options.firstFenceNotExtendsNode ? "var x := 0" : "extends Node\nvar x := 0";
  const fences = Array.from({ length: fenceCount }, (_, index) => {
    const body = index === 0 ? firstBody : "extends Node\nvar y := 0";
    return ["```gdscript", body, "```"].join("\n");
  });

  const write = async (relative: string, content: string) => {
    const abs = path.join(root, relative);
    await mkdir(path.dirname(abs), { recursive: true });
    await writeFile(abs, content);
  };

  await write("kit/docs/quickstart.md", fences.join("\n\n"));
  await write("kit/README.md", "# Kit\n");
  await write("kit/docs/README.md", "# Docs\n");
  await write("kit/docs/api.md", "# API\n");
  await write("kit/docs/limitations.md", "# Limitations\n");
  await write("kit/docs/troubleshooting.md", "# Troubleshooting\n");
  await write("kit/examples/README.md", "# Examples\n");
  await write("kit/examples/01-playback/README.md", options.brokenLink ? "# 01\n\n[missing](../missing.md)\n" : "# 01\n");
  await write("kit/examples/02-game-signals/README.md", "# 02\n");
  await write("kit/examples/03-song-form/README.md", "# 03\n");
  await write(
    "crates/LICENSE.md",
    options.license ?? "MIT License\n\nPermission is hereby granted, free of charge, to any person obtaining a copy of this software.\n",
  );
}

describe.skipIf(process.platform === "win32")("public docs + example package", () => {
  let realZipPath: string | undefined;
  let realExtractDir: string | undefined;
  let realOutDir: string | undefined;

  afterAll(async () => {
    if (realOutDir) {
      await rm(realOutDir, { recursive: true, force: true });
    }
  });

  // Builds the real canonical bundle once and shares it across content tests.
  async function realBundle(): Promise<{ zipPath: string; extractDir: string }> {
    if (realZipPath && realExtractDir) {
      return { zipPath: realZipPath, extractDir: realExtractDir };
    }
    realOutDir = await mkdtemp(path.join(os.tmpdir(), "gamestruments-public-docs-test-"));
    const result = runPackage(["--out-dir", realOutDir]);
    expect(result.status).toBe(0);
    expect(result.output).toContain("local draft (not a release candidate)");
    realZipPath = path.join(realOutDir, ZIP_NAME);
    realExtractDir = path.join(realOutDir, "extracted");
    extractZip(realZipPath, realExtractDir);
    return { zipPath: realZipPath, extractDir: realExtractDir };
  }

  it("builds a zip with every public doc, the license, the README, and exactly one example", async () => {
    const { extractDir } = await realBundle();
    expect(walkFiles(extractDir)).toEqual(EXPECTED_FILES);

    const gdFiles = EXPECTED_FILES.filter((file) => file.endsWith(".gd"));
    expect(gdFiles).toEqual(["example.gd"]);

    for (const file of walkFiles(extractDir)) {
      for (const ext of FORBIDDEN_EXTENSIONS) {
        expect(file).not.toContain(ext);
      }
      expect(file).not.toContain("addons/");
      expect(file).not.toContain("crates/");
      expect(file).not.toContain("project.godot");
    }
  });

  it("extracts example.gd byte-identical to the quickstart's first gdscript fence", async () => {
    const { extractDir } = await realBundle();
    const quickstart = await readFile(path.join(REPO_ROOT, "kit", "docs", "quickstart.md"), "utf8");
    const example = await readFile(path.join(extractDir, "example.gd"), "utf8");
    expect(example).toBe(firstGdscriptFence(quickstart));
    expect(example.startsWith("extends Node")).toBe(true);
  });

  it("rewrites links to omitted .gd files as inline code while keeping doc links and anchors", async () => {
    const { extractDir } = await realBundle();
    const playback = await readFile(path.join(extractDir, "kit", "examples", "01-playback", "README.md"), "utf8");
    expect(playback).toContain("Read `playback.gd`");
    expect(playback).toContain("`playback.gd` (included in the paid kit)");
    expect(playback).not.toContain("](playback.gd)");
    expect(playback).toContain("](../../docs/quickstart.md#suspense-recipe-complete-script)");
    expect(playback).toContain("](../README.md)");

    const signals = await readFile(path.join(extractDir, "kit", "examples", "02-game-signals", "README.md"), "utf8");
    expect(signals).toContain("`game_signals.gd`");
    expect(signals).not.toContain("](game_signals.gd)");

    const songForm = await readFile(path.join(extractDir, "kit", "examples", "03-song-form", "README.md"), "utf8");
    expect(songForm).toContain("`song_form.gd`");
    expect(songForm).not.toContain("](song_form.gd)");
  });

  it("prepends the paid-kit notice to the copied READMEs without touching canonical files", async () => {
    const { extractDir } = await realBundle();
    const bundledKit = await readFile(path.join(extractDir, "kit", "README.md"), "utf8");
    const bundledDocs = await readFile(path.join(extractDir, "kit", "docs", "README.md"), "utf8");
    expect(bundledKit.split("\n")[0]).toBe(NOTICE);
    expect(bundledDocs.split("\n")[0]).toBe(NOTICE);

    const canonicalKit = await readFile(path.join(REPO_ROOT, "kit", "README.md"), "utf8");
    const canonicalDocs = await readFile(path.join(REPO_ROOT, "kit", "docs", "README.md"), "utf8");
    expect(canonicalKit).not.toContain("not the addon");
    expect(canonicalDocs).not.toContain("not the addon");
  });

  it("leaves no broken relative markdown links in the bundle", async () => {
    const { extractDir } = await realBundle();
    const files = walkFiles(extractDir).filter((file) => file.endsWith(".md"));
    for (const file of files) {
      const content = await readFile(path.join(extractDir, file), "utf8");
      for (const link of relativeMarkdownLinks(content)) {
        const resolved = path.normalize(path.join(path.dirname(file), link));
        expect(existsSync(path.join(extractDir, resolved)), `${file} links to missing ${resolved}`).toBe(true);
      }
    }
  });

  it("rebuilds byte-for-byte identical across host time zones", async () => {
    const firstDir = await mkdtemp(path.join(os.tmpdir(), "gamestruments-public-docs-rebuild-a-"));
    const secondDir = await mkdtemp(path.join(os.tmpdir(), "gamestruments-public-docs-rebuild-b-"));
    try {
      const first = runPackage(["--out-dir", firstDir], { TZ: "UTC" });
      expect(first.status).toBe(0);
      const second = runPackage(["--out-dir", secondDir], { TZ: "Pacific/Auckland" });
      expect(second.status).toBe(0);

      const firstZip = path.join(firstDir, ZIP_NAME);
      const secondZip = path.join(secondDir, ZIP_NAME);
      expect(sha256(firstZip)).toBe(sha256(secondZip));
    } finally {
      await rm(firstDir, { recursive: true, force: true });
      await rm(secondDir, { recursive: true, force: true });
    }
  });

  it("rejects a quickstart with the wrong fence count", async () => {
    for (const count of [1, 3]) {
      const root = await mkdtemp(path.join(os.tmpdir(), "gamestruments-public-docs-fence-"));
      const outDir = path.join(root, "out");
      try {
        await writeFixture(root, { quickstartFences: count });
        const { status, output } = runPackage(["--project-root", root, "--out-dir", outDir]);
        expect(status).not.toBe(0);
        expect(output).toContain(
          `kit/docs/quickstart.md must contain exactly two gdscript fences (found ${count})`,
        );
      } finally {
        await rm(root, { recursive: true, force: true });
      }
    }
  });

  it("rejects a quickstart whose first fence does not begin with extends Node", async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "gamestruments-public-docs-extends-"));
    const outDir = path.join(root, "out");
    try {
      await writeFixture(root, { firstFenceNotExtendsNode: true });
      const { status, output } = runPackage(["--project-root", root, "--out-dir", outDir]);
      expect(status).not.toBe(0);
      expect(output).toContain('first gdscript fence must begin with "extends Node"');
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it("fails loudly on a relative docs link that is not a known omitted code/scene file", async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "gamestruments-public-docs-link-"));
    const outDir = path.join(root, "out");
    try {
      await writeFixture(root, { brokenLink: true });
      const { status, output } = runPackage(["--project-root", root, "--out-dir", outDir]);
      expect(status).not.toBe(0);
      expect(output).toContain("which is not in the bundle");
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it("refuses to copy a license source that is not MIT", async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "gamestruments-public-docs-license-"));
    const outDir = path.join(root, "out");
    try {
      await writeFixture(root, { license: "GPL-3.0-only\n" });
      const { status, output } = runPackage(["--project-root", root, "--out-dir", outDir]);
      expect(status).not.toBe(0);
      expect(output).toContain("is not the expected MIT license text");
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it("does not silently erase a typo in an unknown script link", async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "gamestruments-public-docs-script-link-"));
    try {
      await writeFixture(root);
      await writeFile(path.join(root, "kit", "examples", "01-playback", "README.md"), "[script](playbck.gd)\n");
      const result = runPackage(["--project-root", root, "--out-dir", path.join(root, "out")]);
      expect(result.status).not.toBe(0);
      expect(result.output).toContain('"playbck.gd"');
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
});
