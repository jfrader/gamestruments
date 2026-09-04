# Gamestruments — Godot 4 Kit

Seed-driven, sample-free, runtime-adaptive music scores for Godot 4 games.

A single `project_secret` (per-title) + instrument palette + seed produces a deterministic adaptive score at level load. Drive bar-quantized state changes (e.g. race phases) at runtime. Pure synthesis; no samples or authoring tools cross the game boundary.

**MIT license** on the Rust core. Full source included.

## Requirements

- Godot 4.x (`.gdextension` declares `compatibility_minimum = 4.2`; developed against 4.7+).
- Supported platforms (v1): Linux x86_64 (binary provided). Windows/macOS require source rebuild (pending runtime QA — see GURI-485).
- No external audio assets or middleware required.

## Quickstart

1. Copy the `addons/gamestruments/` tree (or the equivalent layout from the archive) into your Godot project so the `.gdextension` and `bin/libgamestruments_godot.so` are under `res://addons/gamestruments/`.
2. Add a `GamestrumentsPlayer` node (or make it an autoload).
3. In the inspector (or from code) set:
   - `project_secret` — a non-empty per-title secret (never a public string like the game name).
   - `style` — one of `fusion`, `neon`, `funk`, `chip`.
   - Optional: `melody_voice` / `harmony_voice` / `drive_voice` / `bass_voice` (strings; empty uses style defaults).
   - Optional traits (0..1 floats): `energy`, `complexity`, `brightness`, `syncopation`.
4. At level load / `_ready` call `generate("your-level-seed")`.
5. Drive runtime changes with `set_race_state(phase, intensity, pressure, final_lap)` (phase strings such as "garage", "grid", "race", "attack", etc.; see api.md).
6. Ensure your project has (or creates) a "Music" bus; the player routes to it. Audio is mono 22050 Hz internally.

See `quickstart.md` for a complete minimal fresh-project integration.

## Scope (What Ships)

- GDExtension binary + `.gdextension` for the supported platform.
- Full MIT Rust source (`crates/engine` + `crates/godot`).
- Minimal demo scene exercising the public API.
- Buyer documentation (`kit/docs/`): this README, quickstart, API reference, limitations, troubleshooting.
- `LICENSE.md`, third-party notices (gdext is MPL-2.0), CHANGELOG excerpt.

**Explicit non-goals (v1)**: no samples, no Strudel, no authoring UI, no pre-rendered WAVs, no Windows/macOS binaries, no web/mobile/console guarantees, no FMOD/Wwise interop.

## Main Claims (Literal)

- `secret + seed + palette + style + traits` → unique deterministic adaptive score (verified by parity test + demo).
- Zero samples in the archive or runtime path.
- Bar-quantized crossovers on state changes via `set_race_state`.
- Godot 4.x native via GDExtension (mono synth only).

See `limitations.md` and `kit-contract.md` for the full honest contract.

## Verification

This kit was built for reproducibility:
- Buyer docs live in `kit/docs/` inside the archive.
- All claims map to shipped code behavior (see `crates/godot/src/lib.rs` and `crates/engine/src/pocket_circuit.rs`).
- Clean-room buyer test is required before release.
- Author QA runbook (Fran's manual checklist + results) lives in the repo at `docs/kit-qa-runbook.md` (not part of the buyer archive).

## Support

Best-effort via GitHub issues for integration and reproducible bugs in the advertised scope. See `kit-contract.md` for maintenance, compatibility, and refund expectations.

Standard price: $24.99 (approved).

For the opportunity, full contract, and post-launch measurement plan see the `docs/` files in the archive (or the repo at release tag).
