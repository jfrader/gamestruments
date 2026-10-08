# Gamestruments

Gamestruments is a free, open source (MIT), seed-driven adaptive music engine
for Godot 4. A game generates a deterministic score at level load from a
per-title namespace, a seed, a recipe and a few traits, then drives
bar-quantized changes from gameplay. Every voice is synthesized: no samples, no
audio files, no authoring tool, no network.

- **Hear it:** <https://gamestruments.gurisitos.games> (the Audio Lab, same engine via WASM)
- **Download the Godot kit:** [GitHub releases](https://github.com/jfrader/gamestruments/releases)
  or <https://gurisitosgames.itch.io/gamestruments-godot> (free; donations welcome)
- **Recipes:** Racing, Suspense (song form) and Adventure in Godot; Folklore in
  the Audio Lab and Rust/WASM engine.

## Use it in Godot

1. Copy `addons/gamestruments/` from a release archive into your project
   (Godot 4.7.x; Linux x86_64, Windows x86_64, macOS arm64/x86_64, Web) and restart
   Godot.
2. Add a `GamestrumentsPlayer` node, set `project_secret` (a stable per-title
   namespace, not a credential), `recipe` and `style`.
3. Call `generate(seed)` at level load and check its boolean result.
4. Drive it from gameplay with `set_race_state(...)`, `set_trace_state(...)` or
   `set_adventure_state(...)`. Changes commit on bar boundaries.

Copy-paste GDScript is in [`kit/README.md`](kit/README.md) and
[`kit/docs/quickstart.md`](kit/docs/quickstart.md); the full API is in
[`kit/docs/api.md`](kit/docs/api.md). Three runnable example scenes live in
[`kit/examples/`](kit/examples/README.md).

## Build and test

Requirements: Rust (the pinned 1.94.0 toolchain is selected automatically by
`rust-toolchain.toml` through rustup) and Node.js 24.

```bash
# Rust engine and Godot extension
cargo test -p gamestruments-engine
cargo clippy -p gamestruments-engine --all-targets -- -D warnings
cargo build -p gamestruments-godot --release   # target/release/libgamestruments_godot.{so,dll,dylib}

# Audio Lab (browser) and TypeScript runtime
npm ci
npm run dev        # open the URL Vite prints
npm run check      # unit tests, typecheck, build, smoke
npm run test:e2e   # Playwright browser tests
```

The Audio Lab loads the engine from the committed WASM build in
`apps/demo/public/engine`. After changing `crates/engine`, rebuild and verify it:

```bash
npm run wasm:build
npm run wasm:verify
```

To try a locally built extension in your own project, create
`res://addons/gamestruments/`, copy `crates/godot/gamestruments.gdextension`
into it and the built library into its `bin/` folder. To check the example
project and doc snippets against a real Godot binary:

```bash
node tests/godot-package-smoke.mjs --godot <godot-binary> --library target/release/libgamestruments_godot.so
```

Render a listening pack from the native engine:

```bash
cargo run -p gamestruments-engine --example render_listen -- target/listen
```

## Repository layout

- `crates/engine`: the single generation authority: generator, transport,
  synth and recipes (Rust; also compiled to WASM for the Lab).
- `crates/godot`: GDExtension exposing `GamestrumentsPlayer`.
- `addons/gamestruments`: the Godot addon folder (descriptor, icon, docs).
- `kit/`: user docs and the example Godot project shipped in release archives.
- `apps/demo`: the browser Audio Lab.
- `packages/runtime`: TypeScript score types, validation and transport used by the Lab.
- `docs/`: design notes, recipe docs and the kit contract.
- `tools/`: WASM build, release packaging and verification scripts.
- `storefront/`: itch.io and Godot Asset Store listing copy.

See [`docs/engine-boundary.md`](docs/engine-boundary.md),
[`docs/procedural-generation.md`](docs/procedural-generation.md) and
[`docs/kit-contract.md`](docs/kit-contract.md) for the design.

## Releases

Pushing a `v*` tag runs `.github/workflows/release.yml`: it builds and smoke
tests the extension under Godot on Linux, Windows and macOS (universal), packages
`gamestruments-<version>-godot4.zip` with `tools/package_kit.sh`, verifies it
with `tools/verify_kit_archive.sh` and creates a draft GitHub release.

## Contributing

Bug reports, ideas and pull requests are welcome. See
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## License and donations

Everything in this repository is MIT licensed, including use in closed-source
and commercial games; see [`LICENSE.md`](LICENSE.md). Third-party terms
(godot-rust is MPL-2.0) are in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)
and `licenses/`.

Gamestruments is free. If it helps your game, you can support development with
an optional donation on [itch.io](https://gurisitosgames.itch.io/gamestruments-godot).
