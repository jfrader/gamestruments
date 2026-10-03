import { spawnSync } from "node:child_process";
import { randomUUID } from "node:crypto";
import { existsSync } from "node:fs";
import { access, chmod, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { afterAll, describe, expect, it } from "vitest";

const harness = fileURLToPath(new URL("./godot-package-smoke.mjs", import.meta.url));

// A tiny stand-in for the Godot binary: it only records its invocation and emits
// the markers/errors the harness is expected to accept or reject. It never
// touches an engine, browser, or extension. For the docs smoke it also verifies
// the four snippets were actually staged (so the harness wiring is exercised,
// not just a hardcoded "3").
const STUB_SOURCE = String.raw`#!/usr/bin/env node
import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const args = process.argv.slice(2);
const mode = process.env.STUB_MODE || "happy";
const logPath = process.env.STUB_LOG;
const cwd = process.cwd();

if (logPath) {
  appendFileSync(logPath, JSON.stringify({ mode, args }) + "\n");
}

// The harness must strip the fixture's developer .godot cache during the copy.
if (existsSync(path.join(cwd, ".godot", "devcache.txt"))) {
  process.stderr.write("ERROR: .godot developer cache was copied into the demo\n");
  process.exit(1);
}

const isEditor = args.includes("--editor") || args.includes("--import");
if (isEditor) {
  mkdirSync(path.join(cwd, ".godot"), { recursive: true });
  writeFileSync(
    path.join(cwd, ".godot", "extension_list.cfg"),
    "res://addons/gamestruments/gamestruments.gdextension\n",
  );
  if (mode === "editor-fail") {
    process.stderr.write("ERROR: rendered editor import failed\n");
    process.exit(1);
  }
  if (mode === "vsync-warning") {
    process.stderr.write("WARNING: Your graphics card drivers seem not to support V-Sync.\n");
  }
  if (mode === "vsync-mode-warning") {
    process.stderr.write("WARNING: Could not set V-Sync mode, as changing V-Sync mode is not supported by the graphics driver.\n");
  }
  process.exit(0);
}

const isDocs = args.includes("res://docs_smoke.gd");
if (isDocs) {
  // Prove the harness staged all three extracted snippets, not just one.
  const snippets = [
    "snippets/readme_racing.gd",
    "snippets/quickstart_racing.gd",
    "snippets/quickstart_suspense.gd",
  ];
  for (const snippet of snippets) {
    const full = path.join(cwd, snippet);
    if (!existsSync(full) || readFileSync(full, "utf8").trim().length === 0) {
      process.stderr.write("ERROR: staged snippet missing or empty " + snippet + "\n");
      process.exit(1);
    }
  }
  if (mode === "docs-missing-marker") {
    process.stdout.write("docs smoke finished without its marker\n");
    process.exit(0);
  }
  process.stdout.write("DOCS_SMOKE_PASS\n");
  if (mode === "docs-nonzero-exit") {
    process.exit(3);
  }
  if (mode === "docs-error-marker") {
    process.stderr.write("ERROR: docs script failed but printed the marker\n");
    process.exit(0);
  }
  if (mode === "docs-warning-marker") {
    process.stderr.write("WARNING: cue rejected\n");
    process.exit(0);
  }
  process.exit(0);
}

const marker = "GAMESTRUMENTS_EXAMPLES_SMOKE_PASS";
const negative = args.includes("--missing-extension");
const fail = negative && mode !== "happy";
if (!fail) {
  process.stdout.write(marker + "\n");
  process.exit(0);
}

if (mode === "missing-marker") {
  process.stdout.write("runtime smoke finished without its marker\n");
  process.exit(0);
}
process.stdout.write(marker + "\n");
if (mode === "nonzero-exit") {
  process.exit(3);
}
if (mode === "error-marker") {
  process.stderr.write("ERROR: script failed but printed the marker\n");
  process.exit(0);
}
if (mode === "objectdb-leak") {
  process.stderr.write("ObjectDB instances leaked at exit\n");
  process.exit(0);
}
if (mode === "resource-in-use") {
  process.stderr.write("WARNING: resources still in use at exit\n");
  process.exit(0);
}
if (mode === "rid-leak") {
  process.stderr.write("RID allocations of type 'CanvasItem' leaked at exit\n");
  process.exit(0);
}
process.exit(0);
`;

const FENCE = ["```gdscript", "extends Node", "var music_ready := false", "func _ready() -> void:", "\tmusic_ready = true", "```"].join("\n");

// Builds a markdown document containing exactly `count` gdscript fences.
function markdownWithFences(count: number): string {
  return Array.from({ length: count }, () => FENCE).join("\n\nsection text\n\n");
}

interface Fixture {
  root: string;
  projectRoot: string;
  stub: string;
  library: string;
  screenshots: string;
}

interface StubInvocation {
  mode: string;
  args: string[];
}

async function createFixture(
  options: { readmeFences?: number; quickstartFences?: number; limitationsFences?: number } = {},
): Promise<Fixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), "gamestruments-godot-smoke-test-"));
  const projectRoot = path.join(root, "project");
  const examples = path.join(projectRoot, "kit", "examples");
  await mkdir(path.join(examples, "tools"), { recursive: true });
  await mkdir(path.join(examples, ".godot"), { recursive: true });
  await writeFile(path.join(examples, ".godot", "devcache.txt"), "developer cache that must not be copied\n");
  await writeFile(path.join(examples, "project.godot"), '[application]\nconfig/name="Smoke Fixture"\n');
  await writeFile(path.join(examples, "tools", "runtime_smoke.gd"), "extends SceneTree\nfunc _init():\n\tquit()\n");
  await mkdir(path.join(projectRoot, "crates", "godot"), { recursive: true });
  await writeFile(
    path.join(projectRoot, "crates", "godot", "gamestruments.gdextension"),
    '[configuration]\nentry_symbol = "gdext_rust_init"\n',
  );

  // Shipped buyer docs: kit/README.md (one fence), kit/docs/quickstart.md (two)
  // and kit/docs/limitations.md (none).
  await mkdir(path.join(projectRoot, "kit"), { recursive: true });
  await mkdir(path.join(projectRoot, "kit", "docs"), { recursive: true });
  await writeFile(
    path.join(projectRoot, "kit", "README.md"),
    markdownWithFences(options.readmeFences ?? 1),
  );
  await writeFile(
    path.join(projectRoot, "kit", "docs", "quickstart.md"),
    markdownWithFences(options.quickstartFences ?? 2),
  );
  await writeFile(
    path.join(projectRoot, "kit", "docs", "limitations.md"),
    markdownWithFences(options.limitationsFences ?? 0),
  );

  const stub = path.join(root, "fake-godot.mjs");
  await writeFile(stub, STUB_SOURCE);
  await chmod(stub, 0o755);

  const library = path.join(root, "libgamestruments_godot.so");
  await writeFile(library, "fake extension library\n");

  const screenshots = path.join(root, "screenshots");
  await mkdir(screenshots, { recursive: true });

  return { root, projectRoot, stub, library, screenshots };
}

