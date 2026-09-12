import { expect, test } from "@playwright/test";

test("Firefox renders final reactive circle values", async ({ page }) => {
  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

  const orbit = page.locator("#orbit");
  await orbit.evaluate((element) => element.replaceWith(element.cloneNode(true)));
  const circle = orbit.locator(".playhead");
  await orbit.evaluate((element) => {
    element.classList.add("is-running");
    element.style.setProperty("--playhead-rotation", "0turn");
  });
  const initial = await circle.evaluate((element) => getComputedStyle(element).transform);

  await orbit.evaluate((element) => {
    element.style.setProperty("--playhead-rotation", "0.25turn");
  });

  await expect.poll(async () => circle.evaluate((element) =>
    getComputedStyle(element).transform,
  )).not.toBe(initial);
  await expect.poll(async () => circle.evaluate((element) =>
    getComputedStyle(element).transform,
  )).not.toBe("none");
});

test("Firefox keeps circle motion with reduced motion enabled", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/#lab");

  const orbit = page.locator("#orbit");
  await orbit.evaluate((element) => element.replaceWith(element.cloneNode(true)));
  const circle = orbit.locator(".playhead");
  await orbit.evaluate((element) => {
    element.classList.add("is-running");
    element.style.setProperty("--playhead-rotation", "0turn");
  });
  const initial = await circle.evaluate((element) => getComputedStyle(element).transform);

  await orbit.evaluate((element) => {
    element.style.setProperty("--playhead-rotation", "0.25turn");
  });

  await expect.poll(async () => circle.evaluate((element) =>
    getComputedStyle(element).transform,
  )).not.toBe(initial);
  await expect.poll(async () => circle.evaluate((element) =>
    getComputedStyle(element).transform,
  )).not.toBe("none");
});
