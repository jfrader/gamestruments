import {
  type PortableScore,
  validatePortableScore,
} from "../../../packages/runtime/src/index.ts";
import { WasmPlayer } from "./wasm-player.ts";

let moduleRef: WebAssembly.Module | null = null;
let exportsRef: WebAssembly.Exports | null = null;
let memoryRef: WebAssembly.Memory | null = null;

async function ensureLoaded(): Promise<void> {
  if (exportsRef) {
    return;
  }
  const response = await fetch(`${import.meta.env.BASE_URL}engine/gamestruments_engine.wasm`);
  if (!response.ok) {
    throw new Error(`Failed to load WASM engine: ${response.status}`);
  }
  const { module, instance } = await WebAssembly.instantiateStreaming(response, {});
  if (!instance.exports.memory) {
    throw new Error("WASM module has no exported memory");
  }
  moduleRef = module;
  exportsRef = instance.exports;
  memoryRef = instance.exports.memory as WebAssembly.Memory;
}

function getExports(): WebAssembly.Exports {
  if (!exportsRef) {
    throw new Error("WASM engine not loaded");
  }
  return exportsRef;
}

function getMemory(): Uint8Array {
  const mem = (memoryRef ?? getExports().memory) as WebAssembly.Memory;
  return new Uint8Array(mem.buffer);
}

function allocAndWrite(data: string | Uint8Array): { ptr: number; len: number } {
  const bytes = data instanceof Uint8Array ? data : new TextEncoder().encode(data);
  const exp = getExports();
  const ptr = (exp.gamestruments_alloc as (size: number) => number)(bytes.length);
  if (ptr === 0 || ptr == null) {
    throw new Error("gamestruments_alloc failed");
  }
  const mem = getMemory();
  mem.set(bytes, ptr);
  return { ptr, len: bytes.length };
}

function readOutput(ptr: number): Uint8Array {
  const exp = getExports();
  const len = (exp.gamestruments_output_len as () => number)();
  const mem = getMemory();
  // copy to detach from possible future growth/overwrite
  const bytes = mem.slice(ptr, ptr + len);
  const status = (exp.gamestruments_status as () => number)();
  if (status !== 0) {
    throw new Error(new TextDecoder().decode(bytes));
  }
  return bytes;
}

export type Arrangement = "all-phases" | "seeded";

export function isArrangement(value: string): value is Arrangement {
  return value === "all-phases" || value === "seeded";
}

export interface GenerateScoreParams {
  seed: string;
  style: string;
  energy: number;
  complexity: number;
  brightness: number;
  syncopation: number;
  recipe?: "racing" | "suspense" | "adventure";
  arrangement?: Arrangement;
  intent?: "loop" | "arc" | "long" | "surprise";
  autoplay?: boolean;
  tension?: number;
  heat?: number;
  mystery?: number;
  pulse?: number;
  /** Seed-reel take index (1-based, unbounded). The engine derives the take
   *  seed from `secret` + `seed` + this index, so takes jump directly. */
  reelIndex?: number;
}

/** The generation input JSON the engine takes, for `params`. */
function generationInput(params: GenerateScoreParams): string {
  return JSON.stringify({
    secret: "",
    seed: params.seed,
    style: params.style,
    recipe: params.recipe ?? "racing",
    // Every recipe's arrangements are `all-phases` or `seeded`, so an
    // unspecified arrangement defaults to the seeded composer.
    arrangement: params.arrangement ?? "seeded",
    intent: params.intent ?? "arc",
    autoplay: params.autoplay ?? false,
    palette: { melody: "", harmony: "", drive: "", bass: "" },
    energy: params.energy,
    complexity: params.complexity,
    brightness: params.brightness,
    syncopation: params.syncopation,
    tension: params.tension ?? params.energy,
    heat: params.heat ?? params.complexity,
    mystery: params.mystery ?? params.brightness,
    pulse: params.pulse ?? params.syncopation,
    reelIndex: params.reelIndex ?? 0,
  });
}

export async function generateScore(params: GenerateScoreParams): Promise<PortableScore> {
  await ensureLoaded();
  const exp = getExports();
  (exp.gamestruments_reset as () => void)();
  const { ptr, len } = allocAndWrite(generationInput(params));
  const outPtr = (exp.gamestruments_score_json as (p: number, l: number) => number)(ptr, len);
  const bytes = readOutput(outPtr);
  const text = new TextDecoder().decode(bytes);
  const score = JSON.parse(text) as PortableScore;
  validatePortableScore(score);
  return score;
}

export async function renderWav(
  score: PortableScore,
  section: string,
  phrases: number,
): Promise<Uint8Array> {
  await ensureLoaded();
  const exp = getExports();
  (exp.gamestruments_reset as () => void)();

  const scoreBytes = new TextEncoder().encode(JSON.stringify(score));
  const s = allocAndWrite(scoreBytes);
  const sec = allocAndWrite(section);
  const outPtr = (exp.gamestruments_render_wav as (
    sp: number,
    sl: number,
    secp: number,
    secl: number,
    ph: number,
  ) => number)(s.ptr, s.len, sec.ptr, sec.len, phrases);
  return readOutput(outPtr);
}

export async function renderWavStereo(
  score: PortableScore,
  section: string,
  phrases: number,
): Promise<Uint8Array> {
  await ensureLoaded();
  const exp = getExports();
  (exp.gamestruments_reset as () => void)();

  const scoreBytes = new TextEncoder().encode(JSON.stringify(score));
  const s = allocAndWrite(scoreBytes);
  const sec = allocAndWrite(section);
  const outPtr = (exp.gamestruments_render_wav_stereo as (
    sp: number,
    sl: number,
    secp: number,
    secl: number,
    ph: number,
  ) => number)(s.ptr, s.len, sec.ptr, sec.len, phrases);
  return readOutput(outPtr);
}

/** The key the score for `params` sounds in, for the live player's `load`. */
export async function rootPitchClass(params: GenerateScoreParams): Promise<number> {
  await ensureLoaded();
  const exp = getExports();
  (exp.gamestruments_reset as () => void)();
  const { ptr, len } = allocAndWrite(generationInput(params));
  return (exp.gamestruments_root_pitch_class as (p: number, l: number) => number)(ptr, len);
}

/** A live player on its own engine instance. */
export async function createPlayer(sampleRate: number): Promise<WasmPlayer> {
  await ensureLoaded();
  if (moduleRef === null) {
    throw new Error("WASM engine not loaded");
  }
  return new WasmPlayer(await WebAssembly.instantiate(moduleRef, {}), sampleRate);
}
