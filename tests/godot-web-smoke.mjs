// Web export smoke for the GDExtension's side module.
//
//   node tests/godot-web-smoke.mjs --godot <binary> --web-library <file.wasm> \
//     --template <web_dlink_nothreads_release.zip> [--out <dir>]
//
// Exports a fresh project with Extensions Support on and Thread Support off,
// serves it without cross-origin isolation headers, and runs it in headless
// Chromium. Fails on any console error, page error, SharedArrayBuffer use, or
// a missing PASS marker.
import { spawnSync } from "node:child_process";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { chromium } from "@playwright/test";

const args = new Map();
for (let index = 2; index < process.argv.length; index += 2) {
  args.set(process.argv[index], process.argv[index + 1]);
}
const godotArgument = args.get("--godot");
const libraryArgument = args.get("--web-library");
const templateArgument = args.get("--template");
if (!godotArgument || !libraryArgument || !templateArgument) {
  throw new Error(
    "Usage: node tests/godot-web-smoke.mjs --godot <binary> --web-library <file.wasm> --template <web_dlink_nothreads_release.zip> [--out <dir>]",
  );
}
const godot = path.resolve(godotArgument);
const library = path.resolve(libraryArgument);
const template = path.resolve(templateArgument);
const outDir = args.get("--out") ? path.resolve(args.get("--out")) : null;
const projectRoot = path.resolve(import.meta.dirname, "..");

function runGodot(commandArgs, label, cwd, { allowAbort = false } = {}) {
  const result = spawnSync(godot, commandArgs, { cwd, encoding: "utf8", timeout: 300_000 });
  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
  // A fresh --import with a GDExtension can abort at editor teardown after the
  // import finished (Godot 4.7.2); the second import must then exit cleanly.
  if (allowAbort && result.status !== 0) {
    return output;
  }
  // The desktop editor has no library to load for a web-only project; that is
  // expected here and the browser run below proves the web library loads.
  const unexpected = output
    .split("\n")
    .filter((line) => /ERROR:|SCRIPT ERROR/.test(line))
    .filter((line) => !/No GDExtension library found for current OS|GDExtension dynamic library not found|Error loading extension: .res:\/\/addons\/gamestruments\/gamestruments\.gdextension/.test(line));
  if (result.status !== 0 || result.error || unexpected.length > 0) {
    process.stdout.write(output);
    throw new Error(`${label} failed with status ${String(result.status)}${result.error ? `: ${result.error.message}` : ""}`);
  }
  return output;
}

function presets(exportPath) {
  return `[preset.0]

name="Web"
platform="Web"
runnable=true
export_filter="all_resources"
include_filter=""
exclude_filter=""
export_path="${exportPath}"

[preset.0.options]

custom_template/debug=""
custom_template/release="${template}"
variant/extensions_support=true
variant/thread_support=false
vram_texture_compression/for_desktop=true
vram_texture_compression/for_mobile=false
html/canvas_resize_policy=2
progressive_web_app/enabled=false
`;
}

const mime = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm" };

function serve(root) {
  const server = http.createServer(async (request, response) => {
    const url = new URL(request.url ?? "/", "http://127.0.0.1");
    const file = path.join(root, url.pathname === "/" ? "index.html" : decodeURIComponent(url.pathname));
    if (!file.startsWith(root)) {
      response.writeHead(403).end();
      return;
    }
    try {
      const body = await readFile(file);
      response.writeHead(200, { "Content-Type": mime[path.extname(file)] ?? "application/octet-stream" }).end(body);
    } catch {
      response.writeHead(404).end();
    }
  });
  return new Promise((resolve) => server.listen(0, "127.0.0.1", () => resolve(server)));
}

