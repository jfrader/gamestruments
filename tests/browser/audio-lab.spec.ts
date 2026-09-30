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

  await page.locator("#new-piece").click();
  await expect(page.locator("#variation-value")).toHaveText("level-002");

  await page.locator("#level-seed").fill("release-e2e");
  await page.locator("#apply-seed").click();
  await expect(page.locator("#variation-value")).toHaveText("release-e2e");
  await expect(page.locator("#version-value")).toHaveText("release-e2e · Version 1");

  await page.locator("#generation-energy").evaluate((input: HTMLInputElement) => {
    input.value = "0.91";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await expect(page.locator("#generation-energy-value")).toHaveText("91%");
  await expect(page.locator("#audition-status")).toContainText("Regenerated");

  // The version axis is a second performance of the same piece. It starts the
  // engine when idle, so playback is asserted directly here.
  await page.locator("#new-version").click();
  await expect(page.locator("#version-value")).toHaveText("release-e2e · Version 2");
  await expect(page.locator("#audition-status")).toContainText("Version 1");
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
  await expect(page.locator("#section-list li")).toHaveCount(14);
  await expect(page.locator("#section-list li").first()).toContainText("Trailhead Camp");
  await expect(page.locator("#runtime-signal")).toContainText("recipe: adventure");

  await page.locator('#phase-buttons button[data-phase="combat"]').click();
  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#mood-name")).toHaveText("Steel and Shadow");

  await page.locator("#start-audio").click();
  expect(runtimeErrors).toEqual([]);
});

test("switching to Strategy plays the match sections without errors", async ({ page }) => {
  const runtimeErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") runtimeErrors.push(message.text());
  });
  page.on("pageerror", (error) => runtimeErrors.push(error.message));

  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");
  await selectRecipe(page, "strategy");
  await expect(page.locator("#audition-status")).toContainText("Opened Strategy");
  await expect(page.locator("#score-title")).toContainText(/Techno|Trance/);
  await expect(page.locator("#section-list li")).toHaveCount(10);
  await expect(page.locator("#runtime-signal")).toContainText("recipe: strategy");

  await page.locator('#phase-buttons button[data-phase="battle"]').click();
  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#mood-name")).toHaveText("Battle");
  await page.waitForTimeout(1500);
  await page.locator("#start-audio").click();
  expect(runtimeErrors).toEqual([]);
});

test("All phases and Seeded are the only Suspense arrangements and keep the same seed", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");
  await expect(page.locator("#arrangement-control")).toBeVisible();
  await selectRecipe(page, "suspense");
  await expect(page.locator("#score-title")).toContainText("Terminal");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  const seeded = page.locator('#arrangement-buttons button[data-arrangement="seeded"]');
  const allPhases = page.locator('#arrangement-buttons button[data-arrangement="all-phases"]');
  await expect(seeded).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#score-title")).toContainText("Seeded");
  const seed = await page.locator("#level-seed").inputValue();
  await allPhases.click();
  await expect(allPhases).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#score-title")).toContainText("All phases");
  const summary = await page.locator("#generator-summary").textContent();
  await seeded.click();
  await expect(seeded).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#score-title")).toContainText("Seeded");
  await allPhases.click();
  await expect(page.locator("#audition-status")).toHaveText("All phases arrangement ready");
  await expect(page.locator("#generator-summary")).toHaveText(summary!);
  await expect(page.locator("#level-seed")).toHaveValue(seed);
  expect(errors).toEqual([]);
});

test("Anomaly can be auditioned directly and survives an arrangement switch", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");
  await page.getByRole("button", { name: "Cue Anomaly", exact: true }).click();
  await page.locator("#center-play").click();
  await expect(page.locator("#mood-name")).toHaveText("Anomaly");
  // Both pool arrangements carry every phase, so the switch is safe and keeps
  // the current section playing instead of falling back to the opening.
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#audition-status")).toHaveText("All phases arrangement ready");
  await expect(page.locator("#mood-name")).toHaveText("Anomaly");
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "playing");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", /Stop engine — playing/);
  await page.locator("#start-audio").click();
  expect(errors).toEqual([]);
});

