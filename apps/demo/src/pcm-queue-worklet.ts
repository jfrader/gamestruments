import { PCM_QUEUE_PROCESSOR, PCM_REPORT_QUANTA, type PcmPlayedReport } from "./pcm-queue.ts";

// The AudioWorkletGlobalScope, which the DOM library does not describe.
declare const currentTime: number;
declare class AudioWorkletProcessor {
  readonly port: MessagePort;
}
declare function registerProcessor(name: string, processor: new () => AudioWorkletProcessor): void;

/** Plays the mono blocks the main thread posts, in order, on every output
 *  channel; silence while the queue is empty. */
class PcmQueueProcessor extends AudioWorkletProcessor {
  readonly #queue: Float32Array[] = [];
  #offset = 0;
  #played = 0;
  #quanta = 0;

  constructor() {
    super();
    this.port.onmessage = (event: MessageEvent<Float32Array>) => {
      this.#queue.push(event.data);
    };
  }

  process(_inputs: Float32Array[][], outputs: Float32Array[][]): boolean {
    const output = outputs[0] ?? [];
    const frames = output[0]?.length ?? 0;
    let written = 0;
    while (written < frames) {
      const block = this.#queue[0];
      if (block === undefined) break;
      const count = Math.min(frames - written, block.length - this.#offset);
      const samples = block.subarray(this.#offset, this.#offset + count);
      for (const channel of output) channel.set(samples, written);
      written += count;
      this.#offset += count;
      if (this.#offset === block.length) {
        this.#queue.shift();
        this.#offset = 0;
      }
    }
    this.#played += written;
    this.#quanta += 1;
    if (this.#quanta % PCM_REPORT_QUANTA === 0) {
      const report: PcmPlayedReport = { played: this.#played, time: currentTime };
      this.port.postMessage(report);
    }
    return true;
  }
}

registerProcessor(PCM_QUEUE_PROCESSOR, PcmQueueProcessor);
