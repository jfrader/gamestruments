import { expect, test, type Page } from "@playwright/test";

test.use({
  launchOptions: { args: ["--autoplay-policy=no-user-gesture-required"] },
});

interface HeadroomMonitor {
  compressors: DynamicsCompressorNode[];
  preNode: AudioNode | null;
}

declare global {
  interface Window {
    __headroomMonitor: HeadroomMonitor;
  }
}

// A fresh GainNode sits at unity gain until its first scheduled automation
// lands, so an uninitialized kick-click envelope can leak a unity-gain onset.
// This drives the post-limiter peak from ~0.175 (normal
// kick) to ~0.61 at the reproduced 144bpm boundary; 0.3 is the safe midpoint.
const FUSION = {
  energy: 0.62,
  complexity: 0.68,
  brightness: 0.52,
  syncopation: 0.72,
} as const;

const MONITOR_HOOK = `(() => {
  const Native = window.AudioContext;
  if (!Native) return;
  const monitor = { compressors: [], preNode: null };
  window.__headroomMonitor = monitor;
  const createCompressor = AudioContext.prototype.createDynamicsCompressor;
  AudioContext.prototype.createDynamicsCompressor = function () {
    const node = createCompressor.call(this);
    monitor.compressors.push(node);
    return node;
  };
  const connect = AudioNode.prototype.connect;
  AudioNode.prototype.connect = function (destination, ...rest) {
    const result = connect.call(this, destination, ...rest);
    if (
      destination instanceof AudioNode &&
      monitor.compressors[0] &&
      destination === monitor.compressors[0] &&
      !monitor.preNode
    ) {
      monitor.preNode = this;
    }
    return result;
  };
})();`;

interface KickLoopMeasurement {
  bpm: number;
  postPeak: number;
  prePeak: number | null;
  rms: number;
  loopSeconds: number;
}

async function measureKickLoop(
  page: Page,
  bpm: number,
): Promise<KickLoopMeasurement> {
  return page.evaluate(
    async ({ bpm, traits }: { bpm: number; traits: typeof FUSION }) => {
      const { generateScore } = (await import(
        "/src/wasm-engine.ts" as string
      )) as typeof import("../../apps/demo/src/wasm-engine.ts");
      const { DemoAudioEngine } = (await import(
        "/src/audio-engine.ts" as string
      )) as typeof import("../../apps/demo/src/audio-engine.ts");
      const score = await generateScore({
        seed: "level-001",
        style: "fusion",
        recipe: "racing",
        arrangement: "original",
        autoplay: false,
        energy: traits.energy,
        complexity: traits.complexity,
        brightness: traits.brightness,
        syncopation: traits.syncopation,
        tension: traits.energy,
        heat: traits.complexity,
        mystery: traits.brightness,
        pulse: traits.syncopation,
      });

      // In-memory kick-only clone; the generated score is never mutated.
      const clone = JSON.parse(JSON.stringify(score)) as typeof score;
      clone.bpm = bpm;
      clone.sections = clone.sections.map((section) => ({
        ...section,
        events: section.events.filter(
          (event) =>
            event.kind === "percussion" &&
            event.voice === "kick" &&
            event.lane.endsWith("-kit"),
        ),
      }));

      const monitor = window.__headroomMonitor;
      monitor.preNode = null;
      monitor.compressors.length = 0;

      const engine = new DemoAudioEngine(clone);
      engine.volume = 0.5;
      await engine.start("attack");

      const actx = monitor.compressors[0]!.context;
      const limiter = monitor.compressors[1]!;
      const preNode = monitor.preNode;

      const tap = (source: AudioNode | null) => {
        const proc = actx.createScriptProcessor(256, 2, 2);
        const mute = actx.createGain();
        mute.gain.value = 0;
        if (source !== null) source.connect(proc);
        proc.connect(mute).connect(actx.destination);
        const agg = {
          minL: Infinity,
          maxL: -Infinity,
          minR: Infinity,
          maxR: -Infinity,
          sumSqL: 0,
          sumSqR: 0,
          frames: 0,
        };
        proc.onaudioprocess = (event) => {
          const left = Array.from(event.inputBuffer.getChannelData(0));
          const right = Array.from(event.inputBuffer.getChannelData(1));
          for (const sample of left) {
            agg.minL = Math.min(agg.minL, sample);
            agg.maxL = Math.max(agg.maxL, sample);
            agg.sumSqL += sample * sample;
          }
          for (const sample of right) {
            agg.minR = Math.min(agg.minR, sample);
            agg.maxR = Math.max(agg.maxR, sample);
            agg.sumSqR += sample * sample;
          }
          agg.frames += left.length;
        };
        return agg;
      };

      const limAgg = tap(limiter);
      const preAgg = tap(preNode);

      const secsPerTick = 60 / bpm / score.ticksPerBeat;
      const attack = score.sections.find((section) => section.id === "attack")!;
      const loopSeconds = attack.lengthTicks * secsPerTick;
      const t0 = actx.currentTime;
      while (actx.currentTime - t0 < loopSeconds + 1.5) {
        await new Promise((resolve) => setTimeout(resolve, 15));
      }
      await engine.stop();

      const peakOf = (agg: ReturnType<typeof tap>) =>
        Math.max(
          Math.max(Math.abs(agg.minL), Math.abs(agg.maxL)),
          Math.max(Math.abs(agg.minR), Math.abs(agg.maxR)),
        );
      const rmsOf = (agg: ReturnType<typeof tap>) =>
        Math.max(
          Math.sqrt(agg.sumSqL / agg.frames),
          Math.sqrt(agg.sumSqR / agg.frames),
        );

      return {
        bpm,
        postPeak: peakOf(limAgg),
        prePeak: preNode !== null ? peakOf(preAgg) : null,
        rms: rmsOf(limAgg),
        loopSeconds,
      };
    },
    { bpm, traits: FUSION },
  );
}

test("Racing kick-only loop keeps the post-limiter peak bounded", async ({ page }) => {
  test.setTimeout(150000);
  await page.addInitScript(MONITOR_HOOK);
  await page.goto("/#lab");
  await page.waitForLoadState("networkidle");
  await expect(page.locator("#generator-summary")).toContainText("engine: wasm", {
    timeout: 30000,
  });

  const at144 = await measureKickLoop(page, 144);
  const at134 = await measureKickLoop(page, 134);

  console.log("racing-headroom 144bpm:", JSON.stringify(at144));
  console.log("racing-headroom 134bpm control:", JSON.stringify(at134));

  // The 144bpm boundary leaks a unity-gain click when the noise envelope is
  // uninitialized (~0.61); initialized it returns to the intended ~0.175.
  expect(at144.rms, "144bpm loop must carry audible energy").toBeGreaterThan(0.005);
  expect(at144.postPeak, "144bpm loop must not be silent").toBeGreaterThan(0.02);
  expect(at144.postPeak, "144bpm post-limiter peak must stay bounded").toBeLessThan(0.3);

  expect(at134.rms, "134bpm control must carry audible energy").toBeGreaterThan(0.005);
  expect(at134.postPeak, "134bpm control must not be silent").toBeGreaterThan(0.02);
  expect(at134.postPeak, "134bpm control post-limiter peak must stay bounded").toBeLessThan(0.3);
});
