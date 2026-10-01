# Prototype Contract

## Promise

Generate deterministic instrumental patterns from a seed and parameters, keep a
portable, engine-neutral score, and transition between feelings at beat-aware
boundaries using game state. Each recipe maps the four generic generation traits
onto its own vocabulary and exposes its own runtime state:

- **Racing** — north-star racing loops: race phase, speed intensity, position
  pressure, final lap, and finish result.
- **Suspense** — a song form plus trace phase, heat, focus, and progress.
- **Adventure** — an eight-section fantasy quest arc (16-bar camp, dungeon,
  boss, and sanctuary; 32-bar explore, town, combat, and victory), plus
  discovery, threat, and quest progress.

## Included

- one Rust generation authority (`crates/engine`) exposed to games through the
  Godot GDExtension and to the Lab through a committed WASM build;
- a portable, versioned score and event model;
- an independent deterministic transition planner;
- bar-quantized crossover with overlapping gain envelopes;
- a playable browser Audio Lab demonstration;
- standalone Lab, Game Types, and Genres browser views;
- four original racing scores covering garage through victory;
- seeded alternate takes for procedural phrases;
- deterministic generation with independent musical sub-seeds;
- Racing, Suspense, and Adventure recipes;
- synthesized voices only, with no samples;
- live controls for seed, style, and the four generation traits;
- native/WASM parity coverage, many-seed stress coverage, and automated runtime
  transition tests.

## Compatibility

- Generation and development: Node.js 24 and modern evergreen browsers.
- Runtime contract: plain JSON-compatible data and a Godot 4.7 GDScript consumer.
- Audio: the Rust engine renders both the game and the browser Lab.
- Reproducibility: seed stability is scoped to a declared generator version.

## Constraints

- Instrumental only: no voice, vocals, speech, lyrics, or voice models.
- Offline after dependencies are installed: no cloud generation, accounts,
  telemetry, analytics, or runtime network requests.
- Everything in the repository is MIT; there is no bundled Strudel/authoring
  dependency and no TypeScript generator pipeline.

## Non-Goals

- a DAW, notation compatibility, or a plugin host;
- AI generation or sample-library redistribution;
- multiplayer clock synchronization or sample-accurate native DSP;
- a production-ready commercial toolkit or final support policy.

Future publishing, packaging, pricing, and open-core/commercial terms remain
undecided. Racing, Suspense, and Adventure demonstrate that game types
enter through recipe and score boundaries rather than one hard-coded product
shape.
