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
  test(`game signals are visible and usable at desktop height ${height}`, async ({ browser }) => {
    await withIsolatedPage(browser, async (page) => {
      await page.setViewportSize({ width: 1366, height });
      await page.goto("/#lab");
      await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
      const panel = page.locator(".race-state");
      await expect(page.locator(".stage-setup #recipe-buttons")).toHaveCount(1);
      await expect(page.locator(".stage-setup #score-buttons")).toHaveCount(1);
      await expect(page.locator(".stage-setup #arrangement-control")).toHaveCount(1);
      expect(await panel.evaluate((element) => getComputedStyle(element).overflowY)).toBe("auto");
      // Game signals are prominent and always visible: no expander to open.
      await expect(page.locator("#game-signals")).toBeVisible();
      await expect(page.locator("#game-signals summary")).toHaveCount(0);
      await page.locator("#game-signals .toggle-control").scrollIntoViewIfNeeded();
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
      // Panels scroll internally; wheel over the page must not scroll the document.
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.mouse.move(700, 590);
      await page.mouse.wheel(0, 800);
      await page.waitForTimeout(250);
      expect(await page.evaluate(() => window.scrollY)).toBe(0);
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

const MOBILE_SIZES = [
  { width: 360, height: 640 },
  { width: 390, height: 844 },
  { width: 414, height: 896 },
];

for (const size of MOBILE_SIZES) {
  test(`mobile ${size.width}x${size.height}: recipe buttons do not overlap and key controls are reachable`, async ({ browser }) => {
    await withIsolatedPage(browser, async (page) => {
      await page.setViewportSize(size);
      await page.goto("/#lab");
      await expect(page.locator('#recipe-buttons button[data-recipe="racing"]')).toBeVisible();
      const metrics = await page.evaluate(() => {
        const r1 = document.querySelector<HTMLButtonElement>('#recipe-buttons button[data-recipe="racing"]')!;
        const r2 = document.querySelector<HTMLButtonElement>('#recipe-buttons button[data-recipe="suspense"]')!;
        const r1b = r1.getBoundingClientRect();
        const r2b = r2.getBoundingClientRect();
        const s1b = r1.querySelector("span")!.getBoundingClientRect();
        const s2b = r2.querySelector("span")!.getBoundingClientRect();
        const play = document.querySelector<HTMLElement>("#center-play")!.getBoundingClientRect();
        const signals = document.querySelector<HTMLElement>("#game-signals")!.getBoundingClientRect();
        const horizontalOverlap = !(r1b.right <= r2b.left + 1 || r2b.right <= r1b.left + 1);
        const verticalOverlap = !(r1b.bottom <= r2b.top || r2b.bottom <= r1b.top);
        return {
          anyOverlap: horizontalOverlap && verticalOverlap,
          sideBySide: r2b.left >= r1b.right - 2,
          sameHeight: Math.abs(r1b.height - r2b.height) <= 1,
          aligned: Math.abs(r1b.top - r2b.top) <= 1,
          span1Fits: s1b.bottom <= r1b.bottom + 0.5 && s1b.top >= r1b.top - 0.5,
          span2Fits: s2b.bottom <= r2b.bottom + 0.5 && s2b.top >= r2b.top - 0.5,
          playTop: play.top,
          signalsTop: signals.top,
          noHorizontalOverflow: document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1,
        };
      });
      expect(metrics.anyOverlap).toBe(false);
      expect(metrics.sideBySide).toBe(true);
      expect(metrics.sameHeight).toBe(true);
      expect(metrics.aligned).toBe(true);
      expect(metrics.span1Fits).toBe(true);
      expect(metrics.span2Fits).toBe(true);
      expect(metrics.playTop).toBeGreaterThan(10);
      expect(metrics.signalsTop).toBeGreaterThanOrEqual(0);
      expect(metrics.noHorizontalOverflow).toBe(true);
    });
  });
}

test("phase buttons use a 2x2 grid for Racing and a 3x2 grid for Suspense", async ({ browser }) => {
  await withIsolatedPage(browser, async (page) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/#lab");
    const readGrid = () => page.evaluate(() => {
      const buttons = Array.from(document.querySelectorAll("#phase-buttons button"));
      const first = buttons[0]!.getBoundingClientRect();
      const third = buttons[2]!.getBoundingClientRect();
      return { count: buttons.length, columns: Math.abs(third.top - first.top) > 1 ? 2 : 3 };
    });
    expect(await readGrid()).toEqual({ count: 4, columns: 2 });
    await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
    expect(await readGrid()).toEqual({ count: 6, columns: 3 });
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
