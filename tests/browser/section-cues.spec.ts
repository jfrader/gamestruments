import { expect, test } from "@playwright/test";
import { installAudioCapture } from "./audio-capture.ts";

declare global { interface Window { cueClock: AudioContext; stoppedBeforeCue: number } }

test("Suspense exposes every music section separately from game signals", async ({ page }) => {
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  const sectionIds = await page.locator("#section-select option").evaluateAll((options) => options.map((option) => (option as HTMLOptionElement).value));
  const rowIds = await page.locator("#section-list button").evaluateAll((buttons) => buttons.map((button) => (button as HTMLButtonElement).dataset.cueSection));
  expect(sectionIds).toEqual(rowIds);
  expect(sectionIds).toHaveLength(17);
  for (const id of ["scan-ii", "breach-ii", "pre-chorus", "break", "bridge-b", "solo", "anomaly", "outro", "coda"]) expect(sectionIds).toContain(id);
  await expect(page.locator("#game-signals")).not.toHaveAttribute("open");
  await page.locator("#section-select").selectOption("anomaly");
  await expect(page.locator("#cue-status")).toHaveText("Start with Anomaly");
  await expect(page.locator("#start-audio")).toContainText("Start engine");
  await page.getByRole("button", { name: "Cue Other Hall", exact: true }).focus();
  await page.keyboard.press("Space");
  await expect(page.locator("#cue-status")).toHaveText("Start with Other Hall");
  await expect(page.locator("#start-audio")).toContainText("Start engine");
  await page.locator("#section-select").selectOption("anomaly");
  await page.locator('#arrangement-buttons button[data-arrangement="original"]').click();
  await expect(page.locator("#section-select option")).toHaveCount(14);
  await expect(page.locator('#section-select option[value="anomaly"]')).toHaveCount(0);
  await expect(page.locator("#cue-status")).toHaveText("Start with Handshake");
});

test("cue buttons wait for a bar and an active blend keeps only the latest queued selection", async ({ page }) => {
  test.setTimeout(45000);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  await page.locator("#center-play").click();
  await page.waitForFunction(() => Number(document.querySelector("#beat-value")!.textContent) >= 2);
  await page.getByRole("button", { name: "Cue Anomaly", exact: true }).click();
  await expect(page.locator("#cue-status")).toHaveText("Cued: Anomaly");
  await expect(page.locator("#mood-name")).toHaveText("Handshake");
  await expect(page.locator("#cue-detail")).toContainText("Starts on bar 2");
  await expect(page.locator("#cancel-cue")).toBeEnabled();
  await expect(page.locator("#mood-name")).toHaveText("Anomaly", { timeout: 6000 });
  await expect(page.locator("#cue-status")).toContainText("Blending:");
  await page.locator("#section-select").selectOption("solo");
  await expect(page.locator("#cue-status")).toHaveText("Queued: Decrypt");
  await page.locator("#section-select").selectOption("bridge-b");
  await expect(page.locator("#cue-status")).toHaveText("Queued: Other Hall");
  await expect(page.locator("#mood-name")).toHaveText("Anomaly");
  await page.locator("#cancel-cue").click();
  await expect(page.locator("#cue-status")).toContainText("Blending:");
  await expect(page.locator("#cancel-cue")).toBeDisabled();
  await expect(page.locator("#cue-status")).toHaveText("Automatic progression", { timeout: 9000 });
  await expect(page.locator("#mood-name")).toHaveText("Anomaly");
  await page.locator("#start-audio").click();
  expect(errors).toEqual([]);
});

