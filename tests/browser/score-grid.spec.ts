import { expect, test } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";

test("score debugger renders the engine's step grid, counts and solo state", async ({ page }) => {
  await page.goto("/?debug=1#lab");
  await selectRecipe(page, "suspense");

  await page.locator('a[data-view-link="debugger"]').click();
  await expect(page.locator(".debugger-view")).toBeVisible();

  const phaseSelect = page.locator("#debugger-phase-select");
  await expect(phaseSelect.locator("option")).not.toHaveCount(0);
  await phaseSelect.selectOption("chorus");

  const grids = page.locator(".debugger-grid");
  const bars = await grids.count();
  expect(bars).toBeGreaterThan(0);

  // The header carries the bar's subdivision, and it is one of the grids the
  // debugger claims to support.
  const headerCells = await grids.first().locator("thead th").count();
  expect([8, 16]).toContain(headerCells - 5);

  // For every voice row that actually hits in this bar, the summary count must
  // equal the number of cells the grid marks, and the velocity column must be
  // populated. This is the part that would pass vacuously before.
  const rows = grids.first().locator("tbody tr");
  const rowCount = await rows.count();
  let rowsWithHits = 0;
  for (let index = 0; index < rowCount; index += 1) {
    const row = rows.nth(index);
    const marked = await row.locator("td.debugger-cell-hit").count();
    if (marked === 0) {
      continue;
    }
    const cells = row.locator("td");
    const cellCount = await cells.count();
    await expect(cells.nth(cellCount - 3)).toHaveText(String(marked));
    await expect(cells.nth(cellCount - 1)).not.toHaveText("-");
    rowsWithHits += 1;
  }
  expect(rowsWithHits).toBeGreaterThan(0);

  await page.screenshot({ path: "test-results/score-debugger.png" });

  // Per-voice solo and mute are wired to the live solo state.
  const firstHitRow = grids.first().locator("tbody tr").filter({
    has: page.locator("td.debugger-cell-hit"),
  }).first();
  const soloButton = firstHitRow.locator("button", { hasText: "Solo" });
  const muteButton = firstHitRow.locator("button", { hasText: "Mute" });
  await soloButton.click();
  await expect(soloButton).toHaveAttribute("aria-pressed", "true");
  await muteButton.click();
  await expect(muteButton).toHaveAttribute("aria-pressed", "true");
  await expect(soloButton).toHaveAttribute("aria-pressed", "false");
});

test("a played bar never becomes a phase in the Lab", async ({ page }) => {
  await page.goto("/?debug=1#lab");
  await selectRecipe(page, "suspense");

  await page.locator('a[data-view-link="debugger"]').click();
  await page.locator("#debugger-phase-select").selectOption("chorus");
  await page.locator(".debugger-bar-container").first().locator("button", { hasText: "Play bar" }).click();

  // The one-bar slice exists only so the transport can play it. It must never
  // leak into the Lab's phase lists or the section stepper.
  await page.locator('a[data-view-link="lab"]').click();
  await expect(page.locator("#section-select option", { hasText: "(Bar" })).toHaveCount(0);
  await expect(page.locator("#section-list li", { hasText: "(Bar" })).toHaveCount(0);
  await page.locator('a[data-view-link="debugger"]').click();
  await expect(page.locator("#debugger-phase-select option", { hasText: "-bar-" })).toHaveCount(0);
});
