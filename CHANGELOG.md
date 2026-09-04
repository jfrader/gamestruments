# Changelog

## Unreleased

### Added

- Added `tools/package_kit.sh` (deterministic buyer archive builder), produced `gamestruments-0.1.0-rc1-godot4.zip` (1 195 402 bytes, SHA-256 3881c2a7... for GURI-567; later 2 337 255 bytes, 8f705ead... after GURI-568 path+self-contained fixes), clean-copy verification + headless smoke from the extracted archive only, `docs/kit-qa-runbook.md` (Fran's author QA checklist with copy-paste steps, 4/5-arg `set_race_state` coverage, claims tick matrix), and updated `kit/docs/README.md` + this changelog. This completes the "Freeze and land the release candidate" + "Package the buyer artifact" + "Verification profiles" + "Buyer documentation" gates for GURI-567.
- Added `kit/demo/kit_demo.tscn` (and supporting `kit_demo.gd`, generator script, `project.godot` for smoke, README) that a buyer can open to prove load-time `generate(seed)` and `set_race_state` adaptive arc (garage/grid/cruise/attack/final-lap/victory). The scene is produced by a checked-in `tools/generate_demo_scene.gd` (never hand-edited .tscn). Documents integration and headless verification steps.
- Added `docs/kit-opportunity.md` (buyer, evidence, differentiator, scope,
  comparables dated 2026-09-04, risks, kill/pivot, measurement skeleton with
  owner Fran) and `docs/kit-contract.md` (supported versions, exact inventory,
  public GamestrumentsPlayer API, non-goals, archive name, MIT + gdext MPL note,
  price hypothesis pending Fran approval).
- Restructured README.md to lead with the Rust engine + GDExtension as the
  shipped kit product for games; Audio Lab is now documented as the authoring /
  research tool only. Added "How games use it", listening-pack example, and
  pinned-build note. TS sections retained but subordinated.
- Added `.github/workflows/release.yml` (workflow_dispatch + v* tags, matrix
  ubuntu / windows-mingw / macos, cargo build of gamestruments-godot --release,
  per-OS lib + sha256 artifacts). This implements the build half of the
  platform-shipping mechanism.
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

- Fixed kit demo project paths and self-contained addon (GURI-568): `kit/demo/kit_demo.tscn` now references `res://kit_demo.gd` (via generator tool that derives demo root from its script location); `tools/package_kit.sh` now stages an identical `addons/gamestruments/` copy inside `kit/demo/addons/` so opening the extracted `kit/demo/` folder directly as a Godot project succeeds (no missing-dependency errors). Updated buyer docs and QA runbook. Rebuilt RC artifact (2 337 255 bytes, SHA-256 8f705ead...).
- Fixed ObjectDB leak ("2 instances") at Godot quit for GamestrumentsPlayer.
  The `AudioStreamPlayer` and `AudioStreamGeneratorPlayback` Gd handles are no
  longer stored long-term; children are looked up transiently by name and only
  temporary handles are used. `exit_tree` stops the player and clears its
  stream. Leak count for the player now drops (background unrelated leaks may
  remain).

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
