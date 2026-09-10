import { expect, test } from "@playwright/test";
import { installAudioCapture } from "./audio-capture.ts";

test("base/development pairs are independent and a held starting section survives regeneration", async ({ page }) => {
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  const options = await page.locator("#section-select option").evaluateAll((items) => items.map((item) => ({ id: (item as HTMLOptionElement).value, text: item.textContent })));
  expect(options).toHaveLength(17);
  for (const [base, variation] of [["verse", "scan-ii"], ["chorus", "breach-ii"]]) {
    const index = options.findIndex((item) => item.id === base);
    expect(options[index]!.text).toContain("16 bars");
    expect(options[index + 1]!.id).toBe(variation);
    expect(options[index + 1]!.text).toContain("16 bars");
  }
  await page.locator("#section-select").selectOption("verse");
  await page.locator("#hold-form").click();
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await page.locator("#advance-form").click();
  await expect(page.locator("#cue-status")).toHaveText("Start with Scan II");
  await page.locator("#new-take").click();
  await expect(page.locator("#level-seed")).toHaveValue("level-002");
  await expect(page.locator("#generator-summary")).toContainText("extended-v2-1-1");
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#section-select")).toHaveValue("scan-ii");
});

test("hold prevents the automatic boundary, Next enters Scan II held, and Resume continues naturally", async ({ page }) => {
  test.setTimeout(160000);
  await installAudioCapture(page);
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  await page.locator("#section-select").selectOption("verse");
  const secondsPerBar = 240 / Number(await page.locator("#tempo-value").textContent());
  await page.locator("#center-play").click();
  await page.waitForFunction((seconds) => window.scanAudio.firstStart !== null &&
    (window.scanAudio.blocks.at(-1)?.time ?? 0) > window.scanAudio.firstStart + seconds, 16 * secondsPerBar - 0.2, { timeout: 75000 });
  await page.evaluate(() => document.querySelector<HTMLButtonElement>("#hold-form")!.click());
  await expect(page.locator("#cue-status")).toHaveText("Holding: Scan");
  await page.waitForFunction((seconds) => window.scanAudio.firstStart !== null &&
    (window.scanAudio.blocks.at(-1)?.time ?? 0) > window.scanAudio.firstStart + seconds, 17 * secondsPerBar, { timeout: 10000 });
  await expect(page.locator("#mood-name")).toHaveText("Scan");
  await page.locator("#advance-form").click();
  await expect(page.locator("#mood-name")).toHaveText("Scan II", { timeout: 6000 });
  await expect(page.locator("#cue-status")).toHaveText("Holding: Scan II", { timeout: 9000 });
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await page.locator("#hold-form").click();
  await expect(page.locator("#cue-status")).toHaveText("Automatic progression");
  await expect(page.locator("#mood-name")).toHaveText("Scan II");
  await expect(page.locator("#mood-name")).toHaveText("Approach", { timeout: 65000 });
  await page.locator("#start-audio").click();
});
