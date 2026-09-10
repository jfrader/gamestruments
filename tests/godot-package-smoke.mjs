import { spawnSync } from "node:child_process";
import { cp, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  args.set(process.argv[index], process.argv[index + 1]);
}

const godotArgument = args.get("--godot");
const libraryArgument = args.get("--library");
if (!godotArgument || !libraryArgument) {
  throw new Error("Usage: node tests/godot-package-smoke.mjs --godot <binary> --library <extension>");
}
const godot = path.resolve(godotArgument);
const library = path.resolve(libraryArgument);

const projectRoot = path.resolve(args.get("--project-root") ?? path.resolve(import.meta.dirname, ".."));
const tempRoot = await mkdtemp(path.join(os.tmpdir(), "gamestruments-godot-smoke-"));
const demoRoot = path.join(tempRoot, "demo");
const addonRoot = path.join(demoRoot, "addons", "gamestruments");

function runGodot(commandArgs, label) {
  const result = spawnSync(godot, commandArgs, {
    cwd: demoRoot,
    encoding: "utf8",
    timeout: 60_000,
  });
  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
  process.stdout.write(output);
  const forbidden = [
    /ERROR:/i,
    /FATAL:/i,
    /crashed with signal/i,
    /ObjectDB instance(?:s| was)? leaked/i,
    /Leaked instance:/i,
    /resources still in use at exit/i,
  ];
  if (result.status !== 0 || result.error || forbidden.some((pattern) => pattern.test(output))) {
    throw new Error(`${label} failed with status ${String(result.status)}${result.error ? `: ${result.error.message}` : ""}`);
  }
  return output;
}

try {
  await cp(path.join(projectRoot, "kit", "demo"), demoRoot, { recursive: true });
  await mkdir(path.join(addonRoot, "bin"), { recursive: true });
  await cp(
    path.join(projectRoot, "crates", "godot", "gamestruments.gdextension"),
    path.join(addonRoot, "gamestruments.gdextension"),
  );
  await cp(library, path.join(addonRoot, "bin", path.basename(library)));
  // Headless --script does not perform the editor's extension discovery pass.
  await mkdir(path.join(demoRoot, ".godot"), { recursive: true });
  await writeFile(
    path.join(demoRoot, ".godot", "extension_list.cfg"),
    "res://addons/gamestruments/gamestruments.gdextension\n",
  );
  const runtimeOutput = runGodot(
    ["--headless", "--verbose", "--path", demoRoot, "--script", "res://tools/runtime_smoke.gd"],
    "runtime smoke",
  );
  if (!runtimeOutput.includes("GAMESTRUMENTS_RUNTIME_SMOKE_PASS")) {
    throw new Error("runtime smoke did not print its success marker");
  }
  const screenshotDirectory = args.get("--screenshots");
  if (screenshotDirectory) {
    await mkdir(path.resolve(screenshotDirectory), { recursive: true });
  }
  const gameplayOutput = runGodot(
    [
      ...(screenshotDirectory ? [] : ["--headless"]),
      "--path", demoRoot, "--script", "res://tools/gameplay_smoke.gd",
      ...(screenshotDirectory ? ["--", "--screenshots", path.resolve(screenshotDirectory)] : []),
    ],
    "gameplay smoke",
  );
  if (!gameplayOutput.includes("GAMESTRUMENTS_GAMEPLAY_SMOKE_PASS")) {
    throw new Error("gameplay smoke did not print its success marker");
  }
  await rm(addonRoot, { recursive: true, force: true });
  await writeFile(path.join(demoRoot, ".godot", "extension_list.cfg"), "");
  const fallbackOutput = runGodot(
    ["--headless", "--path", demoRoot, "--script", "res://tools/gameplay_smoke.gd", "--", "--without-music"],
    "missing-extension gameplay smoke",
  );
  if (!fallbackOutput.includes("GAMESTRUMENTS_GAMEPLAY_SMOKE_PASS")) {
    throw new Error("missing-extension gameplay smoke did not print its success marker");
  }
} finally {
  await rm(tempRoot, { recursive: true, force: true });
}
