import { expect, test } from "@playwright/test";
import { mkdirSync } from "node:fs";
import { selectRecipe } from "./recipe.ts";

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
  const playhead = orbit.locator(".playhead");
  const pausedTransform = await playhead.evaluate((element) => getComputedStyle(element).transform);
  await page.waitForTimeout(250);
  expect(await playhead.evaluate((element) => getComputedStyle(element).transform)).toBe(pausedTransform);

  await page.locator("#center-play").click();
  await expect(orbit).toHaveClass(/is-running/);

  const firstTransform = await playhead.evaluate((element) => getComputedStyle(element).transform);

  await page.waitForTimeout(250);

  const secondTransform = await playhead.evaluate((element) => getComputedStyle(element).transform);
  expect(firstTransform).not.toBe("none");
  expect(secondTransform).not.toBe(firstTransform);

  await page.locator("#start-audio").click();
  await expect(orbit).not.toHaveClass(/is-running/);
  const stoppedTransform = await playhead.evaluate((element) => getComputedStyle(element).transform);
  await page.waitForTimeout(250);
  expect(await playhead.evaluate((element) => getComputedStyle(element).transform)).toBe(stoppedTransform);
  expect(runtimeErrors).toEqual([]);
});

test("desktop reduced motion does not disable playback animation", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

  const orbit = page.locator("#orbit");
  const playhead = orbit.locator(".playhead");
  await page.locator("#center-play").click();
  await expect(orbit).toHaveClass(/is-running/);

  const firstPlayheadTransform = await playhead.evaluate((element) =>
    getComputedStyle(element).transform,
  );
  await page.waitForTimeout(250);
  const secondPlayheadTransform = await playhead.evaluate((element) =>
    getComputedStyle(element).transform,
  );

  expect(firstPlayheadTransform).not.toBe("none");
  expect(secondPlayheadTransform).not.toBe(firstPlayheadTransform);
  await page.locator("#start-audio").click();
  await expect(orbit).not.toHaveClass(/is-running/);
});

test("orbit part rings represent instruments and animate independently (racing section)", async ({ page }) => {
  test.setTimeout(60000);
  const runtimeErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning") runtimeErrors.push(`${message.type()}: ${message.text()}`);
  });
  page.on("pageerror", (error) => runtimeErrors.push(`pageerror: ${error.message}`));

  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");

  // Use default racing score which has melody/harmony/bass/kit (+lift in some)
  const orbit = page.locator("#orbit");
  const partRings = orbit.locator(".orbit-part");
  await page.locator("#center-play").click();
  await expect(orbit).toHaveClass(/is-running/);

  mkdirSync("test-results", { recursive: true });
  // capture 4 frames at different moments during playback for visual review
  for (let k = 0; k < 4; k++) {
    await page.waitForTimeout(80 + k * 60);
    await page.screenshot({ path: `test-results/orbit-${Date.now()}.png`, animations: "allow" });
  }

  // wait for at least one part to be shown (non-hidden)
  await expect(partRings.first()).toBeVisible({ timeout: 2000 });

  // assert up to 6 but at least 4 rings visible for the section (some racing sections expose 4+)
  const shown = await partRings.evaluateAll((els) =>
    els.filter((e) => !e.hasAttribute("hidden")).length
  );
  expect(shown).toBeGreaterThanOrEqual(4);

  // each visible has --part-color and --part-opacity set
  for (let i = 0; i < shown; i++) {
    const r = partRings.nth(i);
    if (await r.isHidden()) continue;
    const color = await r.evaluate((el) => getComputedStyle(el).getPropertyValue("--part-color").trim());
    const op = await r.evaluate((el) => getComputedStyle(el).getPropertyValue("--part-opacity").trim());
    expect(color.length).toBeGreaterThan(3);
    expect(parseFloat(op)).toBeGreaterThan(0.1);
  }

  // sample two frames during playback; parts move independently
  const sample = async () => {
    const vals: string[] = [];
    for (let i = 0; i < shown; i++) {
      const r = partRings.nth(i);
      if (await r.isHidden()) continue;
      const rot = await r.evaluate((el) => getComputedStyle(el).getPropertyValue("--part-rotation").trim());
      const sc = await r.evaluate((el) => getComputedStyle(el).getPropertyValue("--part-scale").trim());
      vals.push(`${rot}|${sc}`);
    }
    return vals;
  };
  const s1 = await sample();
  await page.waitForTimeout(180);
  const s2 = await sample();
  // at least some difference across the rings' values between samples (independent animation)
  const anyDiff = s1.some((v, idx) => s2[idx] !== v) || s1.length !== s2.length;
  expect(anyDiff).toBe(true);

  await page.locator("#start-audio").click();
  await expect(orbit).not.toHaveClass(/is-running/);
  expect(runtimeErrors).toEqual([]);
});

test("a stopped Lab leaves the page untouched between frames", async ({ page }) => {
  await page.goto("/#lab");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm");
  // Suspense renders the most section rows, the largest per-frame surface.
  await selectRecipe(page, "suspense");
  await page.waitForTimeout(500);
  const mutations = await page.evaluate(
    () =>
      new Promise<number>((resolve) => {
        let count = 0;
        const observer = new MutationObserver((records) => {
          count += records.length;
        });
        observer.observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true });
        setTimeout(() => {
          observer.disconnect();
          resolve(count);
        }, 1000);
      }),
  );
  expect(mutations).toBe(0);
});
