# Prototype Contract

## Promise

Generate deterministic instrumental patterns from a level seed and parameters,
export engine-neutral musical events, and transition between feelings at
beat-aware boundaries using game state. Pocket Circuit uses style, energy,
complexity, brightness, and syncopation for generation, then race phase, speed
intensity, position pressure, final-lap state, and finish result at runtime.
Lantern Trail independently uses wonder, danger, mystery, and motion.

## Included

- Strudel-backed authoring adapter and example score;
- portable, versioned score and event types;
- independent deterministic transition planner;
- bar-quantized crossover with overlapping gain envelopes;
- playable browser demonstration;
- standalone Lab, Game Types, and Genres browser views;
- four original racing scores covering garage through victory;
- seeded alternate takes for procedural phrases;
- deterministic level generation with independent musical sub-seeds;
- Pocket Circuit racing and Lantern Trail adventure generation recipes;
- a Node CLI with recipe-specific traits, versioned manifests, and SHA-256 score
  checksums;
- publishable runtime and Studio ESM packages with TypeScript declarations;
- a standalone Godot example that consumes exported JSON without Strudel;
- live controls for level seed, style, energy, complexity, brightness, and
  syncopation;
- golden fixtures, many-seed stress coverage, and automated runtime transition
  tests.

## Compatibility

- Authoring and development: Node.js 24 and modern evergreen browsers.
- Runtime contract: plain JSON-compatible data, TypeScript reference code, and a
  Godot 4.7 GDScript consumer example.
- Audio: browser Web Audio demonstration only in this prototype.
- Reproducibility: seed stability is scoped to a declared generator version.

## Constraints

- Instrumental only: no voice, vocals, speech, lyrics, or voice models.
- Offline after dependencies are installed: no cloud generation, accounts,
  telemetry, analytics, or runtime network requests.
- The authoring/demo side is AGPL-3.0-or-later because it uses Strudel.
- The independent runtime reference is MIT and must not import Strudel.

## Non-Goals

- production native audio adapters, a DAW, or notation compatibility beyond
  supported Strudel APIs;
- AI generation, full synthesis, sample-library redistribution;
- multiplayer clock synchronization or sample-accurate native DSP;
- a production-ready commercial toolkit or final support policy.

Future publishing, packaging, pricing, and open-core/commercial terms remain
undecided. Pocket Circuit and Lantern Trail demonstrate that game types and
genres enter through recipe and score boundaries rather than one hard-coded
product shape.
