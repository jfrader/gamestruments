# Limitations

These are honest, shipped limitations of the v1 kit. They are not bugs.

## Synthesis Model

- Mono synth only. Internal sample rate is 22050 Hz. Output is pushed as identical left/right frames.
- Pure synthesis (voices: epiano, supersaw/pulse, pluck, chip squares, triangle bass, glass, warm organ, etc.). No sample import, no WAV/MIDI export, no sample library.
- No multi-channel, stereo imaging, or spatialization inside the engine.
- No reverb, delay, or effects beyond what the individual voices implement (e.g. neon echo is part of the voice definition in the generator).

## Sections and Adaptation

- Always generates exactly six sections: `garage`, `grid`, `cruise` (Race Flow), `attack` (Position Fight), `final-lap`, `victory` (Finish).
- Adaptive transport starts in "garage". `set_race_state` drives transitions among the sections using the built-in rule set (intensity, position_pressure, final_lap, race_phase).
- State changes are quantized to bar boundaries. New sections start at phrase bar zero.
- The `phase` strings you pass to `set_race_state` are matched against the adaptive rules. Using unknown phases may leave the transport in the prior section.
- `finish_result` is always set to `"none"` in the shipped `set_race_state` wrapper (victory rule exists for completeness but is not exercised by the public API in v1).

## Determinism and Versions

- Output is deterministic for a given `(secret, seed, style, palette, traits, generator version)`.
- No byte-identical guarantee across minor Godot or gdext patch releases. Musical identity + section structure + event timing/velocity/voice/pitch are the contract.
- Generator version is embedded (see engine source at release tag).

## Platforms and Builds

- v1 ships a Linux x86_64 binary only (`libgamestruments_godot.so`).
- Windows and macOS require a source rebuild from the included crates (escape hatch). Full runtime QA for those platforms is tracked as GURI-485 and not part of this release.
- Web, mobile, and console targets are out of scope for v1.
- The GDExtension uses gdext 0.5 series at build time; buyers rebuilding must match the pinned Rust + gdext surface for binary parity.

## Integration Surface

- Only `GamestrumentsPlayer` (a `Node` subclass) and its documented exports + two methods are supported.
- The internal `AudioStreamPlayer` child is created with a fixed name and bus routing. Do not assume you can freely reparent or replace it.
- No Godot editor plugins, custom resource types, or visual scripting nodes.
- No authoring UI or pattern editor of any kind (the Audio Lab is a separate AGPL tool, not shipped).

## Audio Routing & Godot Specifics

- Requires (or creates) a bus named "Music". If the bus is missing or muted you will hear silence.
- Godot `AudioStreamGenerator` + `AudioStreamGeneratorPlayback` lifetime management is delicate. The shipped implementation uses transient lookups + explicit stop/null/free on exit_tree / PREDELETE to avoid ObjectDB leaks from the player itself. Background engine leaks unrelated to this kit may still appear in the debugger.
- `generate` must be called at least once before state changes or you will get no audio.

## Content Scope

- No pre-rendered audio assets, no catalog of WAVs, no redistribution of samples.
- The Rust source is included so buyers can inspect, rebuild, or fork under MIT terms (with the usual gdext MPL-2.0 combined-work notices).
- No Strudel, no TypeScript runtime, no Node/TS authoring code in the kit.

See `kit-contract.md` for the complete non-goals list and support expectations. See `troubleshooting.md` for how to diagnose the common symptoms of these limitations.