async function exportProject(tempRoot) {
  const project = path.join(tempRoot, "project");
  const exportDir = path.join(tempRoot, "export");
  await mkdir(path.join(project, "addons", "gamestruments", "bin"), { recursive: true });
  await mkdir(exportDir, { recursive: true });
  // Only the web entries: the editor running the export must not try to load a
  // desktop library this project does not carry.
  const gdextension = await readFile(path.join(projectRoot, "crates", "godot", "gamestruments.gdextension"), "utf8");
  await writeFile(
    path.join(project, "addons", "gamestruments", "gamestruments.gdextension"),
    gdextension.split("\n").filter((line) => !/^(linux|windows|macos)\./.test(line)).join("\n"),
  );
  await cp(library, path.join(project, "addons", "gamestruments", "bin", "gamestruments_godot.wasm"));
  await cp(path.join(import.meta.dirname, "godot-web-smoke.gd"), path.join(project, "web_smoke.gd"));
  await cp(path.join(import.meta.dirname, "godot-web-smoke-scene.gd"), path.join(project, "make_scene.gd"));
  const projectFile = (mainScene) =>
    `config_version=5\n\n[application]\n\nconfig/name="Gamestruments Web Smoke"\n${mainScene}\n[rendering]\n\nrenderer/rendering_method="gl_compatibility"\n`;
  await writeFile(path.join(project, "project.godot"), projectFile(""));
  const exportPath = path.join(exportDir, "index.html");
  await writeFile(path.join(project, "export_presets.cfg"), presets(exportPath));
  runGodot(["--headless", "--path", project, "--import"], "first import", project, { allowAbort: true });
  runGodot(["--headless", "--path", project, "--import"], "import", project);
  const scene = runGodot(["--headless", "--path", project, "--script", "res://make_scene.gd"], "scene generation", project);
  if (!scene.includes("WEB_SMOKE_SCENE_SAVED")) {
    throw new Error("web smoke scene was not generated");
  }
  await writeFile(path.join(project, "project.godot"), projectFile('run/main_scene="res://web_smoke.tscn"\n'));
  runGodot(["--headless", "--path", project, "--export-release", "Web", exportPath], "web export", project);
  const exported = await readFile(path.join(exportDir, "gamestruments_godot.wasm")).catch(() => null);
  if (!exported) {
    throw new Error("web export did not copy the side module next to index.html");
  }
  return exportDir;
}

async function runInBrowser(exportDir) {
  const server = await serve(exportDir);
  const { port } = server.address();
  const browser = await chromium.launch({ args: ["--autoplay-policy=no-user-gesture-required"] });
  const lines = [];
  const problems = [];
  try {
    const page = await browser.newPage({ viewport: { width: 320, height: 180 } });
    page.on("console", (message) => {
      const text = message.text();
      lines.push(`${message.type()}: ${text}`);
      if (message.type() === "error") problems.push(text);
    });
    page.on("pageerror", (error) => problems.push(`pageerror: ${error.message}`));
    const started = Date.now();
    await page.goto(`http://127.0.0.1:${port}/index.html`);
    const isolated = await page.evaluate(() => globalThis.crossOriginIsolated);
    if (isolated || (await page.evaluate(() => typeof SharedArrayBuffer !== "undefined"))) {
      problems.push("page is cross-origin isolated or exposes SharedArrayBuffer; the smoke must run without them");
    }
    const deadline = Date.now() + 180_000;
    while (Date.now() < deadline && !lines.some((line) => /GAMESTRUMENTS_WEB_SMOKE_(PASS|FAIL)/.test(line))) {
      await page.waitForTimeout(250);
    }
    const reportLine = lines.find((line) => line.includes("GAMESTRUMENTS_WEB_SMOKE {"));
    const report = reportLine ? JSON.parse(reportLine.slice(reportLine.indexOf("{"))) : null;
    return { isolated, wallMs: Date.now() - started, report, lines, problems, pass: lines.some((line) => line.includes("GAMESTRUMENTS_WEB_SMOKE_PASS")) };
  } finally {
    await browser.close();
    server.close();
  }
}

const tempRoot = await mkdtemp(path.join(os.tmpdir(), "gamestruments-web-smoke-"));
let failed = false;
try {
  const exportDir = await exportProject(tempRoot);
  const result = await runInBrowser(exportDir);
  if (outDir) {
    await mkdir(outDir, { recursive: true });
    await writeFile(path.join(outDir, "web-smoke.log"), `${result.lines.join("\n")}\n`);
  }
  console.log(JSON.stringify({ isolated: result.isolated, wallMs: result.wallMs, report: result.report }));
  if (!result.pass || result.problems.length > 0) {
    failed = true;
    console.error(`web smoke failed:\n${[...result.problems, ...(result.report?.failures ?? [])].join("\n") || "no PASS marker"}`);
  }
} finally {
  await rm(tempRoot, { recursive: true, force: true });
}
if (failed) {
  process.exit(1);
}
console.log("GAMESTRUMENTS_WEB_EXTENSION_SMOKE_PASS");
