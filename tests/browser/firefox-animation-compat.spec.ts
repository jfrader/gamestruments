import { expect, test } from "@playwright/test";

test("Firefox renders final reactive circle values", async ({ page }) => {
  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

  const orbit = page.locator("#orbit");
  const outerRing = orbit.locator(".orbit-ring--outer");
  await orbit.evaluate((element) => {
    element.classList.add("is-running");
    element.style.setProperty("--outer-rotation", "0turn");
    element.style.setProperty("--outer-scale", "1");
  });
  const initial = await outerRing.evaluate((element) => getComputedStyle(element).transform);

  await orbit.evaluate((element) => {
    element.style.setProperty("--outer-rotation", "0.25turn");
  });

  await expect.poll(async () => outerRing.evaluate((element) =>
    getComputedStyle(element).transform,
  )).not.toBe(initial);
  await expect.poll(async () => outerRing.evaluate((element) =>
    getComputedStyle(element).transform,
  )).not.toBe("none");
});
