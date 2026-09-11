import { expect, test } from "@playwright/test";
import { installAudioCapture } from "./audio-capture.ts";

for (const voice of ["reverse-cymbal", "air-impact"]) {
  test(`${voice} renders its intended envelope through Web Audio`, async ({ page }) => {
    await installAudioCapture(page);
    await page.goto("/#lab");
    await page.evaluate(async (voice) => {
      const modulePath = "/src/audio-engine.ts";
      const { DemoAudioEngine } = await import(modulePath);
      const score = { schemaVersion: 1, id: "effect-probe", title: "Effect probe", bpm: 120,
        beatsPerBar: 4, ticksPerBeat: 960, crossfadeBars: 2, defaultSection: "fx", rules: [],
        sections: [{ id: "fx", label: "FX", feeling: "probe", color: "#000", lengthTicks: 7680,
          events: [{ id: `fx:${voice}`, section: "fx", lane: "fx", kind: "percussion", voice,
            startTick: 0, durationTicks: 1920, velocity: 0.2 }] }] };
      const engine = new DemoAudioEngine(score);
      await engine.start("fx");
      Object.assign(window, { effectProbe: engine });
    }, voice);
    await page.waitForFunction(() => window.scanAudio.firstStart !== null &&
      (window.scanAudio.blocks.at(-1)?.time ?? 0) > window.scanAudio.firstStart + 1.6);
    const levels = await page.evaluate(() => {
      const capture = window.scanAudio;
      const rms = (from: number, to: number) => {
        const samples = capture.blocks.filter((block) => block.time - capture.firstStart! >= from && block.time - capture.firstStart! < to).flatMap((block) => block.samples);
        return Math.sqrt(samples.reduce((sum, sample) => sum + sample * sample, 0) / samples.length);
      };
      return { early: rms(0.15, 0.35), late: rms(0.8, 1.0) };
    });
    await page.evaluate(async () => {
      const engine = (window as unknown as { effectProbe: { stop(): Promise<void> } }).effectProbe;
      await engine.stop();
    });
    if (voice === "reverse-cymbal") expect(levels.late).toBeGreaterThan(levels.early * 2);
    else expect(levels.early).toBeGreaterThan(levels.late * 2);
    expect(Math.max(levels.early, levels.late)).toBeGreaterThan(0.0002);
    expect(Math.max(levels.early, levels.late)).toBeLessThan(0.1);
  });
}
