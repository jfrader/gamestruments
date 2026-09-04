# Gamestruments Godot 4 Kit — Product Contract

**Status**: approved. Standard price $24.99 (approved by Fran 2026-09-04). This document is the authoritative buyer contract.

## Supported Versions

- Engine: Godot 4.x (`.gdextension` declares `compatibility_minimum = 4.2`; developed and validated against 4.7+).
- Binding technology: GDExtension (current build uses gdext 0.4.5 series).
- Renderer / audio path: Godot `AudioStreamGenerator` + `AudioStreamGeneratorPlayback` (mono 22050 Hz internal synth, exposed via Music bus).
- File formats at runtime: none (generation and synthesis happen inside the extension from secret + seed + palette).
- Platforms: linux.x86_64 shipped in initial release. Windows and macOS binaries pending platform runtime QA (GURI-485). Source builds are the escape hatch.
- Rust toolchain: pinned via the release workflow at tag time (buyers rebuilding from source use stable).

## Exact Inventory Buyers Receive

The buyer archive (`gamestruments-<version>-godot4.zip`) contains:

- GDExtension binary (`libgamestruments_godot.so` / `.dll` / `.dylib` as appropriate for the build matrix) placed for `res://addons/gamestruments/bin/...`.
- `gamestruments.gdextension` file (configuration + library paths).
- Full MIT-licensed Rust core source: the `crates/engine` (generator, transport, synth) and `crates/godot` (GDExtension glue) directories, including `Cargo.toml`, `src/`, and any required build files to rebuild the cdylib.
- A minimal demo scene (and supporting files) exercising the public API in a fresh Godot project.
- Documentation under `kit/docs/` (and root for contract/opportunity): `README.md` (scope, requirements, quickstart), `quickstart.md`, `api.md`, `limitations.md`, `troubleshooting.md`; plus `docs/kit-contract.md`, `docs/kit-opportunity.md`, `CHANGELOG.md` excerpt.
- Licenses: `LICENSE.md` (or equivalent) declaring MIT for the Gamestruments Rust crates; third-party notices for all dependencies that require them (including the MPL-2.0 gdext runtime used to produce the binary).
- `CHANGELOG.md` excerpt for the kit.

No Strudel, no samples, no Node/TS authoring code, no pre-rendered audio assets are included in the runtime kit.

## Demo Scope

The included demo scene shows:
- Setting `project_secret`, `style`, voice/palette fields, and trait values in the inspector.
- Calling `generate(seed)` at level load / _ready.
- Driving `set_race_state(phase, intensity, pressure, final_lap)` from game logic (buttons or exported methods).
- Hearing synthesized music with bar-quantized state transitions.

The demo is intentionally minimal; it is not a full game. It proves the integration path a buyer will use.

## Public API the Buyer May Rely On

`GamestrumentsPlayer` (extends Node) — the only class buyers instantiate and script against:

Exported properties (settable in inspector or from code):
- `project_secret: String` — per-title secret. Must be non-empty. "Do not use a public name like 'pocket-circuit'."
- `style: String` — e.g. "funk", "fusion", "neon", "chip".
- Voice / palette fields (exact names): `melody_voice`, `harmony_voice`, `drive_voice`, `bass_voice` (strings; empty uses style defaults).
- Trait fields: `energy`, `complexity`, `brightness`, `syncopation` (floats 0..1).

Methods:
- `generate(seed: String)` — generates the deterministic Pocket Circuit score from secret + seed + style + palette + traits. Must be called before state changes. Sets internal transport and synth.
- `set_race_state(phase: String, intensity: float, pressure: float, final_lap: bool)` — requests a state change; the transport quantizes to the next bar boundary and performs the crossover.

Additional observable behavior buyers can rely on (documented):
- Generation is deterministic for the tuple (secret, seed, style, palette, traits, generator version).
- State changes commit on bar boundaries; sections start at phrase bar zero.
- Audio is rendered at 22050 Hz mono internally and pushed to the Godot generator playback (buyer routes via "Music" bus).

No other public symbols or extension points are part of the supported contract. Internal modules may change.

## Explicit Non-Goals

- No authoring UI, Strudel Lab, or pattern editor ships in the kit.
- No sample import, sample library, or WAV/MIDI export path for buyers.
- No Godot < 4 support.
- No FMOD, Wwise, or other middleware bridging.
- No pre-baked catalog of WAVs; generation happens at runtime in the shipped game.
- No web, mobile, or console platform guarantees in v1.
- No guarantee of byte-identical output across minor Godot or gdext patch releases (semantic + musical identity is the contract).

## Buyer Archive Name, Versioning, and Release Channels

- Archive: `gamestruments-<version>-godot4.zip` (e.g. `gamestruments-0.2.0-godot4.zip`).
- Versioning: follows the workspace crate version at tag time. A matching git tag `v*` triggers the release workflow.
- Release channels: tags produce the candidate; buyers pull from itch "stable" release. Pre-releases / dev builds are not part of the supported buyer contract.
- The release workflow (`.github/workflows/release.yml`) builds on `workflow_dispatch` and `v*` tag push for the three desktop OSes and attaches the per-platform libraries + sha256 sidecars.

## Support, Maintenance, Compatibility, Refund, and Update Expectations

- Support: best-effort via GitHub issues for integration and reproducible bugs in the advertised scope. Response target: acknowledgement within 5 business days for paid-kit buyers.
- Maintenance: the kit will track Godot 4.x and gdext stable releases that do not break the public API surface. Major Godot 5 or breaking gdext changes are out of scope for this contract.
- Compatibility: a buyer who pins the exact archive and Godot version used at purchase time can expect the generate + set_race_state contract to continue working. Rebuilding from the included source uses the Rust stable at build time.
- Refunds: per itch.io policy (generally 30 days / reasonable use). High refund rates trigger the kill/pivot criteria in the opportunity brief.
- Updates: purchasers of a version receive the source for that version. Future versions are separate purchases or may be offered as upgrades at the author's discretion. No automatic update mechanism is provided.

## License Terms

- The Rust core (`crates/engine` and `crates/godot` glue) is licensed under the MIT License. Buyers receive the full source and may use, modify, and redistribute it under the MIT terms (including in closed-source games).
- The GDExtension binary ships under the same MIT terms as the Rust source from which it is built.
- Verification against crates: workspace and both `gamestruments-*` crates declare `license = "MIT"`. The `godot` (gdext) runtime dependency is MPL-2.0. Buyers must receive appropriate license notices for all components. Inclusion of full MPL-2.0 text + any required notices for the binary distribution path must be confirmed before the first binary-bearing release.
- No AGPL or Strudel code enters the runtime path or the buyer archive.

## Price

Standard price: $24.99 (approved by Fran 2026-09-04). No launch discounts for v1.

See `docs/kit-opportunity.md` for the dated comparables table and rationale.

## Claim-to-Evidence Notes (internal)

Every statement above maps to:
- shipped files in the archive,
- the `GamestrumentsPlayer` class and its exported API (observable in a clean Godot project),
- the release workflow producing the named artifacts,
- the reproducibility test for the reserved take (empty secret + "level-004" + funk traits → `pocket-circuit-generated-v1-9-0-7864ec71`),
- the MIT declarations in `Cargo.toml` files and the notices that will accompany the archive.

If a claim cannot be demonstrated from the archive + docs alone, it is removed.
