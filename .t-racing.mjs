import { chromium } from "@playwright/test";
import { writeFileSync } from "node:fs";
const dir = "/tmp/opencode/racing-cand";
const STYLES = ["fusion", "neon", "funk"];
const browser = await chromium.launch({ args: ["--autoplay-policy=no-user-gesture-required"] });
const takes = [];
for (const style of STYLES) {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 }, recordVideo: { dir, size: { width: 1280, height: 720 } } });
  await page.addInitScript(() => {
    const OrigAC = window.AudioContext || window.webkitAudioContext;
    const proto = AudioNode.prototype, origConnect = proto.connect;
    window.__chunks = [];
    let mixer = null, mixDest = null, tapping = false;
    function ensure() { if (mixer) return; mixer = new OrigAC(); mixDest = mixer.createMediaStreamDestination();
      const rec = new MediaRecorder(mixDest.stream, { mimeType: "audio/webm" });
      rec.ondataavailable = (e) => { if (e.data.size) window.__chunks.push(e.data); }; window.__recorder = rec; }
    proto.connect = function (t, ...r) {
      if (!tapping && t && t.context && t === t.context.destination) {
        try { ensure(); const tap = t.context.createMediaStreamDestination(); origConnect.call(this, tap);
          tapping = true; origConnect.call(mixer.createMediaStreamSource(tap.stream), mixDest); tapping = false; } catch (e) {}
      }
      return origConnect.call(this, t, ...r);
    };
    window.AudioContext = class extends OrigAC { constructor(...a) { super(...a); try { ensure(); } catch (e) {} } };
    function cursor() { const c = document.createElement("div"); c.id = "__cur";
      c.style.cssText = "position:fixed;z-index:2147483647;width:20px;height:20px;margin:-10px 0 0 -10px;border-radius:50%;background:rgba(255,255,255,.9);box-shadow:0 0 0 2px rgba(0,0,0,.65);pointer-events:none;left:-100px;top:-100px;transition:transform .06s linear";
      document.documentElement.appendChild(c);
      document.addEventListener("mousemove", (e) => { c.style.left = e.clientX + "px"; c.style.top = e.clientY + "px"; }, true);
      document.addEventListener("mousedown", () => { c.style.transform = "scale(1.7)"; }, true);
      document.addEventListener("mouseup", () => { c.style.transform = "scale(1)"; }, true); }
    if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", cursor); else cursor();
  });
  await page.goto("https://gamestruments.gurisitos.games/#lab", { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#section-list li", { timeout: 30000 });
  await page.locator("#recipe-select-trigger").click();
  await page.locator('#recipe-select-menu button[data-recipe="racing"]').click();
  await page.waitForTimeout(800);
  await page.locator(`#score-buttons button:has-text("${style}")`).first().click({ timeout: 5000 });
  await page.locator("#level-seed").fill("race-" + Math.random().toString(36).slice(2, 6));
  await page.locator("#apply-seed").click();
  await page.waitForTimeout(900);
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click({ timeout: 4000 }).catch(() => {});
  await page.waitForTimeout(400);
  await page.locator("#center-play").click();
  await page.evaluate(() => { try { window.__recorder.start(); } catch {} });
  await page.waitForTimeout(7000);
  await page.getByRole("button", { name: "Cue Starting Grid", exact: true }).first().click({ timeout: 4000 }).catch(() => {});
  await page.waitForTimeout(9000);
  await page.getByRole("button", { name: "Cue Drum Break", exact: true }).first().click({ timeout: 4000 }).catch(() => {});
  await page.waitForTimeout(8000);
  const b64 = await page.evaluate(() => new Promise((res) => {
    const timer = setTimeout(() => res(""), 6000);
    try { window.__recorder.onstop = async () => { clearTimeout(timer); const b = new Blob(window.__chunks, { type: "audio/webm" }); const u = new Uint8Array(await b.arrayBuffer()); let s = ""; for (let i = 0; i < u.length; i++) s += String.fromCharCode(u[i]); res(btoa(s)); }; window.__recorder.stop(); } catch (e) { clearTimeout(timer); res(""); }
  }));
  const v = page.video(); await page.close();
  writeFileSync(`${dir}/${style}.audio.webm`, Buffer.from(b64, "base64"));
  takes.push({ style, video: await v.path(), audio: `${dir}/${style}.audio.webm` });
  console.log("grabado", style);
}
writeFileSync(`${dir}/takes.json`, JSON.stringify(takes, null, 2));
await browser.close(); process.exit(0);
