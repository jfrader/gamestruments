# Procedural Generation

## Identity And Adaptation

Gamestruments separates level generation from runtime adaptation.

- A level seed creates a stable musical identity.
- A style selects the sound-world instrument pool and lead voice.
- The timbre domain then mixes pad, drive, and lift voices from that pool.
- Generation traits shape tempo, density, register, harmony, and syncopation.
- Runtime state selects arrangements derived from that same identity.

Changing runtime intensity or pressure does not reroll the composition. A game
generates once while loading a level, keeps the resulting portable score, and
passes changing game state to the runtime transport.

## Deterministic Domains

Each recipe derives independent named sub-seeds for harmony, motif, rhythm,
timbre, arrangement, and ornaments. The domains make generation repeatable and
keep changes in one generator subsystem from consuming random values intended
for another subsystem.

Racing sections share the generated key, progression, eighth-note pulse, and
recognizable motif contour. Garage, grid, cruise, attack, final lap, and victory
immediately reshape that material with different onset masks, density, register,
articulation, bass motion, and percussion. A newly activated section starts at
phrase bar zero even when the crossover begins on a later global bar.

Racing has two arrangements: `original` (default) and `extended`. Extended
generates the six original sections byte-for-byte, then adds four new sections
harvested from the same piece and palette — `ignition` (8 bars), `slipstream`
(16 bars), `redline` (16 bars), and `cooldown` (8 bars) — for a ten-section
order: garage, ignition, grid, cruise, slipstream, attack, redline, final-lap,
victory, cooldown. The new sections are cueable and appear in the autoplay tour
but carry no new adaptive rules. When `autoplay` is set, the attached song form
tours the sections: `original` tours garage, grid, cruise (×2), attack,
final-lap, victory, then loops from grid; `extended` tours all ten sections once
each, then loops from grid. With autoplay off, both arrangements remain
state-driven and unchanged, and both wrappers leave the form empty until the
autoplay adapter is explicitly enabled.

Suspense is a third interaction model for long tense sessions (Arkhos and
similar infiltration games). Form still auto-advances, but the writing is
texture, not pop: a drone, a 2–3 note cell (Santaolalla), and a machine pulse
(Mr. Robot). The 38-phase pool is the only authority: one seed picks a shared
harmonic arc, seeded figure/drum/length choices, a transition gesture per join,
and — for `seeded` — the form itself. Styles are `terminal`, `cipher`, and
`noir`. Generation traits are tension, heat, mystery, and pulse. Runtime state is
`tracePhase`, heat, focus, and progress.

Adventure is a third interaction model for fantasy and exploration games. In
gameplay mode it is area-selected: eight sections (`camp`, `explore`, `town`,
`dungeon`, `combat`, `boss`, `sanctuary`, `victory`) each carry a stable modal
identity that survives indefinitely on loop. `camp`, `dungeon`, `boss`, and
`sanctuary` are 16 bars; `explore`, `town`, `combat`, and `victory` are 32, and
each section develops its material across phrases rather than repeating copied
halves. Church modes, open-fifth drones, and plucked/bowed/breath voices
replace the racing palette. Styles are `folk` (earthy medieval folk), `dark`
(dark medieval fantasy), and `orchestral` (orchestral RPG). Generation traits
are wonder, danger, mystery, and motion. Runtime state is `areaPhase` plus
discovery, threat, and quest progress; quest completion always wins, and a high
threat escalates combat into `boss`. The voices are harp, recorder, vielle, and
bell plus frame-drum and tambourine percussion — synthesized, acoustic-inspired
timbres rather than sample recordings. The seeded pool adds a combat set
(`skirmish`, `assault`, `chase`, riding the peak band with combat/boss) and a
happiness set (`festival`, `reunion`, `dawn`, riding the groove and break bands
with town/sanctuary), so the seeded composer can interleave brighter and more
driving material into the same quest. When `autoplay` is set, an attached song
form tours the eight sections and loops from `explore`.

## API

Generation is owned by the Rust engine in `crates/engine`. The Audio Lab
(`apps/demo`) calls it through the committed WASM build
(`apps/demo/public/engine/gamestruments_engine.wasm`, rebuilt with
`npm run wasm:build`) and plays it with the engine's live player. Games generate
inside the GDExtension at `generate(seed)`. See `docs/engine-boundary.md` for
the boundary and `crates/engine/src/<recipe>.rs` for each deterministic
generator.

Traits are normalized to `0..1`. Omitted traits use recipe defaults. The
portable score contains expanded events only: no traits, random generator, or
domain seeds cross the runtime boundary.

## Versioning

Deterministic output is stable for the tuple
`(secret, seed, style, palette, traits, generator version)`. The secret acts as
a per-title namespace so different games never collide on the same default
sound even with identical seeds.

The Rust engine is the single generation authority. The retired TypeScript
authoring generator and its CLI/manifests are no longer part of the repository.

## Catalog level-004 (parity test)

The frozen Tiny Torque take `level-004` (Grid section) under
`catalog/racing/tiny-torque-level-004/` is the reference for engine
parity. Empty secret + seed "level-004" + funk style + the recorded traits
reproduces exactly `racing-generated-v1-9-0-7864ec71` (see the golden
render test and `render_listen` example in `crates/engine`).

Shipped catalog takes remain as validation fixtures. Games using the kit call
`generate(seed)` at runtime rather than loading the JSON.
