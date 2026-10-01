import type { Page } from "@playwright/test";

/** The audio the engine sends to the Lab's worklet. */
interface EngineAudio {
  blocks: number;
  frames: number;
  peak: number;
  /** The first ten seconds of samples, for level checks. */
  samples: number[];
}

declare global { interface Window { engineAudio: EngineAudio } }

/** The engine renders at this rate; see apps/demo/src/engine-playback.ts. */
export const ENGINE_SAMPLE_RATE = 48000;
const KEPT_SECONDS = 10;

/** Record the mono blocks the engine posts to the Lab's audio worklet. */
export async function captureEngineAudio(page: Page): Promise<void> {
  await page.addInitScript((kept) => {
    const capture: EngineAudio = { blocks: 0, frames: 0, peak: 0, samples: [] };
    window.engineAudio = capture;
    const post = MessagePort.prototype.postMessage;
    MessagePort.prototype.postMessage = function (this: MessagePort, message: unknown, ...rest: unknown[]) {
      if (message instanceof Float32Array) {
        capture.blocks += 1;
        capture.frames += message.length;
        for (const sample of message) {
          capture.peak = Math.max(capture.peak, Math.abs(sample));
          if (capture.samples.length < kept) capture.samples.push(sample);
        }
      }
      return (post as (...args: unknown[]) => void).call(this, message, ...rest);
    } as typeof MessagePort.prototype.postMessage;
  }, ENGINE_SAMPLE_RATE * KEPT_SECONDS);
}

/** Wait until `seconds` of engine audio have been rendered. */
export async function waitForEngineSeconds(page: Page, seconds: number, timeout = 60000): Promise<void> {
  await page.waitForFunction(
    (frames) => window.engineAudio.frames >= frames,
    seconds * ENGINE_SAMPLE_RATE,
    { timeout },
  );
}

/** Wait until the Lab's bar/beat counter reaches `bar` and `beat`. */
export async function waitForBeat(page: Page, bar: number, beat: number, timeout = 60000): Promise<void> {
  await page.waitForFunction(
    ([bar, beat]) => {
      const current = Number(document.querySelector("#bar-value")?.textContent);
      const currentBeat = Number(document.querySelector("#beat-value")?.textContent);
      return current > bar || (current === bar && currentBeat >= beat);
    },
    [bar, beat] as const,
    { timeout },
  );
}
