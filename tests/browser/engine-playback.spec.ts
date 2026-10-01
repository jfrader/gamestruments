import { expect, test, type Page } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";

interface EngineCapture {
  blocks: number;
  peak: number;
}

declare global { interface Window { engineAudio: EngineCapture } }

/** Record the audio blocks the engine sends to its worklet. */
async function captureEngineAudio(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const capture: EngineCapture = { blocks: 0, peak: 0 };
    window.engineAudio = capture;
    const post = MessagePort.prototype.postMessage;
    MessagePort.prototype.postMessage = function (this: MessagePort, message: unknown, ...rest: unknown[]) {
      if (message instanceof Float32Array) {
        capture.blocks += 1;
        for (const sample of message) capture.peak = Math.max(capture.peak, Math.abs(sample));
      }
      return (post as (...args: unknown[]) => void).call(this, message, ...rest);
    } as typeof MessagePort.prototype.postMessage;
  });
}

async function playEngineSuspense(page: Page, errors: string[]): Promise<void> {
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  await captureEngineAudio(page);
  await page.goto("/?engine#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");
  await selectRecipe(page, "suspense");
  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
}

test("?engine plays the engine's own audio and moves the orbit", async ({ page }) => {
  const errors: string[] = [];
  await playEngineSuspense(page, errors);
  await page.waitForTimeout(1500);
  const audio = await page.evaluate(() => window.engineAudio);
  expect(audio.blocks).toBeGreaterThan(10);
  expect(audio.peak).toBeGreaterThan(0.001);

  const playhead = page.locator("#orbit .playhead");
  const first = await playhead.evaluate((element) => getComputedStyle(element).transform);
  await page.waitForTimeout(300);
  expect(await playhead.evaluate((element) => getComputedStyle(element).transform)).not.toBe(first);
  expect(errors).toEqual([]);
});

test("?engine cues a section on the next bar and cancels the cue", async ({ page }) => {
  const errors: string[] = [];
  await playEngineSuspense(page, errors);
  const cue = page.locator("#section-list button[data-cue-section]", { hasText: "Cue" }).first();
  await cue.click();
  await expect(page.locator("#cue-status")).toContainText("Cued:");
  await page.locator("#cancel-cue").click();
  await expect(page.locator("#cue-status")).not.toContainText("Cued:");
  expect(errors).toEqual([]);
});

test("?engine keeps playing through a new version", async ({ page }) => {
  const errors: string[] = [];
  await playEngineSuspense(page, errors);
  await page.locator("#new-version").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", /playing|waiting|crossing/);
  const before = await page.evaluate(() => window.engineAudio.blocks);
  await page.waitForTimeout(1000);
  expect(await page.evaluate(() => window.engineAudio.blocks)).toBeGreaterThan(before);
  expect(errors).toEqual([]);
});
