import { expect, test, type Page } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";
import { installAudioCapture } from "./audio-capture.ts";

function watchErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(`pageerror: ${error.message}`));
  page.on("console", (message) => {
    if (message.type() === "error") {
      errors.push(`console: ${message.text()}`);
    }
  });
  return errors;
}

async function expectTourSettled(page: Page): Promise<void> {
  const bpm = Number(await page.locator("#tempo-value").textContent());
  expect(bpm).toBeGreaterThan(0);
  const twoBarCrossfadeMs = 8 * 60000 / bpm;
  await expect(page.locator("#transition-label")).toHaveText("Form playing", {
    timeout: Math.ceil(twoBarCrossfadeMs) + 2000,
  });
}

test("Racing opens in the auto tour and advances Garage → Starting Grid on the authored boundary", async ({ page }) => {
  test.setTimeout(30000);
  const errors = watchErrors(page);

  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");
  await expect(page.locator("#section-control")).toBeVisible();
  await expect(page.locator("#hold-form")).toHaveText("Hold auto tour");
  await expect(page.locator("#mood-name")).toHaveText("Garage");

  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#mood-name")).toHaveText("Garage");

  await expect(page.locator("#mood-name")).toHaveText("Starting Grid", { timeout: 20000 });
  await expectTourSettled(page);

  await page.locator("#hold-form").click();
  await expect(page.locator("#hold-form")).toHaveText("Resume auto tour");
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#transition-label")).toHaveText("Holding section");
  await expect(page.locator("#mood-name")).toHaveText("Starting Grid");
  await expect(page.locator("#section-select")).toBeEnabled();
  await expect(page.locator("#advance-form")).toBeEnabled();

  await page.locator("#hold-form").click();
  await expect(page.locator("#hold-form")).toHaveText("Hold auto tour");
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "false");

  await page.locator("#start-audio").click();
  expect(errors).toEqual([]);
});

test("Adventure opens in the auto tour and advances Trailhead Camp → The Old Forest on the authored boundary", async ({ page }) => {
  test.setTimeout(75000);
  const errors = watchErrors(page);

  await page.goto("/#lab");
  await selectRecipe(page, "adventure");
  await expect(page.locator("#score-title")).toContainText("Folk");
  await expect(page.locator("#section-list li")).toHaveCount(8);
  await expect(page.locator("#section-control")).toBeVisible();
  await expect(page.locator("#hold-form")).toHaveText("Hold auto tour");
  await expect(page.locator("#mood-name")).toHaveText("Trailhead Camp");

  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#mood-name")).toHaveText("Trailhead Camp");

  await expect(page.locator("#mood-name")).toHaveText("The Old Forest", { timeout: 65000 });
  await expectTourSettled(page);

  await page.locator("#hold-form").click();
  await expect(page.locator("#hold-form")).toHaveText("Resume auto tour");
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#transition-label")).toHaveText("Holding section");
  await expect(page.locator("#mood-name")).toHaveText("The Old Forest");
  await expect(page.locator("#section-select")).toBeEnabled();
  await expect(page.locator("#advance-form")).toBeEnabled();

  await page.locator("#hold-form").click();
  await expect(page.locator("#hold-form")).toHaveText("Hold auto tour");
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "false");

  await page.locator("#start-audio").click();
  expect(errors).toEqual([]);
});

const ADVENTURE_STYLES = [
  { label: "Medieval Folk", title: "Folk" },
  { label: "Dark Fantasy", title: "Dark" },
  { label: "Orchestral RPG", title: "Orchestral" },
] as const;

for (const { label, title } of ADVENTURE_STYLES) {
  test(`${label}: Web Audio voices stay audible, finite, and free of heavy clipping`, async ({ page }) => {
    test.setTimeout(40000);
    await installAudioCapture(page);
    const errors = watchErrors(page);

    await page.goto("/#lab");
    await selectRecipe(page, "adventure");
    await expect(page.locator("#section-list li")).toHaveCount(8);
    if (title !== "Folk") {
      await page.locator("#score-buttons button", { hasText: label }).click();
    }
    await expect(page.locator("#score-title")).toContainText(title);

    await page.locator("#center-play").click();
    await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");

    await page.waitForFunction(() => {
      const capture = window.scanAudio;
      const samples = capture.blocks.reduce((sum, block) => sum + block.samples.length, 0);
      return capture.sampleRate > 0 && samples >= capture.sampleRate * 3;
    }, undefined, { timeout: 30000 });

    const stats = await page.evaluate(() => {
      const capture = window.scanAudio;
      let sumSquares = 0;
      let count = 0;
      let peak = 0;
      let clipped = 0;
      let nonFinite = 0;
      for (const block of capture.blocks) {
        for (const sample of block.samples) {
          if (!Number.isFinite(sample)) {
            nonFinite++;
            continue;
          }
          const magnitude = Math.abs(sample);
          if (magnitude > peak) peak = magnitude;
          if (magnitude >= 0.99) clipped++;
          sumSquares += sample * sample;
          count++;
        }
      }
      return { rms: Math.sqrt(sumSquares / count), peak, clipped, nonFinite, count };
    });

    await page.locator("#start-audio").click();

    expect(stats.count).toBeGreaterThan(0);
    expect(stats.nonFinite, "samples must be finite").toBe(0);
    expect(stats.peak, "signal must be audible").toBeGreaterThan(0.01);
    expect(stats.rms, "signal must carry energy").toBeGreaterThan(0.002);
    expect(stats.clipped / stats.count, "no heavy clipping").toBeLessThan(0.001);
    expect(errors).toEqual([]);
  });
}
