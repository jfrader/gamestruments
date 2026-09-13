import { expect, test, type Page, type TestInfo } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";
import { writeFile } from "node:fs/promises";
import type { SuspenseArrangement } from "../../apps/demo/src/wasm-engine.ts";

import { installAudioCapture } from "./audio-capture.ts";

async function verifyFormDownbeat(style: string, clickScan: boolean, page: Page, testInfo: TestInfo, arrangement: SuspenseArrangement = "original"): Promise<void> {
  test.setTimeout(arrangement !== "original" && clickScan ? 120000 : 60000);
  await installAudioCapture(page);

  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");
  await selectRecipe(page, "suspense");
  await expect(page.locator("#score-title")).toContainText("Terminal");
  if (style !== "Terminal") {
    await page.locator("#score-buttons button", { hasText: style }).click();
    await expect(page.locator("#score-title")).toContainText(style);
  }
  if (arrangement === "original") {
    await page.locator('#arrangement-buttons button[data-arrangement="original"]').click();
  }
  await expect(page.locator(`#arrangement-buttons button[data-arrangement="${arrangement}"]`)).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator('#phase-buttons button[data-phase="scan"]')).toHaveAttribute("aria-pressed", "true");
  if (clickScan) {
    await page.locator('#section-select').selectOption("verse");
  }
  const bpm = Number(await page.locator("#tempo-value").textContent());
  const sectionBarsByArrangement: Record<SuspenseArrangement, number> = {
    original: 8,
    extended: 16,
    theme: 8,
  };
  const sectionBars = clickScan ? sectionBarsByArrangement[arrangement] : 8;
  const secondsToBoundary = sectionBars * 4 * 60 / bpm;
  await page.locator("#center-play").click();
  await expect(page.locator("#mood-name")).toHaveText(clickScan ? "Scan" : "Handshake");
  await expect(page.locator("#transition-label")).toHaveText("Form playing");

  if (arrangement !== "original" && clickScan) {
    await page.waitForFunction((halfwaySeconds) => {
      const capture = window.scanAudio;
      return capture.firstStart !== null &&
        (capture.blocks.at(-1)?.time ?? 0) > capture.firstStart + halfwaySeconds;
    }, 32 * 60 / bpm + 0.5, { timeout: 45000 });
    await expect(page.locator("#mood-name")).toHaveText("Scan");
    await expect(page.locator("#transition-label")).toHaveText("Form playing");
  }

  await page.waitForFunction((boundarySeconds) => {
    const capture = window.scanAudio;
    return capture.firstStart !== null &&
      (capture.blocks.at(-1)?.time ?? 0) >= capture.firstStart + boundarySeconds + 1.5;
  }, secondsToBoundary, { timeout: 105000 });
  await expect(page.locator("#mood-name")).toHaveText(clickScan ? arrangement === "extended" ? "Scan II" : "Approach" : "Scan");
  await expect(page.locator("#bar-value")).toHaveText(String(sectionBars + 1).padStart(2, "0"));

  const result = await page.evaluate((boundarySeconds) => {
    const capture = window.scanAudio;
    const boundary = (capture.firstStart ?? 0) + boundarySeconds;
    const blocks = capture.blocks.filter((block) => block.time >= boundary - 1.5 && block.time <= boundary + 1.5);
    return {
      kicks: capture.kicks.filter((kick) => Math.abs(kick.time - boundary) < 0.02),
      rms: blocks.map((block) => ({
        offset: block.time - boundary,
        value: Math.sqrt(block.samples.reduce((sum, value) => sum + value * value, 0) / block.samples.length),
      })),
      samples: blocks.flatMap((block) => block.samples),
      sampleRate: capture.sampleRate,
    };
  }, secondsToBoundary);
  await page.locator("#start-audio").click();

  const wav = Buffer.alloc(44 + result.samples.length * 2);
  wav.write("RIFF");
  wav.writeUInt32LE(wav.length - 8, 4);
  wav.write("WAVEfmt ", 8);
  wav.writeUInt32LE(16, 16);
  wav.writeUInt16LE(1, 20);
  wav.writeUInt16LE(1, 22);
  wav.writeUInt32LE(result.sampleRate, 24);
  wav.writeUInt32LE(result.sampleRate * 2, 28);
  wav.writeUInt16LE(2, 32);
  wav.writeUInt16LE(16, 34);
  wav.write("data", 36);
  wav.writeUInt32LE(result.samples.length * 2, 40);
  result.samples.forEach((sample, index) => {
    wav.writeInt16LE(Math.round(Math.max(-1, Math.min(1, sample)) * 32767), 44 + index * 2);
  });
  const filename = clickScan ? "scan-to-approach.wav" : "handshake-to-scan.wav";
  const audioPath = testInfo.outputPath(filename);
  await writeFile(audioPath, wav);
  await testInfo.attach(filename, { path: audioPath, contentType: "audio/wav" });

  expect(result.kicks).toHaveLength(1);
  expect(result.kicks[0]!.time - result.kicks[0]!.scheduledAt).toBeGreaterThan(0.05);
  const before = Math.max(...result.rms.filter((block) => block.offset >= -0.15 && block.offset < 0).map((block) => block.value));
  const attack = Math.max(...result.rms.filter((block) => block.offset >= 0 && block.offset < 0.25).map((block) => block.value));
  expect(attack).toBeGreaterThan(0.02);
  expect(attack).toBeGreaterThan(before * 2);
}

