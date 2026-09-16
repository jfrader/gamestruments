import { expect, test } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";
import { writeFile } from "node:fs/promises";
import { installAudioCapture } from "./audio-capture.ts";

test("the seeded pool keeps the automatic crossover moving phase to phase", async ({ page }, testInfo) => {
  test.setTimeout(240000);
  await installAudioCapture(page, true);
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await expect(page.locator("#score-title")).toContainText("Seeded");
  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#transition-label")).toHaveText("Form playing");

  // The composed form is the whole show: the mood leaves its opening phase on
  // the automatic boundary (no cue, no manual section pick).
  const first = (await page.locator("#mood-name").textContent()) ?? "";
  await page.waitForFunction(
    (first) => document.querySelector("#mood-name")!.textContent !== first,
    first,
    { timeout: 120000 },
  );
  const second = (await page.locator("#mood-name").textContent()) ?? "";
  expect(second).not.toBe("");
  expect(second).not.toBe(first);

  await page.waitForTimeout(1500);
  const capture = await page.evaluate(async () => {
    const state = window.scanAudio;
    return { recording: await state.finishRecording!() };
  });
  await page.locator("#start-audio").click();
  const recording = testInfo.outputPath("seeded-crossover.webm");
  await writeFile(recording, Buffer.from(capture.recording, "base64"));
  await testInfo.attach("seeded-crossover.webm", { path: recording, contentType: "audio/webm" });
});
