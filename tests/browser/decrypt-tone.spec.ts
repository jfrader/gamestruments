import { expect, test } from "@playwright/test";
import { writeFile } from "node:fs/promises";
import { installAudioCapture } from "./audio-capture.ts";

declare global { interface Window { decryptToneStarts: number[] } }

test("Decrypt plays its backing without the confirmed first-beat glass oscillator", async ({ page }, testInfo) => {
  await installAudioCapture(page, true);
  await page.addInitScript(() => {
    window.decryptToneStarts = [];
    const Native = window.AudioContext;
    window.AudioContext = class extends Native {
      constructor(options?: AudioContextOptions) {
        super(options);
        const create = this.createOscillator.bind(this);
        this.createOscillator = () => {
          const node = create();
          const values: { value: number; time: number }[] = [];
          const set = node.frequency.setValueAtTime.bind(node.frequency);
          node.frequency.setValueAtTime = (value, time) => { values.push({ value, time }); return set(value, time); };
          const start = node.start.bind(node);
          node.start = (time = 0) => {
            const value = values.filter((item) => item.time <= time).sort((a, b) => a.time - b.time).at(-1)?.value ?? node.frequency.value;
            window.decryptToneStarts.push(value);
            start(time);
          };
          return node;
        };
      }
    };
  });
  await page.goto("/#lab");
  await page.locator('#recipe-buttons button[data-recipe="suspense"]').click();
  await expect(page.locator("#score-title")).toContainText("Terminal A#");
  await expect(page.locator("#level-seed")).toHaveValue("level-001");
  await page.getByRole("button", { name: "Cue Decrypt", exact: true }).click();
  await page.locator("#center-play").click();
  await expect(page.locator("#mood-name")).toHaveText("Decrypt");
  await page.waitForFunction(() => window.scanAudio.firstStart !== null &&
    (window.scanAudio.blocks.at(-1)?.time ?? 0) > window.scanAudio.firstStart + 4);
  const result = await page.evaluate(async () => ({
    frequencies: window.decryptToneStarts,
    kicks: window.scanAudio.kicks.length,
    audio: await window.scanAudio.finishRecording!(),
  }));
  await page.locator("#start-audio").click();
  const file = testInfo.outputPath("decrypt-confirmed-tone-removed.webm");
  await writeFile(file, Buffer.from(result.audio, "base64"));
  await testInfo.attach("decrypt-confirmed-tone-removed.webm", { path: file, contentType: "audio/webm" });
  const beep = 440 * 2 ** ((70 - 69) / 12);
  expect(result.frequencies.some((frequency) => Math.abs(frequency - beep) < 0.01)).toBe(false);
  expect(result.frequencies.some((frequency) => Math.abs(frequency - beep * 2.003) < 0.01)).toBe(false);
  expect(result.frequencies.some((frequency) => Math.abs(frequency - beep / 4) < 0.01)).toBe(true);
  expect(result.kicks).toBeGreaterThanOrEqual(3);
});
