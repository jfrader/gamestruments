import { expect, test } from "@playwright/test";
import { writeFile } from "node:fs/promises";
import { installAudioCapture } from "./audio-capture.ts";

test("default Extended keeps kick and hat continuity over the entire automatic crossover", async ({ page }, testInfo) => {
  test.setTimeout(600000);
  await installAudioCapture(page, true);
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="extended"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(3);
  await expect(page.locator("#score-title")).toContainText("Extended");
  const bpm = Number(await page.locator("#tempo-value").textContent());
  const secondsPerBar = 240 / bpm;
  const sections = await page.locator("#section-list li").evaluateAll((rows) => rows.map((row) => ({
    id: (row.querySelector("button") as HTMLButtonElement).dataset.cueSection!,
    label: row.querySelector("span")!.textContent!.split(" · ")[0]!,
    bars: Number(row.querySelector("span")!.textContent!.match(/(\d+) bars/)![1]),
  })).filter((section) => section.id !== "outro" && section.id !== "coda"));
  let cursor = 0;
  const windows = sections.map((section) => { const start = cursor; cursor += section.bars; return { ...section, start, end: cursor }; });
  test.setTimeout(Math.ceil((cursor * secondsPerBar + 40) * 1000));
  await page.locator("#center-play").click();
  await expect(page.locator("#mood-name")).toHaveText("Handshake");
  for (let index = 1; index <= windows.length; index++) {
    const boundary = windows[index - 1]!.end * secondsPerBar;
    await page.waitForFunction((boundary) => {
      const capture = window.scanAudio;
      return capture.firstStart !== null && (capture.blocks.at(-1)?.time ?? 0) > capture.firstStart + boundary + 0.15;
    }, boundary, { timeout: Math.ceil((windows[index - 1]!.bars * secondsPerBar + 12) * 1000) });
    await expect(page.locator("#mood-name")).toHaveText(windows[index]?.label ?? "Scan");
  }
  await page.waitForTimeout(1000);
  const capture = await page.evaluate(async () => {
    const state = window.scanAudio;
    return { origin: state.firstStart!, kicks: state.kicks, hats: state.hats, recording: await state.finishRecording!() };
  });
  await page.locator("#start-audio").click();
  const recording = testInfo.outputPath("extended-full-cycle.webm");
  await writeFile(recording, Buffer.from(capture.recording, "base64"));
  await testInfo.attach("extended-full-cycle.webm", { path: recording, contentType: "audio/webm" });
  const hits = {
    kick: capture.kicks.map((hit) => (hit.time - capture.origin) / secondsPerBar),
    hat: capture.hats.map((hit) => (hit.time - capture.origin) / secondsPerBar),
  };
  const expected = { kick: [] as number[], hat: [] as number[] };
  for (const window of windows) {
    if (window.id === "intro" || window.id === "break") continue;
    for (let bar = window.start; bar < window.end; bar++) {
      expected.kick.push(bar, bar + 0.5);
      expected.hat.push(bar + 0.125, bar + 0.375, bar + 0.625, bar + 0.875);
    }
  }
  for (const voice of ["kick", "hat"] as const) {
    const actual = hits[voice].filter((bar) => bar >= windows[0]!.end - 0.001 && bar < cursor - 0.001);
    expect(actual.length, `${voice}: no missing or duplicated hits`).toBe(expected[voice].length);
    actual.forEach((bar, index) => expect(Math.abs(bar - expected[voice][index]!) * secondsPerBar, `${voice} at bar ${bar + 1}`).toBeLessThan(0.02));
  }
  const report = { bpm, windows, expectedKicks: expected.kick.length, expectedHats: expected.hat.length, hits };
  await testInfo.attach("extended-rhythm-audit.json", { body: Buffer.from(JSON.stringify(report, null, 2)), contentType: "application/json" });
  expect(hits.kick.some((bar) => Math.abs(bar - cursor) < 0.001), "kick on the return to Scan").toBe(true);
});
