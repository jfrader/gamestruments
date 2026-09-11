import { expect, test } from "@playwright/test";

async function withIsolatedPage<T>(browser: any, fn: (page: import('@playwright/test').Page) => Promise<T>): Promise<T> {
  const context = await browser.newContext();
  const page = await context.newPage();
  try {
    return await fn(page);
  } finally {
    await context.close();
  }
}

for (const height of [768, 600]) {
  test(`expanded game signals can be wheel-scrolled and used at desktop height ${height}`, async ({ browser }) => {
    await withIsolatedPage(browser, async (page) => {
      await page.setViewportSize({ width: 1366, height });
      await page.goto("/#lab");
      await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
      const panel = page.locator(".race-state");
      await expect(page.locator(".stage-setup #recipe-buttons")).toHaveCount(1);
      await expect(page.locator(".stage-setup #score-buttons")).toHaveCount(1);
      await expect(page.locator(".stage-setup #arrangement-control")).toHaveCount(1);
      expect(await panel.evaluate((element) => getComputedStyle(element).overflowY)).toBe("auto");
      await panel.hover({ position: { x: 40, y: 80 } });
      await page.mouse.wheel(0, 2000);
      await expect.poll(() => panel.evaluate((element) => {
        const summary = element.querySelector("summary")!.getBoundingClientRect();
        const bounds = element.getBoundingClientRect();
        return summary.top >= bounds.top && summary.bottom <= bounds.bottom;
      })).toBe(true);
      await page.locator("#game-signals summary").click();
      const before = await panel.evaluate((element) => element.scrollTop);
      await panel.hover({ position: { x: 40, y: 80 } });
      await page.mouse.wheel(0, 4000);
      await expect.poll(() => panel.evaluate((element) => element.scrollTop)).toBeGreaterThan(before);
      const usable = await panel.evaluate((element) => {
        const bounds = element.getBoundingClientRect();
        const control = element.querySelector(".toggle-control")!.getBoundingClientRect();
        return control.top >= bounds.top && control.bottom <= bounds.bottom;
      });
      expect(usable).toBe(true);
      const knob = await page.locator("#game-signals .toggle-control i").boundingBox();
      expect(knob).not.toBeNull();
      await page.mouse.click(knob!.x + knob!.width / 2, knob!.y + knob!.height / 2);
      await expect(page.locator("#final-lap")).toBeChecked();
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
      expect(await page.evaluate(() => document.documentElement.scrollHeight <= innerHeight)).toBe(true);
    });
  });
}

test("central setup selectors work and do not overlap the player", async ({ browser }) => {
  await withIsolatedPage(browser, async (page) => {
    await page.setViewportSize({ width: 1440, height: 1000 });
    await page.goto("/#lab");
    await page.locator('.stage-setup button[data-recipe="suspense"]').click();
    await page.locator('.stage-setup button[data-experiment-index="2"]').click();
    await expect(page.locator("#score-title")).toContainText("Noir");
    await page.locator('.stage-setup button[data-arrangement="original"]').click();
    await expect(page.locator('#arrangement-buttons button[data-arrangement="original"]')).toHaveAttribute("aria-pressed", "true");
    const bounds = await page.evaluate(() => {
      const rect = (selector: string) => {
        const box = document.querySelector(selector)!.getBoundingClientRect();
        return { top: box.top, bottom: box.bottom };
      };
      return { setup: rect(".stage-setup"), mood: rect(".mood-readout"), orbit: rect(".orbit"), clock: rect(".transport-readout") };
    });
    expect(bounds.setup.bottom).toBeLessThanOrEqual(bounds.mood.top);
    expect(bounds.mood.bottom).toBeLessThanOrEqual(bounds.orbit.top);
    expect(bounds.orbit.bottom).toBeLessThanOrEqual(bounds.clock.top);
  });
});

test("the crossover sidebar can be wheel-scrolled to its final cue", async ({ browser }) => {
  await withIsolatedPage(browser, async (page) => {
    await page.setViewportSize({ width: 1366, height: 600 });
    await page.goto("/#lab");
    await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
    const panel = page.locator(".mix-state");
    await panel.hover({ position: { x: 40, y: 80 } });
    await page.mouse.wheel(0, 4000);
    await expect.poll(() => panel.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);
    const button = page.getByRole("button", { name: "Cue Closed Session", exact: true });
    const bounds = await panel.boundingBox();
    const cue = await button.boundingBox();
    expect(cue!.y).toBeGreaterThanOrEqual(bounds!.y);
    expect(cue!.y + cue!.height).toBeLessThanOrEqual(bounds!.y + bounds!.height);
    await page.mouse.click(cue!.x + cue!.width / 2, cue!.y + cue!.height / 2);
    await expect(page.locator("#mood-name")).toHaveText("Closed Session");
  });
});

test("narrow screens keep setup above the player and use normal page scrolling", async ({ browser }) => {
  await withIsolatedPage(browser, async (page) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/#lab");
    await page.locator('.stage-setup button[data-recipe="suspense"]').click();
    await page.locator("#game-signals summary").click();
    const layout = await page.evaluate(() => {
      const panel = document.querySelector(".race-state")!;
      return { setupBottom: document.querySelector(".stage-setup")!.getBoundingClientRect().bottom,
        playerTop: document.querySelector(".player-surface")!.getBoundingClientRect().top,
        panelHeight: panel.clientHeight, panelScroll: panel.scrollHeight,
        width: document.documentElement.clientWidth, scrollWidth: document.documentElement.scrollWidth };
    });
    expect(layout.setupBottom).toBeLessThanOrEqual(layout.playerTop);
    expect(layout.panelScroll).toBeLessThanOrEqual(layout.panelHeight + 1);
    expect(layout.scrollWidth).toBeLessThanOrEqual(layout.width);
    await page.locator("#game-signals .toggle-control").click();
    await expect(page.locator("#final-lap")).toBeChecked();
  });
});

test("recipe switch keeps .stage-setup and .player-surface top positions identical (regression)", async ({ browser }) => {
  await withIsolatedPage(browser, async (page) => {
    await page.setViewportSize({ width: 1440, height: 1000 });
    await page.goto("/#lab");
    const getTops = () => page.evaluate(() => {
      const s = document.querySelector(".stage-setup")!.getBoundingClientRect().top;
      const p = document.querySelector(".player-surface")!.getBoundingClientRect().top;
      return { s: Math.round(s), p: Math.round(p) };
    });
    await page.locator('#recipe-buttons button[data-recipe="racing"]').click();
    const t1 = await getTops();
    await page.screenshot({ path: "/tmp/opencode/screenshots/game-type-selector-desktop.png" });
    await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
    await getTops();
    await page.screenshot({ path: "/tmp/opencode/screenshots/game-type-selector-desktop.png" });
    await page.locator('#recipe-buttons button[data-recipe="racing"]').click();
    const t3 = await getTops();
    expect(Math.abs(t3.s - t1.s)).toBeLessThanOrEqual(1);
    expect(Math.abs(t3.p - t1.p)).toBeLessThanOrEqual(1);
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/#lab");
    await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
    await page.screenshot({ path: "/tmp/opencode/screenshots/game-type-selector-mobile.png" });
  });
});
