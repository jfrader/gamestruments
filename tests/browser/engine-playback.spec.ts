import { expect, test, type Page } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";
import { captureEngineAudio, waitForEngineSeconds } from "./engine-audio.ts";

async function playEngineSuspense(page: Page, errors: string[]): Promise<void> {
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  await captureEngineAudio(page);
  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");
  await selectRecipe(page, "suspense");
  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
}

test("the Lab plays the engine's own audio and moves the orbit", async ({ page }) => {
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

test("the Lab cues a section on the next bar and cancels the cue", async ({ page }) => {
  const errors: string[] = [];
  await playEngineSuspense(page, errors);
  const cue = page.locator("#section-list button[data-cue-section]", { hasText: "Cue" }).first();
  await cue.click();
  await expect(page.locator("#cue-status")).toContainText("Cued:");
  await page.locator("#cancel-cue").click();
  await expect(page.locator("#cue-status")).not.toContainText("Cued:");
  expect(errors).toEqual([]);
});

test("the Lab keeps playing through a new version", async ({ page }) => {
  const errors: string[] = [];
  await playEngineSuspense(page, errors);
  await page.locator("#new-version").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", /playing|waiting|crossing/);
  const before = await page.evaluate(() => window.engineAudio.blocks);
  await page.waitForTimeout(1000);
  expect(await page.evaluate(() => window.engineAudio.blocks)).toBeGreaterThan(before);
  expect(errors).toEqual([]);
});

test("the Lab auditions the melody, the rhythm and the full mix through the engine", async ({ page }) => {
  const errors: string[] = [];
  await playEngineSuspense(page, errors);
  for (const solo of ["melody", "rhythm", "full"]) {
    const button = page.locator(`#solo-buttons button[data-solo="${solo}"]`);
    await button.click();
    await expect(button).toHaveAttribute("aria-pressed", "true");
  }
  const before = await page.evaluate(() => window.engineAudio.blocks);
  await page.waitForTimeout(500);
  expect(await page.evaluate(() => window.engineAudio.blocks)).toBeGreaterThan(before);
  expect(errors).toEqual([]);
});

test("Folklore plays and cues its chacarera without Adventure game signals", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  await captureEngineAudio(page);
  await page.goto("/#lab");
  await selectRecipe(page, "folklore");
  await expect(page.locator("#score-title")).toContainText("Folklore");
  await expect(page.locator("#game-signals")).toBeHidden();
  await expect(page.locator("#section-list li")).toHaveCount(9);
  for (const section of ["punteo", "respiro", "pena"]) {
    await expect(page.locator(`#section-list button[data-cue-section="${section}"]`)).toBeVisible();
  }
  await page.locator("#center-play").click();
  await waitForEngineSeconds(page, 2);
  const audio = await page.evaluate(() => window.engineAudio);
  expect(audio.peak).toBeGreaterThan(0.001);
  expect(audio.samples.every(Number.isFinite)).toBe(true);
  await page.locator("#hold-form").click();
  await page.locator('#section-list button[data-cue-section="pena"]').click();
  await expect(page.locator("#mood-name")).toHaveText("Peña", { timeout: 20000 });
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await page.locator("#start-audio").click();
  await selectRecipe(page, "adventure");
  await expect(page.locator("#game-signals")).toBeVisible();
  await expect(page.locator("#score-buttons")).toContainText("Medieval Folk");
  expect(errors).toEqual([]);
});

test("Folklore plays Carnavalito through the same style and cue controls", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  await captureEngineAudio(page);
  await page.goto("/#lab");
  await selectRecipe(page, "folklore");
  const carnavalito = page.locator("#score-buttons button").filter({ hasText: "Carnavalito" });
  await carnavalito.click();
  await expect(carnavalito).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#score-title")).toContainText("Carnavalito");
  await expect(page.locator("#game-signals")).toBeHidden();
  await page.locator("#center-play").click();
  await waitForEngineSeconds(page, 2);
  const audio = await page.evaluate(() => window.engineAudio);
  expect(audio.peak).toBeGreaterThan(0.001);
  expect(audio.samples.every(Number.isFinite)).toBe(true);
  await page.locator("#hold-form").click();
  await page.locator('#section-list button[data-cue-section="estribillo"]').click();
  await expect(page.locator("#mood-name")).toHaveText("Estribillo", { timeout: 20000 });
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await page.locator("#start-audio").click();
  await page.locator("#score-buttons button").filter({ hasText: "Chacarera" }).click();
  await expect(page.locator("#section-list li")).toHaveCount(9);
  await expect(page.locator("#game-signals")).toBeHidden();
  expect(errors).toEqual([]);
});
