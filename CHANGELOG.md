# Changelog

## Unreleased

### Added
- Buyer archives now include a root quickstart, machine-readable release provenance and native-library hashes, the catalog fixture needed by shipped Rust tests and examples, and exact third-party attribution metadata.
- Cross-platform Godot release candidates now include Linux x86_64, Windows x86_64, and universal macOS arm64/x86_64 libraries in one verified archive.
- Added target-native Godot runtime smoke tests, extracted-source rebuild verification, 256-seed Rust generation stress coverage, and browser end-to-end coverage for generation controls during playback.
- Audio Lab: added a prominent play/pause control to the unobstructed center of a music-reactive orbit, with beat, rhythm, melody, and bar motion; clear icon-label spacing; synchronized header controls; and a Space shortcut outside form fields.

### Changed
- Buyer archive `README.md` is rewritten in a friendlier, task-first voice and links the browser preview at https://gamestruments.gurisitos.games (Audio Lab: same generator, browser audio layer).
- Standard price lowered to $12.99 (set 2026-09-10; revisitable after launch).
- Buyer archive `README.md` now leads with a copy-paste integration guide: install the addon, configure `GamestrumentsPlayer`, call `generate(seed)`, and drive `set_race_state(...)`. The playable demo is documented as an optional reference, and the storefront listing moves usage steps above the feature list.
- Replaced the Godot generator panel with a playable four-circuit race series against a rival, grip-assisted steering, boost, and music responding automatically to gameplay. Each circuit configures its own shipped style and seed in the garage, so all four styles (neon, pocket funk, fusion, micro motor) are audible without editing code. A first-run intro card states that the music adapts to gameplay. Includes a separate music integration script, pause/restart, and gameplay regression checks.
- Extracted archive verification now tests the complete shipped Rust source and validates native formats, architectures, duplicate addon copies, licenses, provenance, and binary path hygiene.
- `GamestrumentsPlayer.generate` and `set_race_state` now return success booleans with descriptive Godot errors, and generator version `1.10.1` validates every score before exposing it to native or WASM callers.
- Repositioned the first kit as adaptive racing music for Godot 4.7.x and reconciled buyer docs with its six-section API, mono runtime sound, desktop platform support, licensing, and manual publication gates.
- Audio Lab: moved score and phase status above the orbit so it never competes with playback, and placed the transport first on narrow screens.
- Audio Lab: restored idle orbit motion and fixed score, seed, comparison, and generation controls that could fail on out-of-range generated melody events.

### Fixed
- Night Circuit now steers with a heading-based arcade handling model: A/D turns the car, grip straightens it when released, and the circuit wall contains it. The previous lane-strafe control read as sideways movement rather than driving.
- Fresh Godot projects without a `Music` bus now play through `Master` until a dedicated music bus is added.
- Fixed invalid generated events reaching downstream renderers or trapping WASM, and fixed Godot playback cleanup so repeated generate/play/transition/free cycles exit without extension-owned leaks.
- Audio Lab: made the orbit still while paused and replaced sharp playback jumps with slow continuous rotation and restrained musical pulses in every desktop motion mode.
- Audio Lab: removed duplicate application state introduced during the UI split so playback, regeneration, phase changes, and both play controls stay synchronized.

- Committed the prebuilt engine WASM under `apps/demo/public/engine/` so the
  Lab builds on Node-only static hosts; CI verifies the artifact is current.

- Audio Lab (`apps/demo`) now generates music via the shared Rust engine exposed through WASM (`npm run wasm:build` + `apps/demo/src/wasm-engine.ts` calling `gamestruments_score_json`). The signed-off Web Audio voices (audio-engine.ts) and `@gamestruments/runtime` transport remain for the lab frontend. DNA summary now shows score id/sections/bpm + engine note. Added `docs/engine-boundary.md`. (GURI-580)

### Added

- Added WASM facade (`crates/engine/src/wasm.rs` + Cargo cdylib), native parity reference (`crates/engine/examples/parity_ref.rs`), and Node harness (`tests/wasm-parity.mjs`) that proves byte-identical PortableScore JSON + 22050 Hz WAV output for fixed inputs between native and `wasm32-unknown-unknown` (plus cross-run determinism). Wired to CI rust job + `npm run parity:wasm`. (GURI-579)
- Added `tools/package_kit.sh`, archive verification, a self-contained demo, and an author QA runbook for deterministic buyer release candidates. The retired Linux-only rc1 is not a releasable artifact.
- Added `kit/demo/kit_demo.tscn` (and supporting `kit_demo.gd`, generator script, `project.godot` for smoke, README) that a buyer can open to prove load-time `generate(seed)` and `set_race_state` adaptive arc (garage/grid/cruise/attack/final-lap/victory). The scene is produced by a checked-in `tools/generate_demo_scene.gd` (never hand-edited .tscn). Documents integration and headless verification steps.
- Added `docs/kit-opportunity.md` (buyer, evidence, differentiator, scope,
  comparables dated 2026-09-04, risks, kill/pivot, measurement skeleton with
  owner Fran) and `docs/kit-contract.md` (supported versions, exact inventory,
  public GamestrumentsPlayer API, non-goals, archive name, MIT + gdext MPL note,
  approved $24.99 standard price and manual release gates).
- Restructured README.md to lead with the Rust engine + GDExtension as the
  shipped kit product for games; Audio Lab is now documented as the authoring /
  research tool only. Added "How games use it", listening-pack example, and
  pinned-build note. TS sections retained but subordinated.
- Added `.github/workflows/release.yml` for target-native Linux, Windows MSVC,
  and universal macOS builds, Godot runtime smoke, one verified archive, and a
  draft GitHub release from `v*` tags.
