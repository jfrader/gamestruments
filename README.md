# Gamestruments

Gamestruments is a seed-driven, sample-free adaptive music engine for Godot 4.
A game generates a deterministic score at level load from a per-title
namespace, instrument palette and seed, then drives bar-quantized state changes
at runtime. Only the GDExtension and its synthesized voices cross the game
boundary: no samples, no Strudel, no authoring UI.

A level seed plus the title namespace gives a stable musical identity. Runtime
parameters select and crossfade pre-generated arrangements inside that identity.

The Audio Lab in `apps/demo` is a browser playground for exploration and
validation. It drives the same engine and is not part of the kit shipped to
games.

## How games use it

In a Godot 4 project:

1. Add the GDExtension (binary + `.gdextension`) under `addons/gamestruments/`.
2. Add a `GamestrumentsPlayer` node (or autoload).
3. In the inspector (or code) set:
   - `project_secret` (a stable per-title namespace, not a security credential)
   - `recipe` (`racing`, `suspense` or `adventure`) and `style` (e.g. "funk")
   - optional voice overrides (melody/harmony/drive/bass) and traits (energy, complexity, brightness, syncopation)
4. At level load call `generate(seed)` and check its boolean result.
5. During play call `set_race_state` (Racing), `set_trace_state` plus the form
   controls (Suspense) or `set_adventure_state` (Adventure) as game state changes.

The engine produces the score once, keeps it, and crosses over between
sections on bar boundaries. `docs/kit-contract.md` has the exact public API and
`docs/kit-opportunity.md` the scope.

Render a listening pack from the native engine:

```bash
cargo run -p gamestruments-engine --example render_listen -- /tmp/gamestruments-listen
```

## Development

The Audio Lab needs Node.js 24.

```bash
npm ci
npm run dev
```

Open the URL Vite prints, choose a level seed, sound world and generation
traits, start audio, then change race phase, speed intensity, position pressure
and final-lap state. The lab plays the engine's own live player through the
committed WASM build (`apps/demo/public/engine`), so it sounds like the Godot
kit; rebuild it with `npm run wasm:build` after engine changes. Runtime changes
commit on bar boundaries and overlap through a crossover.

The Audition controls isolate melody or backing, jump to any section, and
compare two level seeds while keeping the section under review.

Views:

- `#lab` runs the active adaptive score;
- `#games` organizes experiments by game interaction model;
- `#genres` compares musical interpretations and opens them in the lab.

```bash
npm run check
npm run test:e2e
```

The GDExtension is built against a specific gdext and Godot API surface, so
rebuild it from the exact source tree and toolchain of the release tag. The
release workflow builds and runs the extension under Godot on Linux x86_64,
Windows x86_64 and macOS with a universal arm64/x86_64 binary.

## Architecture

Runtime (the kit), in Rust:

- `crates/engine`: generator, transport and synth. Deterministic from
  namespace, seed, palette and traits. Racing, Suspense and Adventure recipes.
- `crates/godot`: GDExtension wrapper exposing `GamestrumentsPlayer`.

Audio Lab only: `packages/runtime` (`@gamestruments/runtime`) holds the
TypeScript score types, validation and transport; `apps/demo` is the browser
playground. The Rust engine is the single generation authority (WASM in the
Lab, the extension's `generate(seed)` in games). See
`docs/kit-contract.md`, `crates/README.md`,
[`docs/engine-boundary.md`](docs/engine-boundary.md) and
[`docs/procedural-generation.md`](docs/procedural-generation.md).

The first collection follows the racing music brief in
[`docs/racing-music-brief.md`](docs/racing-music-brief.md).

All voices are synthesized; no samples are used or redistributed.

## Scope and licence

- The runtime kit is Godot 4 + GDExtension + synthesized voices only.
- Release binaries target Linux x86_64, Windows x86_64 and macOS arm64/x86_64.
- No samples, authoring UI, Strudel or pre-baked WAVs ship to buyers.
- `docs/kit-contract.md` is the commercial contract (inventory, API,
  non-goals). The product price is $12.99.
- Everything in this repository, runtime and Audio Lab, is MIT, with no bundled
  authoring dependency; see `LICENSE.md`.
