import { expect, test } from "@playwright/test";

test("generation controls remain functional before and during playback", async ({ page }) => {
  const runtimeErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning") {
      runtimeErrors.push(`${message.type()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => runtimeErrors.push(`pageerror: ${error.message}`));

  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

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
  await expect(page.locator("#start-audio")).toContainText("Stop engine");

  const chip = page.locator("#score-buttons button", { hasText: "Chip" });
  await chip.click();
  await expect(chip).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("#start-audio")).toContainText("Stop engine");
  await expect(page.locator("#audition-status")).toContainText("Generated");

  await page.locator("#start-audio").click();
  await expect(page.locator("#start-audio")).toContainText("Start engine");
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

  await page.locator("#recipe-buttons button", { hasText: "Suspense" }).click();
  await expect(page.locator("#audition-status")).toContainText("Opened Suspense");
  await expect(page.locator("#score-title")).toContainText(/Terminal|Cipher|Noir/);
  await expect(page.locator("#section-list li").first()).toContainText("Handshake");
  await expect(page.locator("#runtime-signal")).toContainText("recipe: suspense");

  await page.locator("#center-play").click();
  await expect(page.locator("#start-audio")).toContainText("Stop engine");
  await expect(page.locator("#transition-label")).toContainText("Form playing");
  expect(runtimeErrors).toEqual([]);
});
