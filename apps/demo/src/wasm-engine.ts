import {
  type PortableScore,
  validatePortableScore,
} from "../../../packages/runtime/src/index.ts";

let exportsRef: WebAssembly.Exports | null = null;
let memoryRef: WebAssembly.Memory | null = null;

async function ensureLoaded(): Promise<void> {
  if (exportsRef) {
    return;
  }
  const response = await fetch("/engine/gamestruments_engine.wasm");
  if (!response.ok) {
    throw new Error(`Failed to load WASM engine: ${response.status}`);
  }
  const { instance } = await WebAssembly.instantiateStreaming(response, {});
  if (!instance.exports.memory) {
    throw new Error("WASM module has no exported memory");
  }
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

export interface GenerateScoreParams {
  seed: string;
  style: string;
  energy: number;
  complexity: number;
  brightness: number;
  syncopation: number;
  recipe?: "pocket-circuit" | "suspense";
  tension?: number;
  heat?: number;
  mystery?: number;
  pulse?: number;
}

export async function generateScore(params: GenerateScoreParams): Promise<PortableScore> {
  await ensureLoaded();
  const exp = getExports();
  (exp.gamestruments_reset as () => void)();

  const input = {
    secret: "",
    seed: params.seed,
    style: params.style,
    recipe: params.recipe ?? "pocket-circuit",
    palette: { melody: "", harmony: "", drive: "", bass: "" },
    energy: params.energy,
    complexity: params.complexity,
    brightness: params.brightness,
    syncopation: params.syncopation,
    tension: params.tension ?? params.energy,
    heat: params.heat ?? params.complexity,
    mystery: params.mystery ?? params.brightness,
    pulse: params.pulse ?? params.syncopation,
  };
  const { ptr, len } = allocAndWrite(JSON.stringify(input));
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
