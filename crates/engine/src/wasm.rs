//! WASM32 facade (only compiled for --target wasm32-unknown-unknown).
//! Provides a minimal, allocator-friendly C ABI for calling the engine from
//! JS (Node WebAssembly, browsers, etc) without wasm-bindgen or JS glue.
//!
//! Build: `cargo build -p gamestruments-engine --target wasm32-unknown-unknown --release`
//! Produces: target/wasm32-unknown-unknown/release/gamestruments_engine.wasm
//!
//! ## API (no_mangle extern "C")
//!
//! ```c
//! uint8_t* gamestruments_alloc(size_t size);
//! uint8_t* gamestruments_score_json(const uint8_t* ptr, size_t len);  // returns out_ptr
//! size_t   gamestruments_output_len(void);
//! uint8_t* gamestruments_render_wav(const uint8_t* score_ptr, size_t score_len,
//!                                   const uint8_t* section_ptr, size_t section_len,
//!                                   size_t phrases);
//! void     gamestruments_reset(void);  // optional: resets bump for fresh allocs
//! ```
//!
//! Input JSON for score_json (exact shape, all keys lowercase):
//! {
//!   "secret": "...",
//!   "seed": "...",
//!   "style": "funk" | "chip" | "fusion" | "neon",
//!   "palette": { "melody": "", "harmony": "", "drive": "", "bass": "" },  // empty = default kit
//!   "energy": 0.62,
//!   "complexity": 0.6,
//!   "brightness": 0.52,
//!   "syncopation": 0.7
//! }
//!
//! Returns: pointer to UTF-8 JSON bytes of PortableScore (schema camelCase as usual).
//!
//! For render_wav: pass *bytes* of a previously-produced score JSON as score_ptr/len,
//! section e.g. "cruise", phrases e.g. 3. Hardcodes 22050 Hz mono 16-bit WAV to match
//! native golden tests and render_wav(..., 22050).
//!
//! ## Ownership contract (critical for callers)
//!
//! - Caller is responsible for allocation of *inputs* via gamestruments_alloc(size).
//!   Write your bytes (JSON utf8, or raw score json bytes, or section utf8) into the
//!   returned region of wasm linear memory. Do not free inputs yourself; they live in
//!   the bump arena until gamestruments_reset() or next full reset.
//! - Outputs (from score_json / render_wav) are written into a *process-global static buffer*
//!   inside the module. The returned pointer is valid **only until the next call** to
//!   score_json or render_wav (which overwrite the buffer).
//! - **You MUST copy the bytes out immediately** (in JS: `new Uint8Array(memory.buffer, ptr, len).slice()`
//!   or Buffer.from(...).slice()). After copy, the slice is yours forever.
//! - No explicit free for outputs. Call gamestruments_reset() between unrelated batches
//!   if you want to reclaim the bump arena for new inputs (does not affect in-flight output ptrs
//!   you already copied).
//! - Memory may grow; always re-acquire `memory` and re-compute views after any call that
//!   might have grown (rare for <1MB payloads).
//! - All numbers little-endian. Panics on bad utf8/json become wasm traps (visible as JS errors).
//! - Determinism: identical inputs on same build produce bit-identical outputs on WASM and native.
//!
//! This is the standard "caller allocates + global output ring" pattern used by many
//! pure-Rust WASM crates that avoid heavier glue.

#![cfg(target_arch = "wasm32")]
#![allow(static_mut_refs)] // intentional global output buffer + bump for no-alloc FFI; single code path, no std-only APIs

use core::slice;

use crate::pocket_circuit::{generate_pocket_circuit, GenerateInput, InstrumentPalette, Style};
use crate::render::render_wav;
use crate::score::PortableScore;

const BUF_SIZE: usize = 2 * 1024 * 1024; // 2 MiB headroom for JSON + WAV (3phrases@22k ~300k)
static mut BUFFER: [u8; BUF_SIZE] = [0u8; BUF_SIZE];
static mut BUMP: usize = 0;
static mut OUT_PTR: *const u8 = core::ptr::null();
static mut OUT_LEN: usize = 0;

#[no_mangle]
pub extern "C" fn gamestruments_alloc(size: usize) -> *mut u8 {
    unsafe {
        let start = BUMP;
        if start + size > BUF_SIZE {
            return core::ptr::null_mut();
        }
        BUMP += size;
        // return pointer into our static; caller writes via memory view
        let base = &raw mut BUFFER as *mut u8;
        base.add(start)
    }
}

#[no_mangle]
pub extern "C" fn gamestruments_reset() {
    unsafe {
        BUMP = 0;
        OUT_PTR = core::ptr::null();
        OUT_LEN = 0;
    }
}

fn write_output(data: &[u8]) {
    unsafe {
        let len = data.len().min(BUF_SIZE);
        // copy via raw to avoid creating & to mut static
        let base = &raw mut BUFFER as *mut u8;
        core::ptr::copy_nonoverlapping(data.as_ptr(), base, len);
        OUT_PTR = base;
        OUT_LEN = len;
    }
}

#[no_mangle]
pub extern "C" fn gamestruments_output_len() -> usize {
    unsafe { OUT_LEN }
}

#[no_mangle]
pub unsafe extern "C" fn gamestruments_score_json(input_ptr: *const u8, input_len: usize) -> *const u8 {
    let input_slice = slice::from_raw_parts(input_ptr, input_len);
    let json_str = core::str::from_utf8(input_slice).expect("input must be valid utf-8");

    #[derive(serde::Deserialize)]
    struct Pal {
        melody: String,
        harmony: String,
        drive: String,
        bass: String,
    }
    #[derive(serde::Deserialize)]
    struct Inp {
        secret: String,
        seed: String,
        style: String,
        palette: Pal,
        energy: f64,
        complexity: f64,
        brightness: f64,
        syncopation: f64,
    }

    let inp: Inp = serde_json::from_str(json_str).expect("invalid parity input JSON");
    let style = Style::parse(&inp.style).expect("unknown style");
    let palette = InstrumentPalette {
        melody: inp.palette.melody,
        harmony: inp.palette.harmony,
        drive: inp.palette.drive,
        bass: inp.palette.bass,
    };
    let gen = GenerateInput {
        secret: inp.secret,
        seed: inp.seed,
        style,
        palette,
        energy: inp.energy,
        complexity: inp.complexity,
        brightness: inp.brightness,
        syncopation: inp.syncopation,
    };

    let score = generate_pocket_circuit(&gen);
    let out = serde_json::to_vec(&score).expect("score serialize failed");
    write_output(&out);
    unsafe { OUT_PTR }
}

#[no_mangle]
pub unsafe extern "C" fn gamestruments_render_wav(
    score_ptr: *const u8,
    score_len: usize,
    section_ptr: *const u8,
    section_len: usize,
    phrases: usize,
) -> *const u8 {
    let score_slice = slice::from_raw_parts(score_ptr, score_len);
    let score: PortableScore = serde_json::from_slice(score_slice).expect("score JSON must parse");

    let section_slice = slice::from_raw_parts(section_ptr, section_len);
    let section = core::str::from_utf8(section_slice).expect("section must be valid utf-8");

    // Hardcode 22050 to match all golden/render tests and native parity_ref
    let wav = render_wav(&score, section, phrases, 22050);
    write_output(&wav);
    unsafe { OUT_PTR }
}
