import { expect, test } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";

test("score debugger renders grids for suspense", async ({ page }) => {
  await page.goto("/#lab");
  
  // Select Suspense recipe
  await selectRecipe(page, "suspense");
  
  // Navigate to Debugger tab
  await page.locator('a[data-view-link="debugger"]').click();
  await expect(page.locator('.debugger-view')).toBeVisible();
  
  // Wait for phase dropdown to populate
  const phaseSelect = page.locator('#debugger-phase-select');
  await expect(phaseSelect.locator('option')).not.toHaveCount(0);
  
  // The phase should automatically trigger rendering, but let's re-select just in case
  const firstOption = await phaseSelect.locator('option').first().getAttribute('value');
  if (firstOption) {
    await phaseSelect.selectOption(firstOption);
  }
  
  // Check for the first bar's grid
  const grid = page.locator('.debugger-grid').first();
  await expect(grid).toBeVisible();
  
  // Assert we have at least one voice row
  const firstVoiceCell = grid.locator('tbody tr').first().locator('td').first();
  await expect(firstVoiceCell).not.toBeEmpty();
  
  // Assert there's a solo and mute button
  const controlsCell = grid.locator('tbody tr').first().locator('td').nth(1);
  await expect(controlsCell.locator('button', { hasText: 'Solo' })).toBeVisible();
  await expect(controlsCell.locator('button', { hasText: 'Mute' })).toBeVisible();
  
  // Take a screenshot of the debugger
  await page.screenshot({ path: '/tmp/opencode/guri-908-debugger.png' });
});