- Updated `docs/procedural-generation.md`: catalog level-004 paragraph now
  points at the engine parity test; gameId references updated to secret +
  palette + seed; added reserved-take reproducibility note (empty secret +
  level-004 + funk reproduces the v1-9-0 catalog id).
- (changelog entry for the kit docs + workflow work)

- Added `docs/kit-plan.md` (milestones in gate order, acceptance criteria, claim-to-evidence matrix, verification profiles with N/A reasons, clean-room buyer task list, risk register, post-launch measurement contract skeleton) and buyer documentation under `kit/docs/` (`README.md`, `quickstart.md`, `api.md`, `limitations.md`, `troubleshooting.md`).
- Pinned pricing decision in `docs/kit-contract.md`: Standard price $24.99 (approved by Fran 2026-09-04). Updated inventory section to list buyer doc paths. No launch discounts.
- All buyer-facing claims kept strictly literal to behavior in the shipped `GamestrumentsPlayer` (exports, generate, set_race_state) and engine generator (styles, sections, determinism, mono synth, zero samples).

- Started a MIT Rust engine (`crates/engine`) and Godot 4 GDExtension
  (`crates/godot`) so games generate and play music at level load from a
  project secret, instrument palette, and seed, without Strudel or a WAV
  library.
- Frozen Tiny Torque `level-004` (Grid) as a catalog take for Pocket Circuit
  main-menu music.

- Added the first playable adaptive racing score for Pocket Circuit, with
  bar-quantized crossovers between garage, grid, race flow, position pressure,
  final lap, and victory states.
- Added a Strudel-backed authoring exporter and an independent portable runtime
  for deterministic game-state transitions.
- Added four racing sound worlds, seeded alternate takes, and standalone Lab,
  Game Types, and Genres views for exploring the experiment catalog.
- Added state-aware four-bar lead motifs, including a syncopated electric-piano
  hook for Countertop Velocity, while preserving each sound world's identity.
- Added melody and rhythm soloing, direct section jumps, and seeded take A/B
  comparison for faster musical review.
- Added a Stop engine control that silences playback and resumes from the same
  audition section when restarted.
- Added deterministic procedural level generation from a seed, sound-world
  style, energy, complexity, brightness, and syncopation parameters.
- Added Lantern Trail, an independent procedural adventure recipe spanning camp,
  exploration, clues, danger, sanctuary, and quest completion.
- Added a generation CLI for both recipes with normalized inputs, named domain
  seeds, versioned manifests, and compact-score SHA-256 checksums.
- Added publishable runtime and Studio ESM packages with TypeScript declarations
  and verified external-consumer imports.
- Added a standalone Godot 4.7 example that loads exported JSON, evaluates
  adaptive rules, and visualizes bar-quantized crossfades without Strudel.
- Added live seed generation, next-seed navigation, generated musical-DNA
  summaries, and seed A/B comparison to the Audio Lab.
- Added offline WAV renderer (`gamestruments_engine::render::render_wav`) to
  `crates/engine` (no Godot dep): renders a PortableScore section looped N
  phrases through the Synth with lab-matching outer seam fade, emitting valid
  16-bit mono PCM RIFF WAV.
- Added golden test for the signed-off Pocket Circuit catalog take (tiny-torque
  level-004 "grid" section, 3 phrases @ 22050 Hz).

### Fixed

- Fixed kit demo project paths and self-contained addon (GURI-568): `kit/demo/kit_demo.tscn` references `res://kit_demo.gd`, and packaging stages an identical addon inside the demo for direct evaluation.
- Fixed `GamestrumentsPlayer` lifecycle ownership by retaining its internal
  `AudioStreamPlayer`, stopping and detaching its stream, and explicitly freeing
  it during tree exit. Repeated runtime cycles now reject leaked-object output.

### Improved

- Each Pocket Circuit style now uses its own instrument kit: fusion electric
  piano and organ, neon supersaw with echo, funk pluck, and bitcrushed chip
  squares with triangle bass.
- Tiny Torque Race Flow now uses the denser former Grid groove; Grid uses the
  sparser former Race loop. The frozen level-004 catalog take is unchanged.
- Non-chip instruments now use analog-style tails, darker filters, sine piano
  bodies, triangle bass, and noise hats; Chip remains the 8-bit voice.
- Countertop Velocity now drives faster fusion with a brighter finish.
- Restored immediately distinct phase arrangements: Garage is sparse and low,
  Grid builds anticipation, Race establishes the groove, Position Fight adds
  pressure, Final Lap reaches the density and register peak, and Finish releases
  into a broad cadence.
- Fresh section crossfades now begin at phrase bar zero, preserving four-bar
  builds while still committing changes on the next global bar.
- Tonal layers use linear crossfades while melody and percussion retain
  equal-power curves, reducing correlated-layer buildup during transitions.
- Final-lap and danger harmony preserve each generated mode while adding leading
  tone and dominant pressure, avoiding clashes during overlapping sections.

### Fixed

- Restored Race Flow, Position Fight, and Final Lap to held-pad grooves: one
  chord per bar, a two-note high hook, half-note bass, and a kick-hat-snare kit,
  instead of stacked stabs and drums on every eighth. Final Lap lift is now a
  held high layer rather than a dense arpeggio.
- Removed fusion swing so every lane shares one eighth-note grid; delayed
  melody and hat offbeats no longer flam against straight kick and snare.
- Brightened the Fusion finish with a higher victory melody and a glassy
  four-bar sparkle figure.
- Corrected pending musical crossovers so a newer game-state parameter change
  can replace or cancel a transition before its quantized start.
- Prevented duplicate downbeat kicks from flattening generated Grid and Race
  grooves for some seeds.
- Prevented raised-seventh treatments from wrapping an existing leading tone
  back to the tonic.
