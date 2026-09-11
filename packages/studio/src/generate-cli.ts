#!/usr/bin/env node

import { mkdir, rename, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import {
  createLanternTrailGenerationManifest,
  createPocketCircuitGenerationManifest,
  compactJson,
} from "./generation-manifest.js";
import {
  generateLanternTrailAdventure,
  type LanternTrailGeneratorInput,
  type NormalizedTrailTraits,
} from "./lantern-trail-generator.js";
import {
  generateRacingLevel,
  type NormalizedMusicTraits,
  type PocketCircuitGeneratorInput,
  type PocketCircuitStyle,
} from "./pocket-circuit-generator.js";

const USAGE = `Usage: gamestruments-generate --seed <seed> [--recipe <recipe>] [options]

Generate a deterministic portable score using one of two recipes.

Common options:
  --seed <value>                 Required deterministic seed
  --recipe <value>               pocket-circuit or lantern-trail
                                 (default: pocket-circuit)
  --output <path|->              Score destination (default: stdout)
  --manifest <path|->            Optional generation manifest destination
  --pretty                       Pretty-print JSON (checksum uses compact JSON)
  --help                         Show this help

Pocket Circuit options:
  --style <value>                fusion, neon, funk, or chip (default: fusion)
  --energy <0..1>                Energy trait
  --complexity <0..1>            Complexity trait
  --brightness <0..1>            Brightness trait
  --syncopation <0..1>           Syncopation trait

Lantern Trail options:
  --wonder <0..1>                Wonder trait
  --danger <0..1>                Danger trait
  --mystery <0..1>               Mystery trait
  --motion <0..1>                Motion trait
`;

type GenerationRecipe = "pocket-circuit" | "lantern-trail";

const RECIPES = new Set<GenerationRecipe>([
  "pocket-circuit",
  "lantern-trail",
]);
const STYLES = new Set<PocketCircuitStyle>([
  "fusion",
  "neon",
  "funk",
  "chip",
]);
const VALUE_OPTIONS = new Set([
  "seed",
  "recipe",
  "style",
  "energy",
  "complexity",
  "brightness",
  "syncopation",
  "wonder",
  "danger",
  "mystery",
  "motion",
  "output",
  "manifest",
]);
const FLAG_OPTIONS = new Set(["pretty", "help"]);
const POCKET_TRAIT_OPTIONS = [
  "energy",
  "complexity",
  "brightness",
  "syncopation",
] as const;
const POCKET_ONLY_OPTIONS = ["style", ...POCKET_TRAIT_OPTIONS] as const;
const LANTERN_ONLY_OPTIONS = [
  "wonder",
  "danger",
  "mystery",
  "motion",
] as const;

interface CommonGenerateCliOptions {
  seed: string;
  output: string;
  manifest?: string;
  pretty: boolean;
}

interface PocketCircuitCliOptions extends CommonGenerateCliOptions {
  recipe: "pocket-circuit";
  style: PocketCircuitStyle;
  traits: Partial<NormalizedMusicTraits>;
}

interface LanternTrailCliOptions extends CommonGenerateCliOptions {
  recipe: "lantern-trail";
  traits: Partial<NormalizedTrailTraits>;
}

type GenerateCliOptions = PocketCircuitCliOptions | LanternTrailCliOptions;

function parseTrait(name: string, raw: string): number {
  const value = Number(raw);
  if (!Number.isFinite(value) || value < 0 || value > 1) {
    throw new Error(`--${name} must be a finite number from 0 through 1`);
  }
  return value;
}

function parseArguments(args: readonly string[]): GenerateCliOptions | "help" {
  const values = new Map<string, string>();
  const flags = new Set<string>();

  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === undefined || !argument.startsWith("--")) {
      throw new Error(`Unexpected positional argument: ${argument ?? ""}`);
    }
    const equalsIndex = argument.indexOf("=");
    const name = argument.slice(2, equalsIndex === -1 ? undefined : equalsIndex);
    const inlineValue = equalsIndex === -1 ? undefined : argument.slice(equalsIndex + 1);

    if (FLAG_OPTIONS.has(name)) {
      if (inlineValue !== undefined) {
        throw new Error(`--${name} does not accept a value`);
      }
      if (flags.has(name)) {
        throw new Error(`--${name} was provided more than once`);
      }
      flags.add(name);
      continue;
    }
    if (!VALUE_OPTIONS.has(name)) {
      throw new Error(`Unknown option: --${name}`);
    }
    if (values.has(name)) {
      throw new Error(`--${name} was provided more than once`);
    }

    const value = inlineValue ?? args[index + 1];
    if (value === undefined || value.length === 0 || (inlineValue === undefined && value.startsWith("--"))) {
      throw new Error(`--${name} requires a value`);
    }
    values.set(name, value);
    if (inlineValue === undefined) {
      index += 1;
    }
  }

  if (flags.has("help")) {
    return "help";
  }

  const seed = values.get("seed");
  if (seed === undefined) {
    throw new Error("--seed is required");
  }
  const rawRecipe = values.get("recipe") ?? "pocket-circuit";
  if (!RECIPES.has(rawRecipe as GenerationRecipe)) {
    throw new Error(`--recipe must be one of: ${[...RECIPES].join(", ")}`);
  }
  const recipe = rawRecipe as GenerationRecipe;

  const incompatibleOptions =
    recipe === "pocket-circuit" ? LANTERN_ONLY_OPTIONS : POCKET_ONLY_OPTIONS;
  const incompatible = incompatibleOptions.find((name) => values.has(name));
  if (incompatible !== undefined) {
    const compatibleRecipe =
      recipe === "pocket-circuit" ? "lantern-trail" : "pocket-circuit";
    throw new Error(
      `--${incompatible} is only valid with --recipe ${compatibleRecipe}`,
    );
  }

  const output = values.get("output") ?? "-";
  const manifest = values.get("manifest");
  if (output === "-" && manifest === "-") {
    throw new Error("--output and --manifest cannot both write to stdout");
  }
  if (
    manifest !== undefined &&
    output !== "-" &&
    manifest !== "-" &&
    resolve(output) === resolve(manifest)
  ) {
    throw new Error("--output and --manifest must use different paths");
  }

  const commonOptions: CommonGenerateCliOptions = {
    seed,
    output,
    ...(manifest === undefined ? {} : { manifest }),
    pretty: flags.has("pretty"),
  };

  if (recipe === "lantern-trail") {
    const traits: Partial<NormalizedTrailTraits> = {};
    for (const name of LANTERN_ONLY_OPTIONS) {
      const raw = values.get(name);
      if (raw !== undefined) {
        traits[name] = parseTrait(name, raw);
      }
    }
    return { ...commonOptions, recipe, traits };
  }

  const rawStyle = values.get("style") ?? "fusion";
  if (!STYLES.has(rawStyle as PocketCircuitStyle)) {
    throw new Error(`--style must be one of: ${[...STYLES].join(", ")}`);
  }
  const traits: Partial<NormalizedMusicTraits> = {};
  for (const name of POCKET_TRAIT_OPTIONS) {
    const raw = values.get(name);
    if (raw !== undefined) {
      traits[name] = parseTrait(name, raw);
    }
  }
  return {
    ...commonOptions,
    recipe,
    style: rawStyle as PocketCircuitStyle,
    traits,
  };
}

