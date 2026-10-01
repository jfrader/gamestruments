import { expect, test } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";
import { waitForBeat } from "./engine-audio.ts";

test("the pool selector exposes every phase and a held form step survives regeneration", async ({ page }) => {
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");
  // The canonical all-phases form loops with a defined next step from every
  // phase, so "advance" and "hold" are deterministic here.
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#score-title")).toContainText("All phases");
  const options = await page.locator("#section-select option").evaluateAll((items) => items.map((item) => ({ id: (item as HTMLOptionElement).value, text: item.textContent })));
  expect(options).toHaveLength(38);
  for (const [base, variation] of [["verse", "scan-ii"], ["chorus", "breach-ii"]]) {
    const index = options.findIndex((item) => item.id === base);
    expect(index).toBeGreaterThanOrEqual(0);
    expect(options[index]!.text).toMatch(/\d+ bars/);
    // The pool builds the variation as its own phase, not a replacement.
    expect(options.some((item) => item.id === variation)).toBe(true);
  }
  await page.locator("#section-select").selectOption("verse");
  await page.locator("#hold-form").click();
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await page.locator("#advance-form").click();
  await expect(page.locator("#section-select")).not.toHaveValue("verse");
  const advanced = await page.locator("#section-select").inputValue();
  expect(advanced).not.toBe("verse");
  await expect(page.locator("#cue-status")).toContainText("Start with");
  await page.locator("#new-piece").click();
  await expect(page.locator("#level-seed")).toHaveValue("level-002");
  await expect(page.locator("#generator-summary")).toContainText("all-phases");
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#section-select")).toHaveValue(advanced);
});

test("hold prevents the automatic boundary, Next enters the next phase held, and Resume continues naturally", async ({ page }) => {
  test.setTimeout(160000);
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
  await page.locator("#section-select").selectOption("verse");
  await page.locator("#center-play").click();
  // Hold on the last beat before the verse's automatic boundary at bar 17.
  await waitForBeat(page, 16, 4, 75000);
  const before = (await page.locator("#mood-name").textContent()) ?? "";
  await page.evaluate(() => document.querySelector<HTMLButtonElement>("#hold-form")!.click());
  await expect(page.locator("#cue-status")).toContainText("Holding:");
  await waitForBeat(page, 17, 2, 20000);
  await expect(page.locator("#mood-name")).toHaveText(before);
  await page.locator("#advance-form").click();
  await expect(page.locator("#mood-name")).not.toHaveText(before, { timeout: 9000 });
  const held = (await page.locator("#mood-name").textContent()) ?? "";
  await expect(page.locator("#cue-status")).toContainText("Holding:", { timeout: 20000 });
  await expect(page.locator("#hold-form")).toHaveAttribute("aria-pressed", "true");
  await page.locator("#hold-form").click();
  await expect(page.locator("#cue-status")).toHaveText("Automatic progression");
  await expect(page.locator("#mood-name")).toHaveText(held);
  await expect(page.locator("#mood-name")).not.toHaveText(held, { timeout: 65000 });
  await page.locator("#start-audio").click();
});
