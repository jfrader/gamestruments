import { expect, test } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";

test("generation controls remain functional before and during playback", async ({ page }) => {
  const runtimeErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning") {
      runtimeErrors.push(`${message.type()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => runtimeErrors.push(`pageerror: ${error.message}`));

  await page.goto("/#lab");

  const volumeSlider = page.locator("#master-volume");
  await expect(volumeSlider).toBeVisible();
  await expect(page.locator("#volume-readout")).toHaveText("100%");

  await volumeSlider.evaluate((input: HTMLInputElement) => {
    input.value = "50";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await expect(page.locator("#volume-readout")).toHaveText("50%");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");
  await expect(page.locator("#section-control")).toBeVisible();
  await expect(page.locator("#hold-form")).toHaveText("Hold auto tour");

  const neon = page.locator("#score-buttons button", { hasText: "Neon" });
  await neon.click();
  await expect(neon).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#audition-status")).toContainText("Generated");

  await page.locator("#new-take").click();
  await expect(page.locator("#variation-value")).toHaveText("level-002");

  await page.locator("#level-seed").fill("release-e2e");
  await page.locator("#apply-seed").click();
  await expect(page.locator("#variation-value")).toHaveText("release-e2e");

  await page.locator("#generation-energy").evaluate((input: HTMLInputElement) => {
    input.value = "0.91";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await expect(page.locator("#generation-energy-value")).toHaveText("91%");
  await expect(page.locator("#audition-status")).toContainText("Regenerated");

  await page.locator("#compare-take").click();
  await expect(page.locator("#compare-take")).toHaveAttribute("aria-pressed", "true");

  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", /Stop engine — playing/);

  const chip = page.locator("#score-buttons button", { hasText: "Chip" });
  await chip.click();
  await expect(chip).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", /Stop engine — playing/);
  await expect(page.locator("#audition-status")).toContainText("Generated");

  await page.locator("#start-audio").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "offline");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", "Start engine");
  expect(runtimeErrors).toEqual([]);
});

test("switching to Suspense generates song-form music instead of racing", async ({
  page,
}) => {
  const runtimeErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      runtimeErrors.push(`${message.type()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => runtimeErrors.push(`pageerror: ${error.message}`));

  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

  await selectRecipe(page, "suspense");
  await expect(page.locator("#audition-status")).toContainText("Opened Suspense");
  await expect(page.locator("#score-title")).toContainText(/Terminal|Cipher|Noir/);
  await expect(page.locator("#section-list li").first()).toContainText("Handshake");
  await expect(page.locator("#runtime-signal")).toContainText("recipe: suspense");

  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", /Stop engine — playing/);
  await expect(page.locator("#transition-label")).toContainText("Form playing");
  expect(runtimeErrors).toEqual([]);
});

test("switching to Adventure generates the eight-section quest arc", async ({ page }) => {
  const runtimeErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      runtimeErrors.push(`${message.type()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => runtimeErrors.push(`pageerror: ${error.message}`));

  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

  await selectRecipe(page, "adventure");
  await expect(page.locator("#audition-status")).toContainText("Opened Adventure");
  await expect(page.locator("#score-title")).toContainText(/Folk|Dark|Orchestral/);
  await expect(page.locator("#section-list li")).toHaveCount(8);
  await expect(page.locator("#section-list li").first()).toContainText("Trailhead Camp");
  await expect(page.locator("#runtime-signal")).toContainText("recipe: adventure");

  await page.getByRole("button", { name: "Combat", exact: true }).click();
  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#mood-name")).toHaveText("Steel and Shadow");

  await page.locator("#start-audio").click();
  expect(runtimeErrors).toEqual([]);
});

test("Extended adds longer beds and Original restores the same seed and score", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");
  await expect(page.locator("#arrangement-control")).toBeVisible();
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await selectRecipe(page, "suspense");
  await expect(page.locator("#score-title")).toContainText("Terminal");
  const original = page.locator('#arrangement-buttons button[data-arrangement="original"]');
  const extended = page.locator('#arrangement-buttons button[data-arrangement="extended"]');
  await expect(extended).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(3);
  await original.click();
  await expect(original).toHaveAttribute("aria-pressed", "true");
  const summary = await page.locator("#generator-summary").textContent();
  const seed = await page.locator("#level-seed").inputValue();
  await extended.click();
  await expect(extended).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#score-title")).toContainText("Extended");
    await expect(page.locator("#section-list li").filter({ has: page.getByRole("button", { name: "Cue Scan", exact: true }) })).toContainText("16 bars");
  await expect(page.locator("#section-list li").first()).toContainText("Handshake · 8 bars");
  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", /Stop engine — playing/);
  await original.click();
  await expect(original).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#audition-status")).toHaveText("Original arrangement restored");
  await expect(page.locator("#generator-summary")).toHaveText(summary!);
  await expect(page.locator("#level-seed")).toHaveValue(seed);
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", /Stop engine — playing/);
  await page.locator("#start-audio").click();
  expect(errors).toEqual([]);
});

test("Anomaly can be auditioned directly and safely rolled back to Original", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");
  await page.getByRole("button", { name: "Cue Anomaly", exact: true }).click();
  await page.locator("#center-play").click();
  await expect(page.locator("#mood-name")).toHaveText("Anomaly");
  await page.locator('#arrangement-buttons button[data-arrangement="original"]').click();
  await expect(page.locator("#audition-status")).toHaveText("Original arrangement restored");
  await expect(page.locator("#mood-name")).toHaveText("Handshake");
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", /Stop engine — playing/);
  await page.locator("#start-audio").click();
  expect(errors).toEqual([]);
});

test("Racing and Suspense keep independent arrangement selections", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");

  // Racing defaults to Extended and offers only Original/Extended.
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await expect(page.locator('#arrangement-buttons button[data-arrangement="extended"]')).toHaveAttribute("aria-pressed", "true");

  // Suspense adds Theme and picks it up without touching Racing's selection.
  await selectRecipe(page, "suspense");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(3);
  await page.locator('#arrangement-buttons button[data-arrangement="theme"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="theme"]')).toHaveAttribute("aria-pressed", "true");

  // Switching to Racing shows Extended again — Theme must not leak over.
  await selectRecipe(page, "racing");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await expect(page.locator('#arrangement-buttons button[data-arrangement="extended"]')).toHaveAttribute("aria-pressed", "true");

  // Back to Suspense: the Theme selection is preserved.
  await selectRecipe(page, "suspense");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(3);
  await expect(page.locator('#arrangement-buttons button[data-arrangement="theme"]')).toHaveAttribute("aria-pressed", "true");
  expect(errors).toEqual([]);
});

test("Adventure hides the arrangement control entirely", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");
  await selectRecipe(page, "adventure");
  await expect(page.locator("#arrangement-control")).toBeHidden();
  expect(errors).toEqual([]);
});

test("Extended to Original while a new phase is active falls back to Garage", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`console: ${message.text()}`);
  });
  await page.goto("/#lab");

  // Racing defaults to Extended; cue and play a phase Original does not have.
  await page.getByRole("button", { name: "Cue Ignition", exact: true }).click();
  await page.locator("#center-play").click();
  await expect(page.locator("#mood-name")).toHaveText("Ignition");

  // Roll back to Original while Ignition is the active section.
  await page.locator('#arrangement-buttons button[data-arrangement="original"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="original"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#audition-status")).toHaveText("Original arrangement restored");
  await expect(page.locator("#mood-name")).toHaveText("Garage");
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await page.locator("#start-audio").click();
  expect(errors).toEqual([]);
});

test("Racing rejects an invalid Theme selection without erroring", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`console: ${message.text()}`);
  });
  await page.goto("/#lab");
  await expect(page.locator('#arrangement-buttons button[data-arrangement="extended"]')).toHaveAttribute("aria-pressed", "true");

  // Simulate a stale/async Theme selection the Racing UI would normally hide.
  await page.evaluate(() => {
    const container = document.querySelector<HTMLElement>("#arrangement-buttons")!;
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.arrangement = "theme";
    container.append(button);
    button.click();
  });

  await expect(page.locator('#arrangement-buttons button[data-arrangement="extended"]')).toHaveAttribute("aria-pressed", "true");
  expect(errors).toEqual([]);
});

test("rapid recipe switches settle on the final recipe without errors", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`console: ${message.text()}`);
  });
  await page.goto("/#lab");

  await selectRecipe(page, "suspense");
  await selectRecipe(page, "adventure");
  await selectRecipe(page, "racing");
  await selectRecipe(page, "suspense");

  await expect(page.locator("#runtime-signal")).toContainText("recipe: suspense");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(3);
  await expect(page.locator('#arrangement-buttons button[data-arrangement="extended"]')).toHaveAttribute("aria-pressed", "true");
  expect(errors).toEqual([]);
});
