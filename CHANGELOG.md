# Changelog

## Unreleased

### Added
- Racing now has an Extended arrangement with four new phases — Ignition, Slipstream, Redline, and Cooldown — woven into a ten-section autoplay tour. Extended is the Lab default; Original remains available with its six musical parts unchanged. Existing game integrations still default to Original.
- Added Adventure, an eight-section fantasy quest recipe (camp, explore, town, dungeon, combat, boss, sanctuary, victory). Camp, dungeon, boss, and sanctuary are 16 bars; explore, town, combat, and victory are 32, and each section develops its material across phrases rather than repeating copied halves. Styles are folk (earthy medieval folk), dark (dark medieval fantasy), and orchestral (orchestral RPG), using synthesized harp, recorder, vielle, and bell voices plus frame-drum and tambourine percussion — acoustic-inspired synthesis, not sample recordings. Godot exposes `recipe = "adventure"` and `set_adventure_state(area_phase, discovery, threat, quest_complete)`, and the Audio Lab can audition it.
- Racing and Adventure now support an opt-in `autoplay` (default off). When enabled, the engine attaches a song form that tours the recipe's sections — Racing tours garage, grid, cruise (twice), attack, final-lap, and victory, then loops from grid; Adventure tours all eight sections, then loops from explore. With autoplay off, both recipes remain state-driven and unchanged. The Lab enables autoplay for Racing and Adventure; Suspense is unchanged, and Press Play is still required.
- Suspense is composed from a 27-phase pool: the fourteen base sections plus Scan II, Breach II, Anomaly, and ten new phases (Half-Time, Sparse, Sub-Groove, Syncopated, Drive, Drum Break, False Stop, Filter Break, Harmonic Bridge, Step-Up Bridge). One seed picks a shared harmonic arc, seeded figures, kit modes and phase lengths, and a transition gesture for every join, so the selected phases play as one piece instead of stitched phrases. Versions are a contiguous family of takes of that piece: Version 1 is the first take, not a separate base derivation.
- Added Suspense, a song-form recipe for long tense sessions (first consumer: Arkhos). Music moves through intro, verse, refrain, pre-chorus, chorus, post-chorus, interlude, bridge, solo, outro, and coda instead of looping one four-bar game-state bed. The Audio Lab can audition Racing or Suspense. Godot `GamestrumentsPlayer` accepts `recipe = "suspense"` and `set_trace_state`.
- Buyer archives now include a root quickstart, machine-readable release provenance and native-library hashes, the catalog fixture needed by shipped Rust tests and examples, and exact third-party attribution metadata.
- Cross-platform Godot release candidates now include Linux x86_64, Windows x86_64, and universal macOS arm64/x86_64 libraries in one verified archive.
- Added target-native Godot runtime smoke tests, extracted-source rebuild verification, 256-seed Rust generation stress coverage, and browser end-to-end coverage for generation controls during playback.
- Audio Lab: added a prominent play/pause control to the unobstructed center of a music-reactive orbit, with beat, rhythm, melody, and bar motion; clear icon-label spacing; synchronized header controls; and a Space shortcut outside form fields.

