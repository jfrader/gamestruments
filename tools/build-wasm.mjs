import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFile, mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";

const projectRoot = path.resolve(import.meta.dirname, "..");
const cargoHome = process.env.CARGO_HOME ?? path.join(os.homedir(), ".cargo");
const destinationDir = path.join(projectRoot, "apps", "demo", "public", "engine");
const destination = path.join(destinationDir, "gamestruments_engine.wasm");
const sourceStamp = path.join(destinationDir, "gamestruments_engine.source.sha256");

async function sourceFiles(inputPath) {
  const details = await stat(inputPath);
  if (details.isFile()) {
    return [inputPath];
  }
  const entries = await readdir(inputPath);
  const nested = await Promise.all(
    entries.sort().map((entry) => sourceFiles(path.join(inputPath, entry))),
  );
  return nested.flat();
}

async function sourceDigest() {
  const inputs = [
    path.join(projectRoot, "Cargo.toml"),
    path.join(projectRoot, "Cargo.lock"),
    path.join(projectRoot, "rust-toolchain.toml"),
    path.join(projectRoot, "crates", "engine"),
  ];
  const files = (await Promise.all(inputs.map(sourceFiles))).flat().sort();
  const hash = createHash("sha256");
  for (const file of files) {
    hash.update(path.relative(projectRoot, file).split(path.sep).join("/"));
    hash.update("\0");
    hash.update(await readFile(file));
    hash.update("\0");
  }
  return hash.digest("hex");
}

const digest = await sourceDigest();
if (process.argv.includes("--verify-source")) {
  await stat(destination);
  const expected = (await readFile(sourceStamp, "utf8")).trim();
  if (expected !== digest) {
    throw new Error("Committed WASM is stale; run npm run wasm:build");
  }
  console.log("WASM_SOURCE_VERIFY_PASS");
  process.exit(0);
}

const rustflags = [
  process.env.RUSTFLAGS,
  `--remap-path-prefix=${projectRoot}=/workspace`,
  `--remap-path-prefix=${cargoHome}=/cargo`,
]
  .filter(Boolean)
  .join(" ");

const result = spawnSync(
  "cargo",
  [
    "build",
    "-p",
    "gamestruments-engine",
    "--target",
    "wasm32-unknown-unknown",
    "--release",
    "--locked",
  ],
  {
    cwd: projectRoot,
    env: {
      ...process.env,
      CARGO_INCREMENTAL: "0",
      RUSTFLAGS: rustflags,
    },
    stdio: "inherit",
  },
);

if (result.status !== 0 || result.error) {
  throw new Error(
    `WASM build failed with status ${String(result.status)}${result.error ? `: ${result.error.message}` : ""}`,
  );
}

const source = path.join(
  projectRoot,
  "target",
  "wasm32-unknown-unknown",
  "release",
  "gamestruments_engine.wasm",
);
await mkdir(destinationDir, { recursive: true });
await copyFile(source, destination);
await writeFile(sourceStamp, `${digest}\n`);
