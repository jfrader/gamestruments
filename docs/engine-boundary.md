# Engine Boundary (Audio Lab + shared Rust engine)

Generation of `PortableScore` (sections, events, id, bpm, rules, crossfades, defaultSection):
- Owned by the shared Rust engine in `crates/engine` (pocket_circuit + score).
- Exposed to the lab via WASM: `gamestruments_score_json` (see `crates/engine/src/wasm.rs`).
- The TypeScript `@gamestruments/studio` generator remains the authoring reference and test oracle only; its outputs are not used at runtime in the lab anymore.

Transport / section selection + transition planning:
- For the web Audio Lab: blessed implementation is `@gamestruments/runtime` (`AdaptiveTransport` + `selectSection` over score rules + nested GameState).
- The Rust `AdaptiveTransport` + `select_section` (in `crates/engine/src/transport.rs`) mirror the behavior for Godot/games.
- This small state machine duplication is accepted and documented; the parity harness covers score identity and selection outcomes. Adding a WASM transport facade was considered but rejected for lower risk to lab UX/timing.

Audio synthesis / sound stage:
- The signed-off sound (SYNTH_VOICES: warm/glass/pulse/pluck/chip + dedicated epiano/organ/supersaw/triangle/bass + kit + full room bus + stereo imaging + compression) lives in `apps/demo/src/audio-engine.ts` (Web Audio).
- The Rust `Synth` (mono, no room) is the reference for the game engine only.
- Path A chosen: WASM owns generation (and could own transport math), Web Audio owns the voices. No AudioWorklet streaming of Rust synth (would lose the signed-off room sound without a full port; listening comparison impossible here).

WASM loader for lab: `apps/demo/src/wasm-engine.ts` (plain instantiateStreaming + typed `generateScore` / `renderWav` wrappers; follows the ownership/alloc contract from wasm.rs and tests/wasm-parity.mjs).

See also: README, docs/procedural-generation.md, AGENTS.md (one engine, two frontends).
