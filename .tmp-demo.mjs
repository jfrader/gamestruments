import { chromium } from "@playwright/test";
import { mkdirSync, writeFileSync } from "node:fs";
const dir = "/tmp/opencode/demo"; mkdirSync(dir, { recursive: true });
const browser = await chromium.launch({ args: ["--autoplay-policy=no-user-gesture-required"] });
const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, recordVideo: { dir, size: { width: 1280, height: 720 } } });
await page.addInitScript(() => {
  const OrigAC = window.AudioContext || window.webkitAudioContext;
  const proto = AudioNode.prototype, origConnect = proto.connect;
  window.__chunks = [];
  function makeTap(ctx) { if (window.__tapDest) return; window.__tapCtx = ctx; window.__tapDest = ctx.createMediaStreamDestination();
    const rec = new MediaRecorder(window.__tapDest.stream, { mimeType: "audio/webm" });
    rec.ondataavailable = (e) => { if (e.data.size) window.__chunks.push(e.data); }; window.__recorder = rec; }
  window.AudioContext = class extends OrigAC { constructor(...a) { super(...a); makeTap(this); } };
  proto.connect = function (t, ...r) { if (t && t.context === window.__tapCtx && t === t.context.destination) { try { origConnect.call(this, window.__tapDest); } catch {} } return origConnect.call(this, t, ...r); };
});
await page.goto("https://gamestruments.gurisitos.games/#lab", { waitUntil: "domcontentloaded" });
await page.waitForSelector("#section-list li", { timeout: 30000 });
const timeline = [];
let t = 0;
const wait = async (sec, caption) => { if (caption) timeline.push({ t, caption }); await page.waitForTimeout(sec * 1000); t += sec; };
const pick = async (recipe, style) => {
  await page.locator("#recipe-select-trigger").click();
  await page.locator(`#recipe-select-menu button[data-recipe="${recipe}"]`).click();
  await page.waitForTimeout(800); t += 0.8;
  await page.locator(`#score-buttons button:has-text("${style}")`).first().click({ timeout: 5000 }).catch(() => {});
  await page.locator("#level-seed").fill("demo-" + Math.random().toString(36).slice(2, 6));
  await page.locator("#apply-seed").click();
  await page.waitForTimeout(900); t += 0.9;
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click({ timeout: 4000 }).catch(() => {});
  await page.waitForTimeout(400); t += 0.4;
};
const cue = async (label) => { await page.getByRole("button", { name: "Cue " + label, exact: true }).first().click({ timeout: 4000 }).catch(() => {}); };
const knob = async (v) => { await page.locator("#generation-energy").evaluate((el, val) => { el.value = String(val); el.dispatchEvent(new Event("input", { bubbles: true })); el.dispatchEvent(new Event("change", { bubbles: true })); }, v); };

await pick("suspense", "noir");
await wait(2.5, "Pick a recipe, a style and a seed — deterministic, no samples");
await page.locator("#center-play").click();
await wait(6, "Press play: the pool composes the whole song");
await cue("Anomaly");
await wait(14, "Cue Anomaly — a machine detour inside the harmonic arc");
await knob(0.95);
await wait(10, "Move tension — the same take reshapes; the label shows the phase");
await page.locator("#start-audio").click().catch(() => {});
await wait(1.5, "Three game types, ten styles — one engine");

await pick("racing", "neon");
await wait(1.5, "Racing · Neon");
await page.locator("#center-play").click();
await wait(3, "Press play");
await cue("Starting Grid");
await wait(14, "Starting Grid — the race bed that never drops its drums");
await cue("Drum Break");
await wait(12, "Drum Break — a deliberate drumless drop, cued by the game");
await knob(0.95);
await wait(8, "Push energy — faster and denser");
await page.locator("#start-audio").click().catch(() => {});
await wait(1.5, null);

await pick("adventure", "orchestral");
await wait(1.5, "Adventure · Orchestral");
await page.locator("#center-play").click();
await wait(3, "Press play");
await cue("Boss");
await wait(12, "Boss — the combat peak the composer places");
await cue("Combat");
await wait(10, "Combat — drive under the fight, state-driven");
await knob(0.95);
await wait(8, "Raise danger — darker, louder, still the same engine");
await page.locator("#start-audio").click().catch(() => {});
await wait(2, "Try the demo — gamestruments.gurisitos.games");

const b64 = await page.evaluate(() => new Promise((res) => { try { window.__recorder.onstop = async () => { const b = new Blob(window.__chunks, { type: "audio/webm" }); const u = new Uint8Array(await b.arrayBuffer()); let s = ""; for (let i = 0; i < u.length; i++) s += String.fromCharCode(u[i]); res(btoa(s)); }; window.__recorder.stop(); } catch (e) { res(""); } }));
const v = page.video(); await page.close();
const apath = `${dir}/session.audio.webm`; writeFileSync(apath, Buffer.from(b64, "base64"));
writeFileSync(`${dir}/timeline.json`, JSON.stringify({ timeline, t, video: await v.path(), audio: apath }, null, 2));
console.log("timeline beats:", timeline.length, "total t=" + t.toFixed(1));
await browser.close();
