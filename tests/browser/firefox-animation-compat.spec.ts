import { expect, test } from "@playwright/test";

test("Firefox renders final reactive circle values", async ({ page }) => {
  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

  const orbit = page.locator("#orbit");
  await orbit.evaluate((element) => element.replaceWith(element.cloneNode(true)));
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

test("reduced motion keeps low-motion music feedback", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/#lab");

  const orbit = page.locator("#orbit");
  await orbit.evaluate((element) => element.replaceWith(element.cloneNode(true)));
  const outerRing = orbit.locator(".orbit-ring--outer");
  const innerRing = orbit.locator(".orbit-ring--inner");
  const playhead = orbit.locator(".playhead");
  await orbit.evaluate((element) => {
    element.classList.add("is-running");
    element.style.setProperty("--outer-opacity", "1");
    element.style.setProperty("--inner-opacity", "1");
    element.style.setProperty("--playhead-opacity", "1");
  });

  await expect.poll(async () => outerRing.evaluate((element) =>
    getComputedStyle(element).opacity,
  )).toBe("1");
  await expect.poll(async () => innerRing.evaluate((element) =>
    getComputedStyle(element).opacity,
  )).toBe("1");
  await expect.poll(async () => playhead.evaluate((element) =>
    getComputedStyle(element).opacity,
  )).toBe("1");
  await expect(outerRing).toHaveCSS("transform", "none");
  await expect(innerRing).toHaveCSS("transform", "none");
  await expect(playhead).toHaveCSS("transform", "none");
});