test("cancelled cues do not stop the current rhythm when their old fade timer expires", async ({ page }) => {
  test.setTimeout(35000);
  await installAudioCapture(page);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  await page.locator("#section-select").selectOption("verse");
  const secondsPerBar = 240 / Number(await page.locator("#tempo-value").textContent());
  await page.locator("#center-play").click();
  await page.waitForFunction(() => Number(document.querySelector("#beat-value")!.textContent) >= 2);
  await page.getByRole("button", { name: "Cue Anomaly", exact: true }).click();
  await page.getByRole("button", { name: "Cue Decrypt", exact: true }).click();
  await expect(page.locator("#cue-status")).toHaveText("Cued: Decrypt");
  await page.locator("#cancel-cue").click();
  await expect(page.locator("#cue-status")).toHaveText("Automatic progression");
  await expect(page.locator("#mood-name")).toHaveText("Scan");
  await page.waitForFunction((seconds) => window.scanAudio.firstStart !== null &&
    (window.scanAudio.blocks.at(-1)?.time ?? 0) > window.scanAudio.firstStart + seconds, secondsPerBar * 5 + 0.2, { timeout: 25000 });
  const kicks = await page.evaluate((secondsPerBar) => window.scanAudio.kicks.map((hit) => (hit.time - window.scanAudio.firstStart!) / secondsPerBar).filter((bar) => bar < 5 - 0.001), secondsPerBar);
  await page.locator("#start-audio").click();
  expect(kicks).toHaveLength(10);
  kicks.forEach((bar, index) => expect(Math.abs(bar - index * 0.5) * secondsPerBar).toBeLessThan(0.02));
  expect(errors).toEqual([]);
});

test("cancelling a looked-ahead cue stops its future voices", async ({ page }) => {
  await installAudioCapture(page);
  await page.addInitScript(() => {
    window.stoppedBeforeCue = 0;
    const Native = window.AudioContext;
    window.AudioContext = class extends Native {
      constructor(options?: AudioContextOptions) {
        super(options); window.cueClock = this;
        const create = this.createOscillator.bind(this);
        this.createOscillator = () => {
          const node = create(); let startAt: number | null = null;
          const start = node.start.bind(node); const stop = node.stop.bind(node);
          node.start = (time = 0) => { startAt = time; start(time); };
          node.stop = (time = 0) => {
            if (startAt !== null && this.currentTime < startAt && time <= startAt) window.stoppedBeforeCue++;
            stop(time);
          };
          return node;
        };
      }
    };
  });
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  await page.locator("#section-select").selectOption("verse");
  const secondsPerBar = 240 / Number(await page.locator("#tempo-value").textContent());
  await page.locator("#center-play").click();
  await page.waitForFunction((seconds) => window.scanAudio.firstStart !== null && window.cueClock.currentTime > window.scanAudio.firstStart + seconds, secondsPerBar - 0.24);
  await page.evaluate(async () => {
    const select = document.querySelector<HTMLSelectElement>("#section-select")!;
    select.value = "anomaly";
    select.dispatchEvent(new Event("change", { bubbles: true }));
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    const cancel = document.querySelector<HTMLButtonElement>("#cancel-cue")!;
    if (cancel.disabled) throw new Error("Cue must remain cancellable before its downbeat");
    cancel.click();
  });
  expect(await page.evaluate(() => window.stoppedBeforeCue)).toBeGreaterThan(0);
  await page.waitForFunction((seconds) => window.scanAudio.firstStart !== null && window.cueClock.currentTime > window.scanAudio.firstStart + seconds, secondsPerBar + 0.3);
  await expect(page.locator("#mood-name")).toHaveText("Scan");
  await page.locator("#start-audio").click();
});

test("a cue cannot replace the transport while audio is still starting", async ({ page }) => {
  await page.addInitScript(() => {
    const Native = window.AudioContext;
    window.AudioContext = class extends Native {
      ready = false;
      override get state(): AudioContextState { return this.ready ? super.state : "suspended"; }
      override async resume(): Promise<void> {
        await new Promise((resolve) => setTimeout(resolve, 1200));
        await super.resume();
        this.ready = true;
      }
    };
  });
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  await page.locator("#center-play").click();
  await expect(page.locator("#section-select")).toBeDisabled();
  await expect(page.getByRole("button", { name: "Cue Anomaly", exact: true })).toBeDisabled();
  await page.evaluate(() => {
    const select = document.querySelector<HTMLSelectElement>("#section-select")!;
    select.value = "anomaly";
    select.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await expect(page.locator("#section-select")).toBeEnabled();
  await expect(page.locator("#section-select")).toHaveValue("intro");
  await expect(page.locator("#mood-name")).toHaveText("Handshake");
  await page.locator("#start-audio").click();
});

test("the complete section selector and cue status fit a compact viewport", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  await page.locator("#section-select").selectOption("coda");
  await expect(page.locator("#cue-status")).toHaveText("Start with Closed Session");
  await expect(page.locator("#section-select option")).toHaveCount(17);
  const width = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, page: document.documentElement.scrollWidth }));
  expect(width.page).toBeLessThanOrEqual(width.viewport);
});
