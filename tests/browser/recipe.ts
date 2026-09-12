import { expect, type Page } from "@playwright/test";

/** Open the Game type selector and pick a recipe. */
export async function selectRecipe(page: Page, recipe: string): Promise<void> {
  await page.locator("#recipe-select-trigger").click();
  const option = page.locator(`#recipe-select-menu button[data-recipe="${recipe}"]`);
  await expect(option).toBeVisible();
  await option.click();
  await expect(page.locator("#recipe-select-trigger")).toHaveAttribute("aria-expanded", "false");
}
