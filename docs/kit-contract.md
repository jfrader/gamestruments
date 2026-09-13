# Gamestruments — Adaptive Music for Godot 4: Product Contract

**Status:** approved scope; release remains gated on automated platform checks, independent buyer testing, human listening, and explicit publication approval.
**Standard price:** $12.99 (set 2026-09-10; revisitable after launch).

This is the authoritative buyer contract for the first Gamestruments runtime kit.

## Supported Environment

- Godot 4.7.x via GDExtension. Future Godot minor releases are not implied. Tested exactly against CI version 4.7.2.
- Linux x86_64 (built on Ubuntu 24.04; older distributions are not claimed), Windows x86_64, and macOS arm64/x86_64.
- Rust 1.94.0 and gdext 0.5.5 for source rebuilds.
- Godot `AudioStreamGenerator` playback with a 22050 Hz mono internal synth routed to `Music` when that bus exists and otherwise to `Master`.
- Fully offline runtime generation; no network requests, accounts, telemetry, samples, or external services.

Compatibility claims apply only after each native library passes the release workflow on its target operating system. The final candidate cannot ship if any platform job is missing or failing.

## Buyer Archive

`gamestruments-<version>-godot4.zip` contains:

- `addons/gamestruments/gamestruments.gdextension`.
- Linux `.so`, Windows `.dll`, and universal macOS `.dylib` under `addons/gamestruments/bin/`.
- The same complete addon under `kit/examples/addons/gamestruments/` for a self-contained examples project.
- `crates/engine` and `crates/godot`, the catalog fixture required by shipped tests and examples, root `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml`.
- `kit/examples/` and buyer documentation under `kit/docs/`.
- Root `README.md`, `RELEASE-MANIFEST.json`, and `CHANGELOG.md`.
- `LICENSE.md`, per-crate MIT license copies, `THIRD_PARTY_NOTICES.md`, exact dependency inventory, required attribution, and dependency license texts under `licenses/`.

The archive contains no Strudel code, TypeScript authoring packages, browser Audio Lab, audio samples, pre-rendered tracks, or private build paths.

## Product Scope

The kit generates deterministic scores from three shipped recipes, each from a per-title namespace, level seed, style, voice palette (Racing), traits, and generator version:

- **Racing:** six sections — `garage`, `grid`, `cruise`, `attack`, `final-lap`, `victory` (original); `extended` adds four more (ignition/slipstream/redline/cooldown) — driven by `set_race_state`. Native default arrangement original.
- **Suspense (song-form):** fourteen base sections from `intro` (Handshake) to `coda` (Closed Session), driven by `set_trace_state`; the `extended` arrangement adds `scan-ii`, `breach-ii`, and `anomaly`; `theme` is additive title bed. Gameplay can hold, advance, or cue the form. Autoplay ignored.
- **Adventure:** eight sections (camp, explore, town, dungeon, combat, boss, sanctuary, victory) driven by `set_adventure_state`. 3 styles. 8 real phases.

The `kit/examples/` project is three independent reference scenes, not a
playable game. `01-playback` generates and plays a Suspense Theme title bed,
`02-game-signals` maps simulated race events to `set_race_state` requests (Original Racing),
and `03-song-form` exercises the Suspense form controls and trace events. They share
one addon copy and need no browser or network. The browser Audio Lab is a
separate preview, not the examples' UI or runtime. See `kit/examples/README.md`. Adventure
integration uses `set_adventure_state` (documented in API).

## Supported Public API

`GamestrumentsPlayer` is the only supported public class.

Exported properties:

- `project_secret: String`
- `recipe: String` — `racing` (default), `suspense`, or `adventure`
- `arrangement: String` — Racing: `original` (default) or `extended`; Suspense: `original` (default), `extended`, or `theme`; ignored by Adventure
- `autoplay: bool` — Racing and Adventure (default `false`); when true attaches form tour (arrangement tour, not audio autostart). Ignored by Suspense. Native default false.
- `style: String` — per recipe: Racing `fusion`, `neon`, `funk`, `chip`; Suspense `terminal`, `cipher`, `noir`; Adventure `folk`, `dark`, `orchestral`
- `melody_voice`, `harmony_voice`, `drive_voice`, `bass_voice: String` — Racing only
- `energy`, `complexity`, `brightness`, `syncopation: float` — read as energy/complexity/brightness/syncopation by Racing, as tension/heat/mystery/pulse by Suspense, as danger/mystery/wonder/motion by Adventure

Methods:

- `generate(seed: String) -> bool`
- `set_race_state(phase: String, intensity: float, pressure: float, final_lap: bool, finish_result: String = "none") -> bool` — Racing
- `set_trace_state(phase: String, heat: float, focus: float, progress: float) -> bool` — Suspense
- `set_adventure_state(area_phase: String, discovery: float, threat: float, quest_complete: bool) -> bool` — Adventure
- `cue_section(section: String) -> bool`
- `set_form_hold(held: bool) -> bool`
- `advance_form() -> bool`
- `is_form_held() -> bool`
- `get_current_section() -> String`

Form controls (`cue_section` etc.) are valid for any score with a form (Suspense always; Racing/Adventure only when `autoplay` enabled at generate).

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
- No game genres beyond the shipped Racing, Suspense and Adventure state models in this release.
- No editor plugin, pattern editor, sample import, MIDI/WAV export, FMOD, or Wwise integration.
- No web, mobile, console, or Godot versions other than 4.7.x.
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
- Support is best-effort through the public comments section on the itch.io product page for reproducible defects within the advertised environment and API. Purchase-specific or private matters use itch.io's purchase-support flow.
- Refunds follow the terms presented by itch.io at purchase time.

If a claim cannot be demonstrated from the immutable archive, target-platform workflow, included examples, and buyer docs, it must not appear on the storefront.