let fixturePromise: Promise<Fixture> | undefined;
function fixture(): Promise<Fixture> {
  fixturePromise ??= createFixture();
  return fixturePromise;
}

async function runHarness(
  mode: string,
  options: { screenshots?: boolean } = {},
): Promise<{ status: number | null; output: string; entries: StubInvocation[] }> {
  const { root, projectRoot, stub, library, screenshots } = await fixture();
  const logPath = path.join(root, "logs", `${mode}-${randomUUID()}.log`);
  await mkdir(path.dirname(logPath), { recursive: true });

  const args = [
    harness,
    "--project-root",
    projectRoot,
    "--godot",
    stub,
    "--library",
    library,
  ];
  if (options.screenshots) {
    args.push("--screenshots", screenshots);
  }

  const result = spawnSync(process.execPath, args, {
    encoding: "utf8",
    env: { ...process.env, STUB_MODE: mode, STUB_LOG: logPath },
  });
  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
  const raw = existsSync(logPath) ? await readFile(logPath, "utf8") : "";
  const entries = raw
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as StubInvocation);

  return { status: result.status, output, entries };
}

describe.skipIf(process.platform === "win32")("godot package smoke harness", () => {
  afterAll(async () => {
    if (fixturePromise) {
      const { root } = await fixturePromise;
      await rm(root, { recursive: true, force: true });
    }
  });

  it("passes a clean headless run, invoking positive and negative runtime smokes plus the docs smoke", async () => {
    const { projectRoot } = await fixture();
    // The fixture source really does contain a developer cache, so a passing run
    // proves the harness filtered it out of the copied demo.
    await expect(access(path.join(projectRoot, "kit", "examples", ".godot", "devcache.txt"))).resolves.toBeUndefined();
    expect(existsSync(path.join(projectRoot, "tests"))).toBe(false);

    const { status, output, entries } = await runHarness("happy");
    expect(status).toBe(0);
    expect(output).not.toMatch(/ERROR:/i);
    expect(entries.map((entry) => entry.args)).toEqual([
      expect.arrayContaining(["--script", "res://tools/runtime_smoke.gd"]),
      expect.arrayContaining(["--script", "res://tools/runtime_smoke.gd", "--missing-extension"]),
      expect.arrayContaining(["--script", "res://docs_smoke.gd"]),
    ]);
  });

  it("stages all four extracted docs snippets and checks the docs marker", async () => {
    const { status, output, entries } = await runHarness("happy");
    expect(status).toBe(0);
    expect(output).toContain("DOCS_SMOKE_PASS");
    const docsRun = entries.find((entry) => entry.args.includes("res://docs_smoke.gd"));
    expect(docsRun?.args).toEqual(expect.arrayContaining(["--headless", "--script", "res://docs_smoke.gd"]));
  });

  it("passes a rendered run, exercising editor import plus both runtime smokes and the docs smoke", async () => {
    const { status, output, entries } = await runHarness("happy", { screenshots: true });
    expect(status).toBe(0);
    expect(output).not.toMatch(/ERROR:/i);
    expect(entries).toHaveLength(4);

    const editorRun = entries.find((entry) => entry.args.includes("--editor"));
    expect(editorRun?.args).toEqual(expect.arrayContaining(["--editor", "--import"]));

    const runtimeRuns = entries.filter((entry) => entry.args.includes("--script"));
    expect(runtimeRuns).toHaveLength(3);
    expect(runtimeRuns.some((entry) => entry.args.includes("--missing-extension"))).toBe(true);
    expect(runtimeRuns.some((entry) => entry.args.includes("res://docs_smoke.gd"))).toBe(true);
  });

  const failureModes: ReadonlyArray<readonly [mode: string, message: string]> = [
    ["error-marker", "missing-extension examples smoke failed"],
    ["objectdb-leak", "missing-extension examples smoke failed"],
    ["resource-in-use", "missing-extension examples smoke failed"],
    ["rid-leak", "missing-extension examples smoke failed"],
    ["nonzero-exit", "missing-extension examples smoke failed with status 3"],
    ["missing-marker", "missing-extension examples smoke did not print success"],
  ];

  for (const [mode, message] of failureModes) {
    it(`rejects ${mode} on the missing-extension invocation`, async () => {
      const { status, output } = await runHarness(mode);
      expect(status).not.toBe(0);
      expect(output).toContain(message);
    });
  }

  const docsFailureModes: ReadonlyArray<readonly [mode: string, message: string]> = [
    ["docs-error-marker", "docs snippets smoke failed"],
    ["docs-warning-marker", "docs snippets smoke failed"],
    ["docs-nonzero-exit", "docs snippets smoke failed with status 3"],
    ["docs-missing-marker", "docs snippets smoke did not print its success marker"],
  ];

  it("allows only the documented software-renderer V-Sync warning", async () => {
    const { status, output } = await runHarness("vsync-warning", { screenshots: true });
    expect(status).toBe(0);
    expect(output).toContain("WARNING: Your graphics card drivers seem not to support V-Sync.");
  });

  it("allows the current Godot wording of the unsupported V-Sync warning", async () => {
    const { status, output } = await runHarness("vsync-mode-warning", { screenshots: true });
    expect(status).toBe(0);
    expect(output).toContain("WARNING: Could not set V-Sync mode");
  });

  for (const [mode, message] of docsFailureModes) {
    it(`rejects ${mode} on the docs smoke invocation`, async () => {
      const { status, output } = await runHarness(mode);
      expect(status).not.toBe(0);
      expect(output).toContain(message);
    });
  }

  it("rejects a malformed README with the wrong fence count", async () => {
    const malformed = await createFixture({ readmeFences: 0 });
    try {
      const { root, projectRoot, stub, library, screenshots } = malformed;
      const logPath = path.join(root, "logs", `malformed-${randomUUID()}.log`);
      await mkdir(path.dirname(logPath), { recursive: true });
      const result = spawnSync(
        process.execPath,
        [harness, "--project-root", projectRoot, "--godot", stub, "--library", library, "--screenshots", screenshots],
        { encoding: "utf8", env: { ...process.env, STUB_MODE: "happy", STUB_LOG: logPath } },
      );
      const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
      expect(result.status).not.toBe(0);
      expect(output).toContain("kit/README.md must contain exactly one gdscript fence");
    } finally {
      await rm(malformed.root, { recursive: true, force: true });
    }
  });

  it("rejects a malformed quickstart with the wrong fence count", async () => {
    const malformed = await createFixture({ quickstartFences: 1 });
    try {
      const { root, projectRoot, stub, library, screenshots } = malformed;
      const logPath = path.join(root, "logs", `malformed-${randomUUID()}.log`);
      await mkdir(path.dirname(logPath), { recursive: true });
      const result = spawnSync(
        process.execPath,
        [harness, "--project-root", projectRoot, "--godot", stub, "--library", library, "--screenshots", screenshots],
        { encoding: "utf8", env: { ...process.env, STUB_MODE: "happy", STUB_LOG: logPath } },
      );
      const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
      expect(result.status).not.toBe(0);
      expect(output).toContain("kit/docs/quickstart.md must contain exactly two gdscript fences");
    } finally {
      await rm(malformed.root, { recursive: true, force: true });
    }
  });

  it("rejects malformed limitations with the wrong fence count", async () => {
    const malformed = await createFixture({ limitationsFences: 1 });
    try {
      const { root, projectRoot, stub, library, screenshots } = malformed;
      const logPath = path.join(root, "logs", `malformed-${randomUUID()}.log`);
      await mkdir(path.dirname(logPath), { recursive: true });
      const result = spawnSync(
        process.execPath,
        [harness, "--project-root", projectRoot, "--godot", stub, "--library", library, "--screenshots", screenshots],
        { encoding: "utf8", env: { ...process.env, STUB_MODE: "happy", STUB_LOG: logPath } },
      );
      const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
      expect(result.status).not.toBe(0);
      expect(output).toContain("kit/docs/limitations.md must not contain gdscript fences");
    } finally {
      await rm(malformed.root, { recursive: true, force: true });
    }
  });

  it("rejects a failed rendered editor import", async () => {
    const { status, output } = await runHarness("editor-fail", { screenshots: true });
    expect(status).not.toBe(0);
    expect(output).toContain("fresh rendered editor import failed");
  });
});
