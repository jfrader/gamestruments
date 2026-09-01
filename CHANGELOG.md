# Changelog

## Unreleased

### Added

- Started a MIT Rust engine (`crates/engine`) and Godot 4 GDExtension
  (`crates/godot`) so games generate and play music at level load from
  `gameId` + seed, without Strudel or a WAV library.
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
