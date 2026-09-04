import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.join(__dirname, "..");
const wasmPath = path.join(projectRoot, "target/wasm32-unknown-unknown/release/gamestruments_engine.wasm");

const FUNK_INPUT = `{"secret":"parity-secret","seed":"parity-001","style":"funk","palette":{"melody":"","harmony":"","drive":"","bass":""},"energy":0.62,"complexity":0.6,"brightness":0.52,"syncopation":0.7}`;
const CHIP_INPUT = `{"secret":"parity-secret","seed":"parity-001","style":"chip","palette":{"melody":"chip","harmony":"chip","drive":"chip","bass":"triangle"},"energy":0.62,"complexity":0.6,"brightness":0.52,"syncopation":0.7}`;

async function loadWasm() {
  const bytes = await readFile(wasmPath);
  const { instance } = await WebAssembly.instantiate(bytes, {});
  if (!instance.exports.memory) {
    throw new Error("wasm instance has no exported memory");
  }
  return instance.exports;
}

function allocAndWrite(exports, data) {
  const bytes = data instanceof Uint8Array ? data : new TextEncoder().encode(data);
  const ptr = exports.gamestruments_alloc(bytes.length);
  if (ptr === 0 || ptr == null) {
    throw new Error("gamestruments_alloc failed (out of bump?)");
  }
  const mem = new Uint8Array(exports.memory.buffer);
  mem.set(bytes, ptr);
  return { ptr, len: bytes.length };
}

function readOutput(exports, ptr) {
  const len = exports.gamestruments_output_len();
  const mem = new Uint8Array(exports.memory.buffer);
  // Always copy to avoid aliasing on next call
  return mem.slice(ptr, ptr + len);
}

async function wasmScoreJson(exports, inputJson) {
  exports.gamestruments_reset();
  const { ptr, len } = allocAndWrite(exports, inputJson);
  const outPtr = exports.gamestruments_score_json(ptr, len);
  return readOutput(exports, outPtr);
}

async function wasmRenderWav(exports, scoreBytes, section, phrases) {
  exports.gamestruments_reset();
  const s = allocAndWrite(exports, scoreBytes);
  const sec = allocAndWrite(exports, section);
  const outPtr = exports.gamestruments_render_wav(s.ptr, s.len, sec.ptr, sec.len, phrases);
  return readOutput(exports, outPtr);
}

async function main() {
  console.log("wasm-parity: loading", wasmPath);
  const exports = await loadWasm();

  // Funk case (default palette)
  const wasmFunkScore = await wasmScoreJson(exports, FUNK_INPUT);
  await writeFile(path.join(projectRoot, "parity-wasm-funk-score.json"), wasmFunkScore);
  const wasmFunkWav = await wasmRenderWav(exports, wasmFunkScore, "cruise", 3);
  await writeFile(path.join(projectRoot, "parity-wasm-funk-render.wav"), wasmFunkWav);
  console.log("wasm-parity: wrote parity-wasm-funk-* (score %d, wav %d)", wasmFunkScore.length, wasmFunkWav.length);

  // Chip case (custom palette)
  const wasmChipScore = await wasmScoreJson(exports, CHIP_INPUT);
  await writeFile(path.join(projectRoot, "parity-wasm-chip-score.json"), wasmChipScore);
  const wasmChipWav = await wasmRenderWav(exports, wasmChipScore, "cruise", 3);
  await writeFile(path.join(projectRoot, "parity-wasm-chip-render.wav"), wasmChipWav);
  console.log("wasm-parity: wrote parity-wasm-chip-* (score %d, wav %d)", wasmChipScore.length, wasmChipWav.length);

  // Compare to native references (must exist from parity_ref)
  const nativeFunkScore = new Uint8Array(await readFile(path.join(projectRoot, "parity-funk-score.json")));
  const nativeFunkWav = new Uint8Array(await readFile(path.join(projectRoot, "parity-funk-render.wav")));
  const nativeChipScore = new Uint8Array(await readFile(path.join(projectRoot, "parity-chip-score.json")));
  const nativeChipWav = new Uint8Array(await readFile(path.join(projectRoot, "parity-chip-render.wav")));

  assert.deepStrictEqual(wasmFunkScore, nativeFunkScore, "funk score JSON must be byte-identical");
  assert.deepStrictEqual(wasmChipScore, nativeChipScore, "chip score JSON must be byte-identical");
  console.log("wasm-parity: score JSON byte-identical vs native: PASS");

  // WAVs: require identical length + max sample diff <=1 (accounts for f32 math / libm rounding variance
  // between native x86_64 and wasm32-unknown-unknown while proving the render pipeline is equivalent)
  function maxSampleDiff(a, b) {
    if (a.length !== b.length) return Infinity;
    let m = 0;
    for (let i = 0; i < a.length; i++) {
      const d = Math.abs(a[i] - b[i]);
      if (d > m) m = d;
    }
    return m;
  }
  const funkWavDiff = maxSampleDiff(wasmFunkWav, nativeFunkWav);
  const chipWavDiff = maxSampleDiff(wasmChipWav, nativeChipWav);
  assert.ok(funkWavDiff <= 1, `funk WAV max diff ${funkWavDiff} > 1`);
  assert.ok(chipWavDiff <= 1, `chip WAV max diff ${chipWavDiff} > 1`);
  console.log("wasm-parity: WAV max-diff<=1 vs native: PASS (funk=%d, chip=%d)", funkWavDiff, chipWavDiff);

  // WASM determinism (run twice)
  const d1 = await wasmScoreJson(exports, FUNK_INPUT);
  const d2 = await wasmScoreJson(exports, FUNK_INPUT);
  assert.deepStrictEqual(d1, d2, "wasm score JSON must be stable across runs");
  const w1 = await wasmRenderWav(exports, d1, "cruise", 3);
  const w2 = await wasmRenderWav(exports, d2, "cruise", 3);
  assert.deepStrictEqual(w1, w2, "wasm WAV must be stable across runs");
  console.log("wasm-parity: determinism check: PASS");

  console.log("wasm-parity: ALL CHECKS PASSED");
}

main().catch((err) => {
  console.error("wasm-parity FAILED:", err);
  process.exit(1);
});
