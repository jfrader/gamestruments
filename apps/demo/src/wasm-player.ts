/** The engine's live player (the one Godot runs) through its WASM exports.
 *  It works on raw bytes only, so it also runs inside an AudioWorklet, which
 *  has no TextEncoder or TextDecoder. */

/** The most frames one `fill` renders; matches the engine's buffer. */
export const MAX_FILL_FRAMES = 8192;

interface PlayerExports {
  memory: WebAssembly.Memory;
  gamestruments_reset(): void;
  gamestruments_alloc(size: number): number;
  gamestruments_output_len(): number;
  gamestruments_status(): number;
  gamestruments_player_new(sampleRate: number): void;
  gamestruments_player_command(ptr: number, length: number): number;
  gamestruments_player_fill(frames: number): number;
}

export interface PlayerResponse {
  ok: boolean;
  /** UTF-8 JSON on success, a UTF-8 error message otherwise. */
  bytes: Uint8Array;
}

export class WasmPlayer {
  readonly #exports: PlayerExports;

  constructor(instance: WebAssembly.Instance, sampleRate: number) {
    this.#exports = instance.exports as unknown as PlayerExports;
    this.#exports.gamestruments_player_new(sampleRate);
  }

  /** Run one UTF-8 JSON player command. */
  command(json: Uint8Array): PlayerResponse {
    const exports = this.#exports;
    exports.gamestruments_reset();
    const ptr = exports.gamestruments_alloc(json.length);
    if (ptr === 0) {
      throw new Error("The player command does not fit in the engine buffer");
    }
    new Uint8Array(exports.memory.buffer).set(json, ptr);
    const output = exports.gamestruments_player_command(ptr, json.length);
    const bytes = new Uint8Array(exports.memory.buffer, output, exports.gamestruments_output_len()).slice();
    return { ok: exports.gamestruments_status() === 0, bytes };
  }

  /** The next `frames` mono samples, valid until the next fill. */
  fill(frames: number): Float32Array {
    const ptr = this.#exports.gamestruments_player_fill(frames);
    return new Float32Array(this.#exports.memory.buffer, ptr, Math.min(frames, MAX_FILL_FRAMES));
  }
}