test("Racing and Suspense keep independent arrangement selections", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");

  // Racing defaults to Seeded and offers All phases and Seeded.
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");

  // Suspense defaults to Seeded and picks it up without touching Racing.
  await selectRecipe(page, "suspense");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");

  // Switching to Racing shows Seeded again — the pool selection must not leak.
  await selectRecipe(page, "racing");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");

  // Back to Suspense: the all-phases selection is preserved.
  await selectRecipe(page, "suspense");
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
  expect(errors).toEqual([]);
});

test("Racing All phases and Seeded offer the two arrangements and announce them", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`console: ${message.text()}`);
  });
  await page.goto("/#lab");
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");

  // Seeded (the default) composes a song form over the unified eleven-phase
  // pool plus the game-signal sections.
  await expect(page.locator("#score-title")).toContainText("Seeded");
  await expect(page.locator("#section-list li")).toHaveCount(14);
  await expect(page.locator("#section-control")).toBeVisible();

  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#audition-status")).toHaveText("All phases arrangement ready");
  await expect(page.locator("#score-title")).toContainText("All phases");
  await expect(page.locator("#section-list li")).toHaveCount(14);

  await page.locator('#arrangement-buttons button[data-arrangement="seeded"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#audition-status")).toHaveText("Seeded arrangement ready");
  await expect(page.locator("#score-title")).toContainText("Seeded");
  await expect(page.locator("#section-list li")).toHaveCount(14);
  await expect(page.locator("#section-control")).toBeVisible();
  expect(errors).toEqual([]);
});

test("Adventure offers exactly two arrangements: All phases and Seeded", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");
  await selectRecipe(page, "adventure");
  await expect(page.locator("#arrangement-control")).toBeVisible();
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#score-title")).toContainText("Seeded");

  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#audition-status")).toHaveText("All phases arrangement ready");
  await expect(page.locator("#score-title")).toContainText("All phases");
  await expect(page.locator("#section-list li")).toHaveCount(14);
  expect(errors).toEqual([]);
});

test("every recipe exposes exactly the All phases and Seeded arrangements", async ({ page }) => {
  await page.goto("/#lab");
  for (const recipe of ["racing", "suspense", "adventure", "strategy"] as const) {
    await selectRecipe(page, recipe);
    await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
    const ids = await page.locator("#arrangement-buttons button").evaluateAll(
      (buttons) => buttons.map((button) => button.getAttribute("data-arrangement")).sort(),
    );
    expect(ids).toEqual(["all-phases", "seeded"]);
    await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");
  }
});

test("All phases and Seeded share the pool, so a cued phase survives the switch", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`console: ${message.text()}`);
  });
  await page.goto("/#lab");

  // Both arrangements carry every pool phase; cue and play Ignition.
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
  await page.getByRole("button", { name: "Cue Ignition", exact: true }).click();
  await page.locator("#center-play").click();
  await expect(page.locator("#mood-name")).toHaveText("Ignition");

  // Roll back to Seeded while Ignition is the active section: the unified pool
  // keeps it cueable instead of falling back to Garage.
  await page.locator('#arrangement-buttons button[data-arrangement="seeded"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#audition-status")).toHaveText("Seeded arrangement ready");
  await expect(page.locator("#mood-name")).toHaveText("Ignition");
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
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");

  // Simulate a stale/async Theme selection the Racing UI would normally hide.
  await page.evaluate(() => {
    const container = document.querySelector<HTMLElement>("#arrangement-buttons")!;
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.arrangement = "theme";
    container.append(button);
    button.click();
  });

  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");
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
  await expect(page.locator("#arrangement-buttons button")).toHaveCount(2);
  await expect(page.locator('#arrangement-buttons button[data-arrangement="seeded"]')).toHaveAttribute("aria-pressed", "true");
  expect(errors).toEqual([]);
});
