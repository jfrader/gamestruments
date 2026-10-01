# Gamestruments

**The first runtime product** is Gamestruments, a seed-driven, sample-free
adaptive music engine for Godot 4. Games generate a deterministic score at level load
from a per-title namespace, instrument palette, and seed, then drive
bar-quantized state changes at runtime. No samples, no Strudel, and no
authoring UI cross the game boundary.

**Audio Lab** (the browser playground in `apps/demo`) is for exploration and
validation only. It drives the same shared engine and is not part of the kit
shipped to games.

A level seed + secret creates a stable musical identity. Runtime parameters
select and crossfade pre-generated arrangements inside that identity.

## How Games Use It (the shipped product)

In a Godot 4 project:

1. Add the GDExtension (binary + `.gdextension`) under `addons/gamestruments/`.
2. Add a `GamestrumentsPlayer` node (or autoload).
3. In the inspector (or code) set:
   - `project_secret` (a stable per-title namespace, not a security credential)
   - `style` (e.g. "funk")
   - optional voice overrides (melody/harmony/drive/bass) and traits (energy, complexity, brightness, syncopation)
4. At level load call `generate(seed)` and check its boolean result.
5. During play call `set_race_state(phase, intensity, pressure, final_lap, finish_result)` as game state changes.

The engine produces the score once, keeps it, and performs musically coherent
crossovers on bar boundaries. See `docs/kit-contract.md` for the exact public
API surface and `docs/kit-opportunity.md` for scope.

Example native-engine listening-pack render:

```bash
cargo run -p gamestruments-engine --example render_listen -- /tmp/gamestruments-listen
```

## Development (Audio Lab + research)

Requires Node.js 24 for the authoring Lab.

```bash
npm ci
npm run dev
```

Open the local URL printed by Vite, choose a level seed, sound world, and
generation traits, then start audio and change race phase, speed intensity,
position pressure, and final-lap state. (Run `npm run wasm:build` once to make
the shared engine available to the lab.) The lab generates and plays through
the shared WASM engine (`crates/engine`): the same live player the Godot addon
runs, so it sounds like the kit. Runtime changes are committed on bar
boundaries and overlap through a musical crossover.

The Audition controls isolate melody or backing, jump directly to any section,
and compare two level seeds while preserving the section being reviewed.

The app has URL-addressable views:

- `#lab` runs the active adaptive score;
- `#games` organizes experiments by game interaction model;
- `#genres` compares musical interpretations and opens them in the lab.

```bash
npm run check
npm run test:e2e
```

**Pinned-build note**: the GDExtension is built against a specific gdext +
Godot API surface. Rebuild from the exact source tree + toolchain used for the
release tag. The release workflow builds and runs the extension under Godot on
Linux x86_64, Windows x86_64, and macOS with a universal arm64/x86_64 binary.

## Architecture

### Runtime (the kit product)

The in-game engine lives in Rust:

- `crates/engine` — MIT generator + transport + synth (no Strudel). Deterministic
  from namespace + seed + palette + traits. Racing, Suspense, and Adventure
  recipes.
- `crates/godot` — GDExtension wrapper exposing `GamestrumentsPlayer`.

Games never see the authoring code. See `docs/kit-contract.md` and
`crates/README.md`.

### Authoring / Research (Audio Lab only)

`packages/runtime` is the TypeScript transport used inside the Lab. `apps/demo`
is the browser playground and validation harness. Generation runs through the
shared WASM engine; the Rust engine is the single generation authority.

The first collection follows the racing music brief (see
[`docs/racing-music-brief.md`](docs/racing-music-brief.md)).

## Procedural API

Generation is owned by the Rust engine in `crates/engine`; the Audio Lab calls
it through the committed WASM build (`apps/demo/public/engine`). See
[`docs/engine-boundary.md`](docs/engine-boundary.md) for the boundary and
[`docs/procedural-generation.md`](docs/procedural-generation.md) for the
generation model. In games, generation happens inside the extension at
`generate(seed)`.

All voices are synthesized; no samples are used or redistributed.

## Scope and Limits (kit product)

- The runtime kit is Godot 4 + GDExtension + synthesized voices only.
- Release binaries target Linux x86_64, Windows x86_64, and macOS arm64/x86_64.
- No samples, no authoring UI, no Strudel, no pre-baked WAVs ship to buyers.
- See `docs/kit-contract.md` (exact inventory, API, non-goals) for the
  authoritative commercial contract. The product price is $12.99.
- Every component in this repository — runtime and Audio Lab — is MIT. There is
  no bundled authoring dependency; see `LICENSE.md`.
