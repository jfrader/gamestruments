# Gamestruments Adaptive Racing Music for Godot 4 — Product Contract

**Status:** approved scope; release remains gated on automated platform checks, independent buyer testing, human listening, and explicit publication approval.
**Standard price:** $24.99.

This is the authoritative buyer contract for the first Gamestruments runtime kit.

## Supported Environment

- Godot 4.7 or newer via GDExtension.
- Linux x86_64, Windows x86_64, and macOS arm64/x86_64.
- Rust 1.94.0 and gdext 0.5.5 for source rebuilds.
- Godot `AudioStreamGenerator` playback with a 22050 Hz mono internal synth routed to the `Music` bus.
- Fully offline runtime generation; no network requests, accounts, telemetry, samples, or external services.

Compatibility claims apply only after each native library passes the release workflow on its target operating system. The final candidate cannot ship if any platform job is missing or failing.

## Buyer Archive

`gamestruments-<version>-godot4.zip` contains:

- `addons/gamestruments/gamestruments.gdextension`.
- Linux `.so`, Windows `.dll`, and universal macOS `.dylib` under `addons/gamestruments/bin/`.
- The same complete addon under `kit/demo/addons/gamestruments/` for a self-contained demo.
- `crates/engine` and `crates/godot`, root `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml`.
- `kit/demo/` and buyer documentation under `kit/docs/`.
- `CHANGELOG.md`, `LICENSE.md`, per-crate MIT license copies, `THIRD_PARTY_NOTICES.md`, and full dependency license texts under `licenses/`.

The archive contains no Strudel code, TypeScript authoring packages, browser Audio Lab, audio samples, pre-rendered tracks, or private build paths.

## Product Scope

The kit generates one deterministic six-section racing score from a per-title namespace, level seed, style, voice palette, traits, and generator version. Game state selects and crossfades among:

- `garage`
- `grid`
- `cruise`
- `attack`
- `final-lap`
- `victory`

The included demo exposes the exact buyer API and shipped synth. It is intentionally an integration example, not a complete game or authoring tool.

## Supported Public API

`GamestrumentsPlayer` is the only supported public class.

Exported properties:

- `project_secret: String`
- `style: String`
- `melody_voice`, `harmony_voice`, `drive_voice`, `bass_voice: String`
- `energy`, `complexity`, `brightness`, `syncopation: float`

Methods:

- `generate(seed: String) -> bool`
- `set_race_state(phase: String, intensity: float, pressure: float, final_lap: bool, finish_result: String = "none") -> bool`

Generation validates every score before playback. Failure returns `false` and emits a descriptive Godot error. State requests before successful generation also return `false`. See `kit/docs/api.md` for exact values and selection rules.

## Buyer-Reliable Behavior

- Identical inputs under the same generator version produce identical score identity and event data.
- State changes commit on bar boundaries and new sections begin at phrase bar zero.
- No generated event may be outside its section or use an invalid note, chord, drum, voice, gain, velocity, or pitch.
- Audio is synthesized in-process and requires no runtime asset loading.
- The addon releases its Godot playback resources when removed from the scene tree.

Exact bytes are not promised across generator versions. Internal Rust modules, child node names, serialized score shape, and browser Audio Lab sound are not public API.

## Explicit Non-Goals

- No general-purpose adaptive music graph or arbitrary game-state authoring.
- No game genres beyond the shipped racing state model in this release.
- No editor plugin, pattern editor, sample import, MIDI/WAV export, FMOD, or Wwise integration.
- No web, mobile, console, or Godot versions older than 4.7.
- No claim that the browser Audio Lab sounds identical to the Godot runtime.
- No guarantee that a rebuilt native library works on an untested target merely because its source compiles.

## Versioning and Release

- Archive name: `gamestruments-<version>-godot4.zip`.
- Pull requests run the three-platform release workflow with a synthetic candidate version so native regressions cannot merge unnoticed.
- A `v*` tag runs the same matrix and creates a draft GitHub release only after every native smoke test and archive verification passes.
- The exact archive SHA-256, source commit, workflow run, tested Godot version, and human acceptance evidence form the release packet.
- itch.io publication is manual and requires explicit approval of that immutable packet. CI never publishes the storefront.

## License and Support

- Gamestruments Rust crates are MIT licensed and may be used in closed-source games subject to the MIT terms.
- gdext 0.5.5 and related binding crates are MPL-2.0; attribution and source-retrieval information ship in `THIRD_PARTY_NOTICES.md`.
- No AGPL or Strudel code enters the buyer archive.
- Support is best-effort through GitHub issues for reproducible defects within the advertised environment and API.
- Refunds follow the terms presented by itch.io at purchase time.

If a claim cannot be demonstrated from the immutable archive, target-platform workflow, included demo, and buyer docs, it must not appear on the storefront.
