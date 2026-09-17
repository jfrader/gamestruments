import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const projectRoot = path.resolve(import.meta.dirname, "..");
const wasmPath = path.join(projectRoot, "apps", "demo", "public", "engine", "gamestruments_engine.wasm");
const outDir = process.argv[2] || "/tmp/opencode/ab";

const INPUT = {
  recipe: "suspense",
  secret: "",
  seed: "ab-take-42",
  style: "terminal",
  arrangement: "seeded",
  intent: "arc",
  tension: 0.62,
  heat: 0.48,
  mystery: 0.72,
  pulse: 0.55,
  reelIndex: 3,
};
const SECTION = "verse";
const CHUNK_PHRASES = 0.2;
const TOTAL_PHRASES = 1.6; // ~40.7 s dense (crosses loop boundary once)

async function loadWasm() {
  const bytes = await readFile(wasmPath);
  const { instance } = await WebAssembly.instantiate(bytes, {});
  if (!instance.exports.memory) {
    throw new Error("wasm instance has no exported memory");
  }
  return instance.exports;
}

function getMem(exports) {
  return new Uint8Array(exports.memory.buffer);
}

function allocAndWrite(exports, data) {
  const bytes = data instanceof Uint8Array ? data : new TextEncoder().encode(data);
  const ptr = exports.gamestruments_alloc(bytes.length);
  if (ptr === 0 || ptr == null) {
    throw new Error("gamestruments_alloc failed (out of bump?)");
  }
  const mem = getMem(exports);
  mem.set(bytes, ptr);
  return { ptr, len: bytes.length };
}

function readOutput(exports, ptr) {
  const len = exports.gamestruments_output_len();
  const mem = getMem(exports);
  const bytes = mem.slice(ptr, ptr + len);
  const status = exports.gamestruments_status();
  if (status !== 0) {
    throw new Error(new TextDecoder().decode(bytes));
  }
  return bytes;
}

async function generateScore(exports) {
  exports.gamestruments_reset();
  const { ptr, len } = allocAndWrite(exports, JSON.stringify(INPUT));
  const outPtr = exports.gamestruments_score_json(ptr, len);
  return readOutput(exports, outPtr);
}

async function renderMonoChunk(exports, scoreBytes, startPhrases, phrases) {
  exports.gamestruments_reset();
  const s = allocAndWrite(exports, scoreBytes);
  const sec = allocAndWrite(exports, SECTION);
  const outPtr = exports.gamestruments_render_wav_chunk(s.ptr, s.len, sec.ptr, sec.len, startPhrases, phrases);
  return readOutput(exports, outPtr);
}

async function renderStereoChunk(exports, scoreBytes, startPhrases, phrases) {
  exports.gamestruments_reset();
  const s = allocAndWrite(exports, scoreBytes);
  const sec = allocAndWrite(exports, SECTION);
  const outPtr = exports.gamestruments_render_wav_stereo_chunk(s.ptr, s.len, sec.ptr, sec.len, startPhrases, phrases);
  return readOutput(exports, outPtr);
}

function extractPcmData(wavBytes) {
  const b = Buffer.from(wavBytes);
  const channels = b.readUInt16LE(22);
  const sr = b.readUInt32LE(24);
  const bits = b.readUInt16LE(34);
  const dataSize = b.readUInt32LE(40);
  const pcm = b.subarray(44, 44 + dataSize);
  return { pcm, channels, sr, bits, dataSize };
}

function makeWav(pcmData, channels, sr) {
  const dataSize = pcmData.length;
  const wav = Buffer.alloc(44 + dataSize);
  wav.write("RIFF", 0, "ascii");
  wav.writeUInt32LE(36 + dataSize, 4);
  wav.write("WAVEfmt ", 8, "ascii");
  wav.writeUInt32LE(16, 16);
  wav.writeUInt16LE(1, 20); // PCM
  wav.writeUInt16LE(channels, 22);
  wav.writeUInt32LE(sr, 24);
  const blockAlign = channels * 2;
  wav.writeUInt32LE(sr * blockAlign, 28);
  wav.writeUInt16LE(blockAlign, 32);
  wav.writeUInt16LE(16, 34);
  wav.write("data", 36, "ascii");
  wav.writeUInt32LE(dataSize, 40);
  pcmData.copy(wav, 44);
  return wav;
}

function pcmToSeconds(pcmBytes, channels) {
  const numSamples = pcmBytes.length / (channels * 2);
  return numSamples / 48000;
}

async function main() {
  console.log("render-ab: loading", wasmPath);
  const exports = await loadWasm();

  console.log("render-ab: generating score for SAME take (reelIndex=" + INPUT.reelIndex + ")");
  const scoreBytes = await generateScore(exports);
  const scoreObj = JSON.parse(new TextDecoder().decode(scoreBytes));
  const tps = scoreObj.bpm * scoreObj.ticksPerBeat / 60;
  const sectionInfo = scoreObj.sections.find((s) => s.id === SECTION);
  const phraseSec = sectionInfo.lengthTicks / tps;
  console.log("render-ab: score bytes:", scoreBytes.length, "section=" + SECTION + " phrase~=" + phraseSec.toFixed(2) + "s");

  const targetSec = TOTAL_PHRASES * phraseSec;
  console.log("render-ab: rendering dense " + (targetSec.toFixed(1)) + "s chunks (chunk=" + CHUNK_PHRASES + ") of SAME take ...");

  const numChunks = Math.ceil(TOTAL_PHRASES / CHUNK_PHRASES);
  const monoPcmParts = [];
  const stereoPcmParts = [];
  for (let i = 0; i < numChunks; i++) {
    const sp = i * CHUNK_PHRASES;
    let ph = CHUNK_PHRASES;
    if (sp + ph > TOTAL_PHRASES) ph = TOTAL_PHRASES - sp;
    if (ph <= 0) break;

    const mWav = await renderMonoChunk(exports, scoreBytes, sp, ph);
    const m = extractPcmData(mWav);
    if (m.sr !== 48000 || m.bits !== 16 || m.channels !== 1) {
      throw new Error("bad mono chunk fmt");
    }
    monoPcmParts.push(m.pcm);

    const sWav = await renderStereoChunk(exports, scoreBytes, sp, ph);
    const s = extractPcmData(sWav);
    if (s.sr !== 48000 || s.bits !== 16 || s.channels !== 2) {
      throw new Error("bad stereo chunk fmt");
    }
    stereoPcmParts.push(s.pcm);
  }

  const monoPcm = Buffer.concat(monoPcmParts);
  const stereoPcm = Buffer.concat(stereoPcmParts);
  const monoWav = makeWav(monoPcm, 1, 48000);
  const stereoWav = makeWav(stereoPcm, 2, 48000);

  await mkdir(outDir, { recursive: true });
  const monoPath = path.join(outDir, "mono.wav");
  const stereoPath = path.join(outDir, "stereo.wav");
  await writeFile(monoPath, monoWav);
  await writeFile(stereoPath, stereoWav);

  const monoDur = pcmToSeconds(monoPcm, 1);
  const stereoDur = pcmToSeconds(stereoPcm, 2);
  console.log("render-ab: wrote");
  console.log("  mono:", monoPath, "(" + monoWav.length + " bytes, " + monoDur.toFixed(2) + "s)");
  console.log("  stereo:", stereoPath, "(" + stereoWav.length + " bytes, " + stereoDur.toFixed(2) + "s)");
}

main().catch((err) => {
  console.error("render-ab FAILED:", err);
  process.exit(1);
});
