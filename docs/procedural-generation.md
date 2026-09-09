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

Pocket Circuit sections share the generated key, progression, eighth-note
pulse, and recognizable motif contour. Garage, grid, cruise, attack, final lap,
and victory immediately reshape that material with different onset masks,
density, register, articulation, bass motion, and percussion. A newly activated
section starts at phrase bar zero even when the crossover begins on a later
global bar.

Lantern Trail uses the same domain contract with an independent adventure
vocabulary. Camp, explore, clue, danger, sanctuary, and quest-complete sections
share one generated identity without reusing Pocket Circuit's traits or rules.

Suspense is a third interaction model for long tense sessions (Arkhos and
similar infiltration games). Form still auto-advances, but the writing is
texture, not pop: a drone, a 2–3 note cell (Santaolalla), and a machine pulse
(Mr. Robot). Harmony stays on one minor sonority. Outro and coda are hold
interrupts. Alert/heat cues the bridge once. Styles are `terminal`, `cipher`,
and `noir`. Generation traits are tension, heat, mystery, and pulse. Runtime
state is `tracePhase`, heat, focus, and progress.

## API

The Audio Lab (`apps/demo`) now uses the shared WASM engine for generation
(`generateScore` wrapper around `gamestruments_score_json`); the Web Audio
render stage remains local. Authoring still uses the TS studio for catalog work.

```ts
import {
  generatePocketCircuitLevel,
  POCKET_CIRCUIT_GENERATOR_VERSION,
} from "@gamestruments/studio";

const generated = generatePocketCircuitLevel({
  seed: "world-3/level-12",
  style: "neon",
  traits: {
    energy: 0.72,
    complexity: 0.48,
    brightness: 0.82,
    syncopation: 0.35,
  },
});

console.log(POCKET_CIRCUIT_GENERATOR_VERSION);
console.log(generated.dna);
saveForRuntime(generated.portableScore);
```

```ts
import {
  generateLanternTrailAdventure,
  LANTERN_TRAIL_GENERATOR_VERSION,
} from "@gamestruments/studio";

const generated = generateLanternTrailAdventure({
  seed: "forest-7",
  traits: {
    wonder: 0.8,
    danger: 0.45,
    mystery: 0.7,
    motion: 0.55,
  },
});

console.log(LANTERN_TRAIL_GENERATOR_VERSION);
saveForRuntime(generated.portableScore);
```

Traits are normalized to `0..1`. Omitted traits use recipe defaults. String and
safe-integer seeds are supported and intentionally occupy different namespaces.

The result contains authoring data and musical DNA for tooling, plus a plain
`PortableScore` for the game. The portable score contains expanded events only:
no Strudel patterns, traits, random generator, or domain seeds cross the runtime
boundary.

## CLI And Manifests

The built `gamestruments-generate` CLI accepts
`--recipe pocket-circuit|lantern-trail`. Recipe-specific flags are validated so
traits cannot silently cross recipe boundaries. Scores may be written to a file
or standard output; optional manifests contain the normalized generation tuple,
named domain seeds, score identity, and a SHA-256 checksum of compact score JSON.

Manifest helpers are exported from `@gamestruments/studio/manifest` for Node
tooling. The portable runtime does not depend on the manifest or Node crypto.

## Versioning

Deterministic output for the **Rust runtime path** (the kit) is stable for the
tuple `(secret, seed, style, palette, traits, generator version)`. The secret
acts as a per-title namespace so different games never collide on the same
default sound even with identical seeds.

The old TS/JSON portable path (Lab + legacy examples) used `gameId` + seed
terminology; that has become `secret` + `palette` + `seed` in the shipping
GDExtension contract.

## Catalog level-004 (parity test)

The frozen Tiny Torque take `level-004` (Grid section) under
`catalog/pocket-circuit/tiny-torque-level-004/` is the reference for engine
parity. Empty secret + seed "level-004" + funk style + the recorded traits
reproduces exactly `pocket-circuit-generated-v1-9-0-7864ec71` (see the golden
render test and `render_listen` example in `crates/engine`).

Shipped catalog takes remain as validation fixtures. Games using the kit call
`generate(seed)` at runtime rather than loading the JSON.
