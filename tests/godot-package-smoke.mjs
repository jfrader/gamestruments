import { spawnSync } from "node:child_process";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { extractGdscriptFences } from "../tools/gdscript-snippets.mjs";

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
const demoRoot = path.join(tempRoot, "examples");
const addonRoot = path.join(demoRoot, "addons", "gamestruments");

function runGodot(commandArgs, label, cwd = demoRoot) {
  const result = spawnSync(godot, commandArgs, {
    cwd,
    encoding: "utf8",
    timeout: 180_000,
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
    /RID allocations.*leaked/i,
    /^WARNING:(?! (?:Your graphics card drivers seem not to support V-Sync\.|Could not set V-Sync mode, as changing V-Sync mode is not supported by the graphics driver\.)\r?$)/im,
  ];
  if (result.status !== 0 || result.error || forbidden.some((pattern) => pattern.test(output))) {
    throw new Error(`${label} failed with status ${String(result.status)}${result.error ? `: ${result.error.message}` : ""}`);
  }
  return output;
}

async function stageDocsSnippets() {
  // The shipped kit docs are the contract: kit/README.md has exactly one
  // standalone Racing _ready, kit/docs/quickstart.md has exactly two standalone
  // scripts (Racing, then Suspense). Prefer the stable kit/ paths so the check
  // also works against an extracted archive.
  const readme = await readFile(path.join(projectRoot, "kit", "README.md"), "utf8");
  const readmeFences = extractGdscriptFences(readme);
  if (readmeFences.length !== 1) {
    throw new Error(`kit/README.md must contain exactly one gdscript fence (found ${readmeFences.length})`);
  }
  const quickstart = await readFile(path.join(projectRoot, "kit", "docs", "quickstart.md"), "utf8");
  const quickstartFences = extractGdscriptFences(quickstart);
  if (quickstartFences.length !== 2) {
    throw new Error(`kit/docs/quickstart.md must contain exactly two gdscript fences (found ${quickstartFences.length})`);
  }
  const limitations = await readFile(path.join(projectRoot, "kit", "docs", "limitations.md"), "utf8");
  const limitationsFences = extractGdscriptFences(limitations);
  if (limitationsFences.length !== 0) {
    throw new Error(`kit/docs/limitations.md must not contain gdscript fences (found ${limitationsFences.length})`);
  }

  // A fresh, minimal project with only the addon and the extracted snippets.
  const docsRoot = path.join(tempRoot, "docs");
  const docsAddonRoot = path.join(docsRoot, "addons", "gamestruments");
  await mkdir(path.join(docsAddonRoot, "bin"), { recursive: true });
  await cp(
    path.join(projectRoot, "crates", "godot", "gamestruments.gdextension"),
    path.join(docsAddonRoot, "gamestruments.gdextension"),
  );
  await cp(library, path.join(docsAddonRoot, "bin", path.basename(library)));
  await writeFile(
    path.join(docsRoot, "project.godot"),
    'config_version=5\n\n[application]\n\nconfig/name="Gamestruments Docs Smoke"\n',
  );
  await mkdir(path.join(docsRoot, ".godot"), { recursive: true });
  await writeFile(
    path.join(docsRoot, ".godot", "extension_list.cfg"),
    "res://addons/gamestruments/gamestruments.gdextension\n",
  );
  await mkdir(path.join(docsRoot, "snippets"), { recursive: true });
  await writeFile(path.join(docsRoot, "snippets", "readme_racing.gd"), readmeFences[0]);
  await writeFile(path.join(docsRoot, "snippets", "quickstart_racing.gd"), quickstartFences[0]);
  await writeFile(path.join(docsRoot, "snippets", "quickstart_suspense.gd"), quickstartFences[1]);
  await cp(
    path.join(import.meta.dirname, "godot-docs-smoke.gd"),
    path.join(docsRoot, "docs_smoke.gd"),
  );

  const docsOutput = runGodot(
    ["--headless", "--verbose", "--path", docsRoot, "--script", "res://docs_smoke.gd"],
    "docs snippets smoke",
    docsRoot,
  );
  if (!docsOutput.includes("DOCS_SMOKE_PASS")) {
    throw new Error("docs snippets smoke did not print its success marker");
  }
}

try {
  await cp(path.join(projectRoot, "kit", "examples"), demoRoot, {
    recursive: true,
    filter: (source) => path.basename(source) !== ".godot",
  });
  await mkdir(path.join(addonRoot, "bin"), { recursive: true });
  await cp(
    path.join(projectRoot, "crates", "godot", "gamestruments.gdextension"),
    path.join(addonRoot, "gamestruments.gdextension"),
  );
  await cp(library, path.join(addonRoot, "bin", path.basename(library)));

  const screenshotDirectory = args.get("--screenshots");
  if (screenshotDirectory) {
    await mkdir(path.resolve(screenshotDirectory), { recursive: true });
    runGodot(["--path", demoRoot, "--editor", "--import"], "fresh rendered editor import");
  } else {
    // --script does not discover extensions; rendered QA covers editor import.
    await mkdir(path.join(demoRoot, ".godot"), { recursive: true });
    await writeFile(
      path.join(demoRoot, ".godot", "extension_list.cfg"),
      "res://addons/gamestruments/gamestruments.gdextension\n",
    );
  }

  const runtimeOutput = runGodot(
    [
      ...(screenshotDirectory ? [] : ["--headless"]),
      "--verbose", "--path", demoRoot, "--script", "res://tools/runtime_smoke.gd",
      ...(screenshotDirectory ? ["--", "--screenshots", path.resolve(screenshotDirectory)] : []),
    ],
    "examples runtime smoke",
  );
  if (!runtimeOutput.includes("GAMESTRUMENTS_EXAMPLES_SMOKE_PASS")) {
    throw new Error("examples runtime smoke did not print its success marker");
  }

  await rm(addonRoot, { recursive: true, force: true });
  await writeFile(path.join(demoRoot, ".godot", "extension_list.cfg"), "");
  const negOutput = runGodot(
    [
      ...(screenshotDirectory ? [] : ["--headless"]),
      "--path", demoRoot, "--script", "res://tools/runtime_smoke.gd", "--",
      ...(screenshotDirectory ? ["--screenshots", path.resolve(screenshotDirectory)] : []),
      "--missing-extension",
    ],
    "missing-extension examples smoke",
  );
  if (!negOutput.includes("GAMESTRUMENTS_EXAMPLES_SMOKE_PASS")) {
    throw new Error("missing-extension examples smoke did not print success (error path must still pass)");
  }

  await stageDocsSnippets();
} finally {
  await rm(tempRoot, { recursive: true, force: true });
}
