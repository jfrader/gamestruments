import { expect, test } from "@playwright/test";

test("the orbit advances while music is playing", async ({ page }) => {
  const runtimeErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning") {
      runtimeErrors.push(`${message.type()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => runtimeErrors.push(`pageerror: ${error.message}`));

  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

  const orbit = page.locator("#orbit");
  const outerRing = orbit.locator(".orbit-ring--outer");
  const playhead = orbit.locator(".playhead");
  await page.locator("#center-play").click();
  await expect(orbit).toHaveClass(/is-running/);

  const firstTransform = await playhead.evaluate((element) => getComputedStyle(element).transform);
  const firstRingTransform = await outerRing.evaluate((element) => getComputedStyle(element).transform);

  await page.waitForTimeout(250);

  const secondTransform = await playhead.evaluate((element) => getComputedStyle(element).transform);
  const secondRingTransform = await outerRing.evaluate((element) => getComputedStyle(element).transform);
  expect(firstTransform).not.toBe("none");
  expect(secondTransform).not.toBe(firstTransform);
  expect(firstRingTransform).not.toBe("none");
  expect(secondRingTransform).not.toBe(firstRingTransform);

  await page.locator("#start-audio").click();
  await expect(orbit).not.toHaveClass(/is-running/);
  expect(runtimeErrors).toEqual([]);
});

test("desktop reduced motion does not disable playback animation", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

  const orbit = page.locator("#orbit");
  const outerRing = orbit.locator(".orbit-ring--outer");
  const playhead = orbit.locator(".playhead");
  await page.locator("#center-play").click();
  await expect(orbit).toHaveClass(/is-running/);

  const firstPlayheadTransform = await playhead.evaluate((element) =>
    getComputedStyle(element).transform,
  );
  const firstRingTransform = await outerRing.evaluate((element) =>
    getComputedStyle(element).transform,
  );
  await page.waitForTimeout(250);
  const secondPlayheadTransform = await playhead.evaluate((element) =>
    getComputedStyle(element).transform,
  );
  const secondRingTransform = await outerRing.evaluate((element) =>
    getComputedStyle(element).transform,
  );

  expect(firstPlayheadTransform).not.toBe("none");
  expect(secondPlayheadTransform).not.toBe(firstPlayheadTransform);
  expect(firstRingTransform).not.toBe("none");
  expect(secondRingTransform).not.toBe(firstRingTransform);
  await page.locator("#start-audio").click();
  await expect(orbit).not.toHaveClass(/is-running/);
});