test("Extended keeps the approved default Handshake-to-Scan downbeat", async ({ page }, testInfo) => {
  await verifyFormDownbeat("Terminal", false, page, testInfo, "extended");
});

test("Extended Scan enters Scan II on its sixteen-bar downbeat", async ({ page }, testInfo) => {
  await verifyFormDownbeat("Terminal", true, page, testInfo, "extended");
});

for (const style of ["Terminal", "Cipher", "Noir"]) {
  test(`${style}: Scan lands an audible kick on bar 9 beat 1 using the real audio clock`, async ({ page }, testInfo) => {
    await verifyFormDownbeat(style, true, page, testInfo);
  });
  test(`${style}: default Extended Play lands a kick after Handshake without clicking a phase`, async ({ page }, testInfo) => {
    await verifyFormDownbeat(style, false, page, testInfo, "extended");
  });
}

test("default Extended keeps the kick and hat going for eight bars after Handshake", async ({ page }) => {
  test.setTimeout(90000);
  await installAudioCapture(page);
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");
  await expect(page.locator('#arrangement-buttons button[data-arrangement="extended"]')).toHaveAttribute("aria-pressed", "true");
  const secondsPerBar = 240 / Number(await page.locator("#tempo-value").textContent());
  await page.locator("#center-play").click();
  await expect(page.locator("#mood-name")).toHaveText("Handshake");
  await page.waitForFunction((seconds) => window.scanAudio.firstStart !== null &&
    (window.scanAudio.blocks.at(-1)?.time ?? 0) > window.scanAudio.firstStart + seconds, 16 * secondsPerBar + 0.2, { timeout: 75000 });
  await expect(page.locator("#mood-name")).toHaveText("Scan");
  const hits = await page.evaluate((secondsPerBar) => {
    const capture = window.scanAudio;
    const relative = (hit: {time: number}) => (hit.time - capture.firstStart!) / secondsPerBar;
    return { kicks: capture.kicks.map(relative).filter((bar) => bar >= 8 - 0.001 && bar < 16 - 0.001),
      hats: capture.hats.map(relative).filter((bar) => bar >= 8 - 0.001 && bar < 16 - 0.001) };
  }, secondsPerBar);
  await page.locator("#start-audio").click();
  expect(hits.kicks).toHaveLength(16);
  expect(hits.hats).toHaveLength(32);
  hits.kicks.forEach((bar, index) => expect(Math.abs(bar - (8 + index * 0.5)) * secondsPerBar).toBeLessThan(0.02));
  hits.hats.forEach((bar, index) => expect(Math.abs(bar - (8.125 + index * 0.25)) * secondsPerBar).toBeLessThan(0.02));
});
