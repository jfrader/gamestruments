import { expect, test, type Page } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";
import { captureEngineAudio } from "./engine-audio.ts";

async function playSeededDefault(page: Page, style: string): Promise<void> {
  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");
  await selectRecipe(page, "suspense");
  await expect(page.locator("#score-title")).toContainText("Terminal");
  if (style !== "Terminal") {
    await page.locator("#score-buttons button", { hasText: style }).click();
    await expect(page.locator("#score-title")).toContainText(style);
  }
  // The retired presets are gone; the seeded pool is the Suspense default.
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#transition-label")).toHaveText("Form playing");
}

test("the seeded pool plays the default Suspense engine and swaps to all-phases", async ({ page }) => {
  test.setTimeout(120000);
  await captureEngineAudio(page);
  await playSeededDefault(page, "Terminal");
  await page.waitForFunction(() => window.engineAudio.peak > 0, null, { timeout: 30000 });
  const rendered = await page.evaluate(() => window.engineAudio.blocks);
  await page.locator("#start-audio").click();
  expect(rendered).toBeGreaterThan(0);

  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#score-title")).toContainText("All phases");
  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await page.locator("#start-audio").click();
});

for (const style of ["Terminal", "Cipher", "Noir", "Trance"]) {
  test(`${style}: the seeded default starts and stops without errors`, async ({ page }) => {
    test.setTimeout(90000);
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await captureEngineAudio(page);
    await playSeededDefault(page, style);
    await page.waitForFunction(() => window.engineAudio.peak > 0, null, { timeout: 20000 });
    await page.locator("#start-audio").click();
    expect(errors).toEqual([]);
  });
}
