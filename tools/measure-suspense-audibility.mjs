import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const projectRoot = path.resolve(import.meta.dirname, "..");
const wasmPath = path.join(projectRoot, "apps", "demo", "public", "engine", "gamestruments_engine.wasm");

async function loadWasm() {
  const bytes = await readFile(wasmPath);
  const { instance } = await WebAssembly.instantiate(bytes, {});
  if (!instance.exports.memory) {
    throw new Error("wasm instance has no exported memory");
  }
  if (typeof instance.exports.gamestruments_alloc !== "function") {
    throw new Error("wasm missing gamestruments_alloc (old build?)");
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
    throw new Error("gamestruments_alloc failed");
  }
  const mem = getMem(exports);
  mem.set(bytes, ptr);
  return { ptr, len: bytes.length };
}

function readResponse(exports, ptr) {
  const len = exports.gamestruments_output_len();
  const status = exports.gamestruments_status();
  const mem = getMem(exports);
  const bytes = mem.slice(ptr, ptr + len);
  return { bytes, status };
}

function readOutput(exports, ptr) {
  const { bytes, status } = readResponse(exports, ptr);
  if (status !== 0) {
    throw new Error("WASM error: " + new TextDecoder().decode(bytes));
  }
  return bytes;
}

async function genScore(exports, traits) {
  exports.gamestruments_reset();
  const input = {
    recipe: "suspense",
    secret: "measure-aud",
    seed: "guri-873-seed",
    style: "terminal",
    arrangement: "seeded",
    intent: "arc",
    tension: traits.tension,
    heat: 0.5,
    mystery: 0.5,
    pulse: 0.5,
    reelIndex: 0,
  };
  const json = JSON.stringify(input);
  const w = allocAndWrite(exports, json);
  const outPtr = exports.gamestruments_score_json(w.ptr, w.len);
  const out = readOutput(exports, outPtr);
  return JSON.parse(new TextDecoder().decode(out));
}

function summarize(score, label) {
  const BAR = 3840; // suspense fixed: 4*960
  const lanes = {};
  const perc = {};
  const phaseLens = {};
  const rhythmSets = new Set();
  const figPatterns = new Set();
  for (const sec of score.sections) {
    const lt = sec.length_ticks ?? sec.lengthTicks ?? 0;
    const bars = Math.max(1, Math.round(lt / BAR));
    phaseLens[sec.id] = bars;
    const kitSteps = [];
    for (const ev of (sec.events || sec.Events || [])) {
      if (ev.lane) {
        const cls = ev.lane.replace(/^(.*-)?/, "").replace(/-.*/, "") || ev.lane;
        lanes[cls] = (lanes[cls] || 0) + 1;
      }
      if (ev.voice) {
        perc[ev.voice] = (perc[ev.voice] || 0) + 1;
        if (typeof ev.start_tick === "number") {
          const rel = ev.start_tick % BAR;
          rhythmSets.add(`${sec.id}:${rel}`);
          if (ev.voice === "kick" || ev.voice === "hat" || ev.voice === "snare" || ev.voice === "tom") {
            kitSteps.push(rel);
          }
        }
      }
    }
    if (kitSteps.length) {
      const pat = [...new Set(kitSteps)].sort((x,y)=>x-y).join(",");
      figPatterns.add(`${sec.id}:${pat}`);
    }
  }
  const distinctFigs = figPatterns.size;
  return { label, nSections: score.sections.length, phaseLens, laneCounts: lanes, percCounts: perc, distinctFigPatterns: distinctFigs, totalEvents: score.sections.reduce((a,s)=>a+(s.events||[]).length,0) };
}

function printDiff(a, b) {
  console.log("=== AUDIBILITY MEASUREMENT (same seed/style/arr/reel) ===");
  console.log("LOW:", JSON.stringify(a, null, 2));
  console.log("HIGH:", JSON.stringify(b, null, 2));
  // show clear numeric diffs
  const dPhases = Object.keys(a.phaseLens).filter(k => a.phaseLens[k] !== b.phaseLens[k]).length;
  const dLanes = Object.keys({...a.laneCounts, ...b.laneCounts}).filter(k => (a.laneCounts[k]||0) !== (b.laneCounts[k]||0)).length;
  const dFigs = (b.distinctFigPatterns || 0) - (a.distinctFigPatterns || 0);
  const dTot = b.totalEvents - a.totalEvents;
  console.log(`DIFF: phase-lengths-changed=${dPhases}, lane-counts-differ=${dLanes}, fig-delta=${dFigs}, events-delta=${dTot}`);
  console.log("CLEAR DIFFERENCE:", (dPhases + dLanes + Math.abs(dFigs) + Math.abs(dTot) > 0) ? "YES" : "NO");
}

async function main() {
  const exports = await loadWasm();
  const low = await genScore(exports, { tension: 0.2 });
  const high = await genScore(exports, { tension: 0.9 });
  const sa = summarize(low, "tension=0.2");
  const sb = summarize(high, "tension=0.9");
  printDiff(sa, sb);
}

main().catch(e => { console.error(e); process.exit(1); });
