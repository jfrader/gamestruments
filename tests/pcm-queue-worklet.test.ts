import assert from "node:assert/strict";
import { afterEach, describe, it, vi } from "vitest";
import { PCM_QUEUE_PROCESSOR, PCM_REPORT_QUANTA } from "../apps/demo/src/pcm-queue.ts";

type Worklet = {
  port: {
    onmessage: ((event: { data: Float32Array }) => void) | null;
    postMessage: ReturnType<typeof vi.fn>;
  };
  process(inputs: Float32Array[][], outputs: Float32Array[][]): boolean;
};

async function processor(): Promise<Worklet> {
  let constructor: (new () => Worklet) | undefined;
  class StubProcessor {
    readonly port = { onmessage: null, postMessage: vi.fn() };
  }
  vi.stubGlobal("AudioWorkletProcessor", StubProcessor);
  vi.stubGlobal("currentTime", 1.25);
  vi.stubGlobal("registerProcessor", (name: string, registered: new () => Worklet) => {
    assert.equal(name, PCM_QUEUE_PROCESSOR);
    constructor = registered;
  });
  vi.resetModules();
  await import("../apps/demo/src/pcm-queue-worklet.ts");
  assert.ok(constructor);
  return new constructor();
}

function quantum(channels: number): Float32Array[][] {
  return [Array.from({ length: channels }, () => new Float32Array(128))];
}

afterEach(() => vi.unstubAllGlobals());

describe("PCM queue output", () => {
  for (const channels of [1, 2]) {
    it(`preserves mono energy across ${channels} output channel(s) and queued block boundaries`, async () => {
      const worklet = await processor();
      const silence = quantum(channels);
      assert.equal(worklet.process([], silence), true);
      for (const channel of silence[0]!) assert.ok(channel.every((sample) => sample === 0));
      const first = Float32Array.from({ length: 75 }, (_, index) => (index % 5 - 2) * 0.125);
      const second = Float32Array.from({ length: 100 }, (_, index) => (index % 7 - 3) * 0.125);
      worklet.port.onmessage?.({ data: first });
      worklet.port.onmessage?.({ data: new Float32Array(0) });
      worklet.port.onmessage?.({ data: second });

      const played = [...first, ...second];
      for (let block = 0; block < PCM_REPORT_QUANTA - 1; block++) {
        const output = quantum(channels);
        assert.equal(worklet.process([], output), true);
        for (let frame = 0; frame < 128; frame++) {
          const sample = played[block * 128 + frame] ?? 0;
          const power = output[0]!.reduce((sum, channel) => sum + channel[frame]! ** 2, 0);
          assert.ok(Math.abs(power - sample ** 2) < 1e-7, `frame ${block * 128 + frame}`);
          if (channels === 1) assert.equal(output[0]![0]![frame], sample);
        }
      }
      assert.deepEqual(worklet.port.postMessage.mock.calls, [[{ played: played.length, time: 1.25 }]]);
    });
  }
});
