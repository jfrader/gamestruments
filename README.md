# Gamestruments

**The runtime product** is a seed-driven, sample-free, MIT Rust engine +
Godot 4 GDExtension. Games generate a unique adaptive score at level load from
a per-title `project_secret` + instrument palette + seed and drive bar-quantized
state changes at runtime. No samples, no Strudel, no authoring UI cross the
game boundary.

**Audio Lab** (the TypeScript + Strudel authoring/research surface in this
repository) is for exploration, validation, and catalog work only. It is not
part of the kit shipped to games.

A level seed + secret creates a stable musical identity. Runtime parameters
select and crossfade pre-generated arrangements inside that identity.

## How Games Use It (the shipped product)

In a Godot 4 project:

1. Add the GDExtension (binary + `.gdextension`) under `addons/gamestruments/`.
2. Add a `GamestrumentsPlayer` node (or autoload).
3. In the inspector (or code) set:
   - `project_secret` (per-title secret — never a public string like the game name)
   - `style` (e.g. "funk")
   - optional voice overrides (melody/harmony/drive/bass) and traits (energy, complexity, brightness, syncopation)
4. At level load call `generate(seed)`.
5. During play call `set_race_state(phase, intensity, pressure, final_lap)` as game state changes.

The engine produces the score once, keeps it, and performs musically coherent
crossovers on bar boundaries. See `docs/kit-contract.md` for the exact public
API surface and `docs/kit-opportunity.md` for scope.

Example listening-pack render (validates parity with the Audio Lab):

```bash
cargo run -p gamestruments-engine --example render_listen -- /tmp/gamestruments-listen
```

## Development (Audio Lab + research)

Requires Node.js 24 for the authoring Lab.

```bash
npm install
npm run dev
```

Open the local URL printed by Vite, choose a level seed, sound world, and
generation traits, then start audio and change race phase, speed intensity,
position pressure, and final-lap state. Generation creates the level identity;
runtime changes are committed on bar boundaries and overlap through a musical
crossover.

The Audition controls isolate melody or backing, jump directly to any section,
and compare two level seeds while preserving the section being reviewed.

The app has URL-addressable views:

- `#lab` runs the active adaptive score;
- `#games` organizes experiments by game interaction model;
- `#genres` compares musical interpretations and opens them in the lab.

```bash
npm run check
```

**Pinned-build note**: the GDExtension is built against a specific gdext +
Godot API surface. Rebuild from the exact source tree + toolchain used for the
release tag for bit-for-bit parity with the distributed binary. Platform
binaries for Windows/macOS are produced by the release workflow but remain
pending full runtime QA (see GURI-485).

## Architecture

### Runtime (the kit product)

The in-game engine lives in Rust:

- `crates/engine` — MIT generator + transport + synth (no Strudel). Deterministic
  from `secret` + `seed` + palette + traits. Pocket Circuit recipe today.
- `crates/godot` — GDExtension wrapper exposing `GamestrumentsPlayer`.

Games never see Strudel or the authoring packages. See
`docs/kit-contract.md` and `crates/README.md`.

### Authoring / Research (Audio Lab only)

`packages/studio` (AGPL-3.0-or-later because of Strudel) is the authoring and
research surface. It is **not** shipped to players. `packages/runtime` is the
old independent TS transport used inside the Lab only.

`apps/demo` is the browser playground and validation harness.

The first collection follows Pocket Circuit's audio brief... (see
[`docs/pocket-circuit-music-brief.md`](docs/pocket-circuit-music-brief.md)).

Lantern Trail proves the recipe boundary... (unchanged).

## Procedural API (authoring / research path)

The TypeScript authoring APIs (`@gamestruments/studio`) are for the Lab and
catalog work only. They are AGPL and do not ship in games.

```ts
import { generatePocketCircuitLevel } from "@gamestruments/studio";
// ... (subordinate, kept for Lab users)
```

See the **How Games Use It** section above and `docs/kit-contract.md` for the
actual runtime contract used by shipped Godot games (Rust inside the
GDExtension).

Generation for games happens inside the extension at `generate(seed)`. See
[`docs/procedural-generation.md`](docs/procedural-generation.md) (updated for
the Rust path) and the catalog parity notes.

All voices are synthesized; no samples are used or redistributed.

## Generation CLI (Lab / catalog only)

The Node CLI is part of the authoring tooling for producing catalog takes and
validating the Rust engine. It is not required by game buyers.

```bash
npm run build:studio
node packages/studio/dist/cli.js ...
```

Game buyers use `generate(seed)` on the `GamestrumentsPlayer` node instead.

## Godot Example (portable-score transport demo)

[`examples/godot`](examples/godot) is the legacy portable-JSON consumer used
during early validation. It is **not** the kit product.

The actual kit integration for buyers is the GDExtension path described in
"How Games Use It" and `docs/kit-contract.md`. The Rust engine generates inside
the game process; no JSON export step is required for the runtime kit.

## Scope and Limits (kit product)

- The runtime kit is Godot 4 + GDExtension + synthesized voices only.
- Linux x86_64 binary is produced by the release workflow; Windows/macOS
  binaries are gated on GURI-485 runtime QA.
- No samples, no authoring UI, no Strudel, no pre-baked WAVs ship to buyers.
- See `docs/kit-contract.md` (exact inventory, API, non-goals) and
  `docs/kit-opportunity.md` (price hypothesis, risks, measurement) for the
  current commercial contract. Both price and final scope are pending Fran
  approval.
- The Audio Lab and its TypeScript packages remain AGPL for the authoring
  surface; they are deliberately kept out of the game runtime.