function serializeJson(value: unknown, pretty: boolean): string {
  return `${pretty ? JSON.stringify(value, null, 2) : compactJson(value)}\n`;
}

async function writeAtomic(path: string, contents: string): Promise<void> {
  const destination = resolve(path);
  await mkdir(dirname(destination), { recursive: true });
  const temporary = `${destination}.tmp-${process.pid}`;
  try {
    await writeFile(temporary, contents, { encoding: "utf8", flag: "wx" });
    await rename(temporary, destination);
  } catch (error) {
    await rm(temporary, { force: true });
    throw error;
  }
}

async function emit(
  destination: string,
  contents: string,
  stdout: NodeJS.WriteStream,
): Promise<void> {
  if (destination === "-") {
    stdout.write(contents);
    return;
  }
  await writeAtomic(destination, contents);
}

export async function runGenerateCli(
  args: readonly string[],
  stdout: NodeJS.WriteStream = process.stdout,
): Promise<void> {
  const options = parseArguments(args);
  if (options === "help") {
    stdout.write(USAGE);
    return;
  }

  if (options.recipe === "lantern-trail") {
    const input: LanternTrailGeneratorInput = {
      seed: options.seed,
      ...(Object.keys(options.traits).length === 0
        ? {}
        : { traits: options.traits }),
    };
    const generated = generateLanternTrailAdventure(input);
    await emit(
      options.output,
      serializeJson(generated.portableScore, options.pretty),
      stdout,
    );
    if (options.manifest !== undefined) {
      await emit(
        options.manifest,
        serializeJson(
          createLanternTrailGenerationManifest(generated),
          options.pretty,
        ),
        stdout,
      );
    }
    return;
  }

  const input: PocketCircuitGeneratorInput = {
    seed: options.seed,
    style: options.style,
    ...(Object.keys(options.traits).length === 0
      ? {}
      : { traits: options.traits }),
  };
  const generated = generateRacingLevel(input);
  await emit(
    options.output,
    serializeJson(generated.portableScore, options.pretty),
    stdout,
  );
  if (options.manifest !== undefined) {
    await emit(
      options.manifest,
      serializeJson(
        createPocketCircuitGenerationManifest(generated),
        options.pretty,
      ),
      stdout,
    );
  }
}

runGenerateCli(process.argv.slice(2)).catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`gamestruments-generate: ${message}\n`);
  process.exitCode = 1;
});
