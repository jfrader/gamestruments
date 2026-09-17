import { expect, test } from "@playwright/test";
import { selectRecipe } from "./recipe.ts";
import { installAudioCapture } from "./audio-capture.ts";

declare global { interface Window { cueClock: AudioContext; stoppedBeforeCue: number } }

test("Suspense exposes every music section separately from game signals", async ({ page }) => {
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");
  const sectionIds = await page.locator("#section-select option").evaluateAll((options) => options.map((option) => (option as HTMLOptionElement).value));
  const rowIds = await page.locator("#section-list button").evaluateAll((buttons) => buttons.map((button) => (button as HTMLButtonElement).dataset.cueSection));
  expect(sectionIds).toEqual(rowIds);
  expect(sectionIds).toHaveLength(28);
  for (const id of ["scan-ii", "breach-ii", "pre-chorus", "break", "bridge-b", "solo", "anomaly", "outro", "coda"]) expect(sectionIds).toContain(id);
  await expect(page.locator("#game-signals")).toBeVisible();
  await page.locator("#section-select").selectOption("anomaly");
  await expect(page.locator("#cue-status")).toHaveText("Start with Anomaly");
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "offline");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", "Start engine");
  await page.getByRole("button", { name: "Cue Other Hall", exact: true }).focus();
  await page.keyboard.press("Space");
  await expect(page.locator("#cue-status")).toHaveText("Start with Other Hall");
  await expect(page.locator("#start-audio")).toHaveAttribute("data-engine-state", "offline");
  await expect(page.locator("#start-audio")).toHaveAttribute("aria-label", "Start engine");
  await page.locator("#section-select").selectOption("anomaly");
  // The pool is the only Suspense authority: both arrangements expose every
  // phase, so alias-switching to all-phases keeps Anomaly cueable.
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator("#section-select option")).toHaveCount(28);
  await expect(page.locator('#section-select option[value="anomaly"]')).toHaveCount(1);
  await expect(page.locator("#cue-status")).toHaveText("Start with Anomaly");
});

test("cue buttons wait for a bar and an active blend keeps only the latest queued selection", async ({ page }) => {
  test.setTimeout(45000);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");
  // all-phases keeps every cue target inside the form, so a finished blend
  // resumes automatic progression instead of holding an out-of-form ending.
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
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
  await selectRecipe(page, "suspense");
  await page.locator('#arrangement-buttons button[data-arrangement="all-phases"]').click();
  await expect(page.locator('#arrangement-buttons button[data-arrangement="all-phases"]')).toHaveAttribute("aria-pressed", "true");
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
    (window.scanAudio.blocks.at(-1)?.time ?? 0) > window.scanAudio.firstStart + seconds, secondsPerBar * 10 + 0.2, { timeout: 40000 });
  const kicks = await page.evaluate((secondsPerBar) => window.scanAudio.kicks.map((hit) => (hit.time - window.scanAudio.firstStart!) / secondsPerBar).filter((bar) => bar < 10 - 0.001), secondsPerBar);
  await page.locator("#start-audio").click();
  // The pool's kit is seeded and the arc may expose the opening block without
  // it, so continuity (no long hole) is the contract, not a kick count.
  expect(kicks.length).toBeGreaterThanOrEqual(2);
  for (let index = 1; index < kicks.length; index++) {
    expect(kicks[index]! - kicks[index - 1]!).toBeLessThan(4.5);
  }
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
  await selectRecipe(page, "suspense");
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
  await selectRecipe(page, "suspense");
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
  await selectRecipe(page, "suspense");
  await page.locator("#section-select").selectOption("coda");
  await expect(page.locator("#cue-status")).toHaveText("Start with Closed Session");
  await expect(page.locator("#section-select option")).toHaveCount(28);
  const width = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, page: document.documentElement.scrollWidth }));
  expect(width.page).toBeLessThanOrEqual(width.viewport);
});

test("engine button reflects playing/waiting/crossing states and mobile controls are sticky", async ({ page }) => {
  test.setTimeout(30000);
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");

  // start playback
  await page.locator("#center-play").click();
  const startBtn = page.locator("#start-audio");
  await expect(startBtn).toHaveAttribute("data-engine-state", "playing");
  const moodLabel = (await page.locator("#mood-name").textContent()) || "";
  await expect(startBtn.locator(".engine-label--from")).toHaveText(moodLabel);
  await expect(startBtn).toHaveAttribute("aria-label", new RegExp(`Stop engine — playing ${moodLabel.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`));

  // wait for a bar to pass so cue waits
  await page.waitForFunction(() => Number(document.querySelector("#beat-value")!.textContent) >= 2);
  await page.getByRole("button", { name: "Cue Scan", exact: true }).click();

  // should go to waiting then crossing
  await expect(startBtn).toHaveAttribute("data-engine-state", "waiting", { timeout: 4000 });
  await expect(startBtn).toHaveAttribute("data-engine-state", "crossing", { timeout: 8000 });

  // --cross-progress increases over frames
  let last = -1;
  for (let i = 0; i < 4; i++) {
    await page.waitForTimeout(80);
    const prog = await startBtn.evaluate((el) => parseFloat((el as HTMLElement).style.getPropertyValue("--cross-progress") || "0"));
    expect(prog).toBeGreaterThan(last);
    last = prog;
  }

  // stop for clean
  await startBtn.click();

  // mobile sticky controls
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/#lab"); // reset for clean layout
  const controls = page.locator(".masthead-controls");
  await expect(controls).toBeVisible();
  const before = await controls.boundingBox();
  expect(before && before.y).toBeLessThanOrEqual(12);
  await page.evaluate(() => window.scrollBy(0, 400));
  await page.waitForTimeout(50);
  const after = await controls.boundingBox();
  expect(after && after.y).toBeLessThanOrEqual(12); // still at viewport top
  // shell has top padding so content not hidden
  const shellPad = await page.locator(".console-shell").evaluate((el) => parseInt(getComputedStyle(el).paddingTop, 10));
  expect(shellPad).toBeGreaterThan(40);
});

test("prev/next section buttons cue the neighbouring sections", async ({ page }) => {
  await page.goto("/#lab");
  await selectRecipe(page, "suspense");
  const select = page.locator("#section-select");
  await expect(select).toBeVisible();
  const options = await select.locator("option").evaluateAll((opts) => opts.map((o) => (o as HTMLOptionElement).value));
  expect(options.length).toBeGreaterThan(2);
  const first = await select.inputValue();
  const startIndex = options.indexOf(first);
  expect(startIndex).toBeGreaterThanOrEqual(0);
  const at = (index: number) => options[(index + options.length) % options.length]!;

  await page.locator("#next-section").click();
  await expect(select).toHaveValue(at(startIndex + 1));
  await expect(page.locator("#next-section")).toHaveAttribute("title", /^Next section: /);
  await expect(page.locator("#prev-section")).toHaveAttribute("title", /^Previous section: /);

  await page.locator("#prev-section").click();
  await expect(select).toHaveValue(at(startIndex));

  // wraps from the first section back to the last
  await select.selectOption(at(0));
  await page.locator("#prev-section").click();
  await expect(select).toHaveValue(at(-1));
});