### Changed
- Suspense now has one authority: the phase pool. The frozen Original, Extended, and Theme presets are retired; the Audio Lab and Godot offer `all-phases` and `seeded` (default). The retired `original`/`extended`/`theme` names still parse, but they resolve to the seeded pool, so scores requested by those names change. The per-preset `progress`-only Disconnect cue is gone.
- Audio Lab: the hi-hat and tambourine now carry their fixed stereo placement (hat right, tambourine left), matching the engine's per-voice pans instead of sitting unpanned as pure-noise voices.
- Suspense phases breathe more: breaks and wait states can shrink to 1–2 bars, and momentum phases can stretch to 24/32 bars, so a version's progression runs longer the way Adventure and Racing sections do instead of every strong phase resolving at 16.
- Suspense development pilot: `Scan` (verse) and `Decrypt` (solo) now develop instead of holding one texture — the layers enter one at a time, the kit drops out for the opening block, and the peak lands on an impact. The other 25 phases stay untouched while it is judged by ear.
- Audio Lab: the game-phase buttons now use the same names as the sections they cue — Exploit reads Breach, Alert reads Complication, Camp reads Trailhead Camp, Grid reads Starting Grid — so a phase is never called two things in two places. The generated ids, engine labels, presets and frozen takes are unchanged.
- Storefront and buyer docs sell Godot and HTML5 as one engine, and drop the warning that the browser mix can differ.
- Adventure: Folk now foregrounds plucked harp and recorder/fiddle phrasing, while Orchestral opens with bowed strings, longer melodic lines, and occasional woodwind answers. Warmer mode-aware harmony gives the safe phases clearer, brighter arrivals instead of accidental diminished-chord passages; the dangerous phases retain their darker contrast.
- Audio Lab: replaced the game-type toggle row with a single large selector that shows the active recipe and its description and opens a list of every recipe, so adding a recipe no longer means adding another button.
- Retired the TypeScript Studio generator and CLI pipeline. The Rust engine is now the single generation authority and the whole repository uses one recipe vocabulary (`racing`, `suspense`, `adventure`); the Audio Lab's genre/experiment copy moved into the demo. This removes the bundled Strudel/AGPL dependency and shrinks the lab bundle from ~73 kB to ~57 kB.
- Replaced the bundled racing-game demo with three independent Godot integration examples: generate/play the Suspense seeded pool, react to Racing game signals, and direct a Suspense song form. Open `kit/examples/project.godot` instead of `kit/demo/project.godot`; the existing browser Audio Lab remains the single showcase UI. The examples work offline without it.
- Audio Lab: the orbit now shows one ring per musical part in the current section (melody, harmony, bass, drums, plus recipe-specific parts like suspense's drone and cell), each pulsing and rotating from its own note events with its instrument named in the tooltip.
- Audio Lab: fixed the mobile Suspense layout — the Sound World fieldset is no longer squeezed to a sliver and the Sound World/Arrangement buttons each take a full-width row; added a regression test.
- Audio Lab: added ◀/▶ section-step buttons flanking the engine button to cue the previous/next section (wrapping at the ends), with tooltips naming the target; the engine button is slightly narrower to make room.
- Audio Lab: on phones the fixed controls bar now reserves its exact measured height (so it no longer covers the eyebrow/title at rest), and the game-type and sound-world buttons each take a full-width row instead of being squeezed side by side.
- Audio Lab: the engine button is now a live readout — it shows the current section while playing, previews the next one while waiting for the bar, and sweeps between section colours during a crossover ("Handshake → Scan"). The volume + engine row is a fixed top bar on phones.
- Audio Lab: cleaned up the Game signals hierarchy (single heading plus helper, no redundant "Game phase" title), fixed the double separator above Final lap, aligned the toggle with its label, and balanced the generator summary wrap.
- Audio Lab: fixed the mobile layout — the game-type buttons no longer overlap or clip, labels are readable, the game signals are reachable with less scrolling, and the phase buttons use a 2x2 grid for Racing and a 3x2 grid for Suspense.
- Audio Lab: moved the game signal controls (phase, intensity, pressure, final lap) out of the collapsed advanced panel to the top of Music controls, right after the level seed, so the adaptive behavior is immediately visible.
- Audio Lab: published a live HTML5 demo on itch.io at https://gurisitosgames.itch.io/gamestruments-audio-lab-demo and linked it from the kit page.
- Audio Lab: added `npm run build:itch`, which builds the lab with relative asset paths and packages `dist/gamestruments-lab-itch.zip` for itch.io HTML5 uploads.
- Audio Lab: added a master volume control (defaults to 100%, persisted in the browser) and improved the mobile layout with 44px touch targets and stacked header controls.
- Audio Lab polish: clearer control hierarchy, larger hit areas and readable labels, consistent spacing, reserved cue-feedback space, and visible queued/blending/held/loading states. Cue feedback and cancellation stay accessible while scrolling the crossover list.
- Audio Lab: Game type, Sound world and Arrangement now sit above the central player. Detailed controls and crossover sections scroll independently on desktop; expanded Game signals remain reachable, with normal page scrolling on narrow screens.
- Suspense's pool now has independent 16-bar Scan/Scan II and Breach/Breach II pairs, with developed variations rather than repeated eight-bar extensions. Lab and game APIs can hold, advance or resume the form to match gameplay. The drum grid stays continuous.
- Audio Lab: complete score-driven music-section selection replaces hard Jump actions. Cue/queued/blending states and cancellation are visible; Suspense's game signals are separated into an advanced panel. Waiting cues cannot cut an active blend, and canceled cues no longer leave stale release timers or future voices behind.
- Removed the confirmed sustained glass-cell beep from Decrypt, Other Hall and Full Breach. Anomaly, the newer melody passages, effects and drums are unchanged.
- Suspense v2 is no longer a pop song: Santaolalla-style drone + 2–3 note cell, Mr. Robot pulse/clock, static minor harmony, no snare backbeat. Form now also passes through a drop (Break) and a second inverted bridge (Other Hall) without rewriting the v2 beds.
- Removed an internal project name from buyer-visible artifacts: the racing recipe is now **Racing** (`recipe = "racing"`), and the engine module, score ids, catalog path, buyer docs, listing, and Audio Lab use that public name.
- Buyer docs now cover both recipes: `kit/docs/api.md` documents Racing and Suspense (styles, trace states, selection priority, form controls, and sections), the buyer README and quickstart add Suspense examples, and the contract, QA runbook, and listing reflect both recipes.
- Kit repositioned as **Gamestruments — Adaptive Music for Godot 4**: Gamestruments is the product; the racing recipe is simply Racing, and the demo remains Night Circuit. Storefront slug moves to `gamestruments-godot`.
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
- Rendered music now plays at a normal listening level instead of sitting roughly 20 dB below it, and every render is limited so it never reaches 0 dBFS. Renders also move from 22 050 Hz to 48 000 Hz mono.
- Native Suspense transitions now follow the pool's generated rules, matching the Audio Lab: there is no `progress`-only Disconnect rule, so only `extract`, `complete`, or `progress >= 0.95` drive the endings.
- Racing Extended: removed the piercing octave lift from the new phases and rewrote Slipstream as a single call-and-response melody with rests and resolved phrase endings. Cooldown now eases into a closing phrase instead of mechanically dropping an octave. The six original musical sections remain unchanged.
- Fixed occasional sharp drum-click spikes at certain tempos in the Audio Lab without reducing the overall mix level or changing the musical parts.
- Godot player teardown now leaves its audio child under Godot's ownership instead of manually freeing it during scene exit; repeated native example scene lifecycles are checked for hangs and leaks.
- Suspense alert/heat cues re-arm after the form leaves the cued section instead of being swallowed. There is no `progress`-only Disconnect rule; `extract`, `complete`, or `progress >= 0.95` drive the endings.
- Fixed the default Suspense opening: Handshake enters Scan on a downbeat kick, then the kick and hats stay on the grid instead of stopping after that first hit. Full rhythm dropouts are reserved for Break and endings.
- Fixed the missing Suspense kick at Scan bar 9: form entrances are prepared by the audio scheduler at their exact boundary instead of rounding a late animation frame up to bar 10. The music and two-bar bed fades are unchanged.
- Audio Lab: Suspense phase buttons now actually change beds. Boot/Scan/Exploit had no adaptive rule so clicks did nothing; Complete/Extract could pin the hold so later clicks never left.
- Audio Lab: switching from Racing to Suspense no longer dies on a missing `garage` section and keeps playing the racing score.
- Night Circuit now steers with a heading-based arcade handling model: A/D turns the car, grip straightens it when released, and the circuit wall contains it. The previous lane-strafe control read as sideways movement rather than driving.
- Fresh Godot projects without a `Music` bus now play through `Master` until a dedicated music bus is added.
- Fixed invalid generated events reaching downstream renderers or trapping WASM, and fixed Godot playback cleanup so repeated generate/play/transition/free cycles exit without extension-owned leaks.
- Audio Lab: made the orbit still while paused and replaced sharp playback jumps with slow continuous rotation and restrained musical pulses in every desktop motion mode.
- Audio Lab: removed duplicate application state introduced during the UI split so playback, regeneration, phase changes, and both play controls stay synchronized.

- Committed the prebuilt engine WASM under `apps/demo/public/engine/` so the
  Lab builds on Node-only static hosts; CI verifies the artifact is current.

- Audio Lab (`apps/demo`) now generates music via the shared Rust engine exposed through WASM (`npm run wasm:build` + `apps/demo/src/wasm-engine.ts` calling `gamestruments_score_json`). The signed-off Web Audio voices (audio-engine.ts) and `@gamestruments/runtime` transport remain for the lab frontend. DNA summary now shows score id/sections/bpm + engine note. Added `docs/engine-boundary.md`. (GURI-580)

### Added

- Added WASM facade (`crates/engine/src/wasm.rs` + Cargo cdylib), native parity reference (`crates/engine/examples/parity_ref.rs`), and Node harness (`tests/wasm-parity.mjs`) that proves byte-identical PortableScore JSON + 48000 Hz WAV output for fixed inputs between native and `wasm32-unknown-unknown` (plus cross-run determinism). Wired to CI rust job + `npm run parity:wasm`. (GURI-579)
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
- Frozen Tiny Torque `level-004` (Grid) as a catalog take for the racing recipe
  main-menu music.

- Added the first playable adaptive racing score for the racing recipe, with
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
- Added golden test for the signed-off racing catalog take (tiny-torque
  level-004 "grid" section, 3 phrases @ 48000 Hz).

### Fixed

- Fixed kit demo project paths and self-contained addon (GURI-568): `kit/demo/kit_demo.tscn` references `res://kit_demo.gd`, and packaging stages an identical addon inside the demo for direct evaluation.
- Fixed `GamestrumentsPlayer` lifecycle ownership by retaining its internal
  `AudioStreamPlayer`, stopping and detaching its stream, and explicitly freeing
  it during tree exit. Repeated runtime cycles now reject leaked-object output.

### Improved

- Each racing style now uses its own instrument kit: fusion electric
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
