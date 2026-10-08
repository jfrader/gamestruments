# Contributing

Thanks for helping. Keep changes small and focused.

## Before you open a pull request

1. Open an issue first for larger changes so we can agree on the approach.
2. Run the checks (Node.js 24, Rust via rustup):

   ```bash
   npm ci
   npm run check
   cargo test -p gamestruments-engine
   cargo clippy -p gamestruments-engine --all-targets -- -D warnings
   ```

3. If you changed `crates/engine`, rebuild the committed WASM with
   `npm run wasm:build` and confirm `npm run wasm:verify` passes.
4. Add a line to `CHANGELOG.md` for user-visible changes.

## Ground rules

- The Rust engine (`crates/engine`) is the only generator. The Audio Lab plays
  it through WASM; do not add a second generator in TypeScript or GDScript.
- Generation must stay deterministic: same inputs and generator version, same
  score. Update test digests only for intentional musical changes and say so in
  the pull request.
- No samples or pre-rendered audio. Every voice is synthesized.
- `GamestrumentsPlayer` is the public API. If you change it, update
  `kit/docs/api.md` and the snippets in `kit/README.md` and
  `kit/docs/quickstart.md` (they are executed by the Godot smoke test).

By contributing you agree that your contributions are licensed under the MIT
License in `LICENSE.md`.
