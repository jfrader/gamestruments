import { expect, test } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";

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
      await selectRecipe(page, "suspense");
      const panel = page.locator(".race-state");
      await expect(page.locator(".stage-setup #recipe-select")).toHaveCount(1);
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
    await selectRecipe(page, "suspense");
    await page.locator('.stage-setup button[data-experiment-index="2"]').click();
    await expect(page.locator("#score-title")).toContainText("Noir");
    await page.locator('.stage-setup button[data-arrangement="all-phases"]').click();
    await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
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
    await selectRecipe(page, "suspense");
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
    await selectRecipe(page, "suspense");
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
  test(`mobile ${size.width}x${size.height}: game-type selector fits and key controls are reachable`, async ({ browser }) => {
    await withIsolatedPage(browser, async (page) => {
      await page.setViewportSize(size);
      await page.goto("/#lab");
      const trigger = page.locator("#recipe-select-trigger");
      await expect(trigger).toBeVisible();
      const metrics = await page.evaluate(() => {
        const triggerEl = document.querySelector<HTMLElement>("#recipe-select-trigger")!;
        const triggerBox = triggerEl.getBoundingClientRect();
        const copyBox = triggerEl.querySelector(".recipe-select-copy")!.getBoundingClientRect();
        const play = document.querySelector<HTMLElement>("#center-play")!.getBoundingClientRect();
        const signals = document.querySelector<HTMLElement>("#game-signals")!.getBoundingClientRect();
        const stageWidth = document.querySelector<HTMLElement>(".stage-setup")!.getBoundingClientRect().width;
        return {
          fullWidth: triggerBox.width >= stageWidth - 4,
          copyFits: copyBox.bottom <= triggerBox.bottom + 0.5 && copyBox.top >= triggerBox.top - 0.5,
          playTop: play.top,
          signalsTop: signals.top,
          noHorizontalOverflow: document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1,
        };
      });
      expect(metrics.fullWidth).toBe(true);
      expect(metrics.copyFits).toBe(true);
      expect(metrics.playTop).toBeGreaterThan(10);
      expect(metrics.signalsTop).toBeGreaterThanOrEqual(0);
      expect(metrics.noHorizontalOverflow).toBe(true);

      // Opening the list keeps every option on screen.
      await trigger.click();
      const options = page.locator("#recipe-select-menu button[data-recipe]");
      await expect(options).toHaveCount(3);
      for (let index = 0; index < 3; index += 1) {
        await expect(options.nth(index)).toBeVisible();
      }
      await trigger.click();
      await expect(page.locator("#recipe-select-menu")).toBeHidden();
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
    await selectRecipe(page, "suspense");
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
    await selectRecipe(page, "racing");
    const t1 = await getTops();
    await page.screenshot({ path: "/tmp/opencode/screenshots/game-type-selector-desktop.png" });
    await selectRecipe(page, "suspense");
    await getTops();
    await page.screenshot({ path: "/tmp/opencode/screenshots/game-type-selector-desktop.png" });
    await selectRecipe(page, "racing");
    const t3 = await getTops();
    expect(Math.abs(t3.s - t1.s)).toBeLessThanOrEqual(1);
    expect(Math.abs(t3.p - t1.p)).toBeLessThanOrEqual(1);
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/#lab");
    await selectRecipe(page, "suspense");
    await page.screenshot({ path: "/tmp/opencode/screenshots/game-type-selector-mobile.png" });
  });
});

test("game signals panel has exactly one visible legend and phase buttons spaced from helper", async ({ browser }) => {
  await withIsolatedPage(browser, async (page) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto("/#lab");
    const panel = page.locator("#game-signals");
    await expect(panel).toBeVisible();
    // exactly one visible legend/heading (removed redundant "Game phase")
    await expect(panel.locator("legend")).toHaveCount(1);
    await expect(panel.locator("legend")).toHaveText("Game signals");
    // phase buttons have clear gap from the helper text
    const gap = await page.evaluate(() => {
      const help = document.querySelector<HTMLElement>("#game-signals .signal-help")!;
      const phase = document.querySelector<HTMLElement>("#phase-buttons")!;
      return Math.round(phase.getBoundingClientRect().top - help.getBoundingClientRect().bottom);
    });
    expect(gap).toBeGreaterThanOrEqual(8);
    // verify spacing on mobile viewport too
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/#lab");
    const gapMobile = await page.evaluate(() => {
      const help = document.querySelector<HTMLElement>("#game-signals .signal-help")!;
      const phase = document.querySelector<HTMLElement>("#phase-buttons")!;
      return Math.round(phase.getBoundingClientRect().top - help.getBoundingClientRect().bottom);
    });
    expect(gapMobile).toBeGreaterThanOrEqual(8);
  });
});

test("mobile suspense: every setup fieldset and button is a full-width row", async ({ browser }) => {
  await withIsolatedPage(browser, async (page) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/#lab");
    await selectRecipe(page, "suspense");
    await expect(page.locator("#arrangement-control")).toBeVisible();
    const metrics = await page.evaluate(() => {
      const setup = document.querySelector(".stage-setup")!;
      const width = Math.round(setup.getBoundingClientRect().width);
      const fieldsets = Array.from(setup.querySelectorAll(":scope > fieldset")).filter(
        (fieldset) => !(fieldset as HTMLElement).hidden,
      );
      const fieldsetWidths = fieldsets.map((fieldset) => Math.round(fieldset.getBoundingClientRect().width));
      const buttons = Array.from(document.querySelectorAll("#score-buttons button, #arrangement-buttons button"));
      const buttonWidths = buttons.map((button) => Math.round(button.getBoundingClientRect().width));
      return {
        width,
        minFieldset: Math.min(...fieldsetWidths),
        minButton: Math.min(...buttonWidths),
        fieldsetCount: fieldsetWidths.length,
        buttonCount: buttonWidths.length,
        noOverflow: document.documentElement.scrollWidth <= document.documentElement.clientWidth + 1,
      };
    });
    expect(metrics.fieldsetCount).toBeGreaterThanOrEqual(3);
    expect(metrics.buttonCount).toBeGreaterThanOrEqual(5);
    // no fieldset or button may be squeezed into a side-by-side column
    expect(metrics.minFieldset).toBeGreaterThanOrEqual(metrics.width - 4);
    expect(metrics.minButton).toBeGreaterThanOrEqual(metrics.width - 8);
    expect(metrics.noOverflow).toBe(true);
  });
});
