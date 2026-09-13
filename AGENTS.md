# Repository agent notes

## Linear workflow

- Track project work in Linear, project **Gamestruments**:
  https://linear.app/gurisitosgames/project/gamestruments-049e669351cd
- New ideas are Linear issues in Backlog. Pick up issues, move `In Progress`,
  comment progress, move `Done` with a closing comment once merged into `main`.
- Feature-branch work on a tracked issue uses a worktree `gamestruments-GURI-N`
  beside this checkout. Do not switch this clone's branch for issue work.
- Read the `linear-workflow` skill before creating or updating any issue.

## Product

- This repo is a **game music library**, not a Pocket Circuit checkout.
- One generator authority: the Rust engine (`crates/engine`). The Audio Lab
  (`apps/demo`, MIT) drives it through the committed WASM build; there is no
  TypeScript generator or CLI pipeline.
- In-game engine is Rust: `crates/engine` (MIT generator + transport + synth)
  and `crates/godot` (GDExtension). Games generate at level load from a
  project secret, instrument palette, and seed. Do not pre-bake every
  procedural race to WAV. Recipes: Racing (racing loops), Suspense (song-form;
  Arkhos), and Adventure (an eight-section fantasy quest arc).
- `@gamestruments/runtime` is the TypeScript MIT transport used by the lab.
- The existing `apps/demo` browser Audio Lab is the single musical showcase. Do
  not duplicate it in Godot or embed a browser in the addon, and do not build an
  independent showcase UI, renamed substitute phases/scenario layer, or code/docs
  panels. Use the lab's actual generated phases, names, and controls.
  Its itch.io HTML preview must be packaged from the same approved lab source,
  not recreated in the kit; code examples and integration docs belong in the
  buyer kit, not in web-lab code panels or documentation tabs.
  If the latest lab source on another unmerged branch includes unapproved release
  content, keep the release blocked and reconcile the source explicitly — never
  recreate a hybrid preview.
  The buyer project in `kit/examples/` contains
  three independently runnable scenes: playback, game signals, and song form.
  Keep their public API calls visible and usable without the lab or a game.
  Generate scene files with `kit/examples/tools/generate_example_scenes.gd`;
  verify clean imports, all three examples, and missing-addon error handling with
  `node tests/godot-package-smoke.mjs --godot <binary> --library <extension>`.
  That smoke also runs the README and quickstart GDScript examples in a separate
  fresh project. Keep those snippets complete and update their checks with them.

## Verify

```bash
npm run check
cargo test -p gamestruments-engine
cargo clippy -p gamestruments-engine --all-targets -- -D warnings
```

Requires Node.js 24. Final checks run in CI after push.

After editing the Rust engine, rebuild the committed WASM (`npm run wasm:build`)
and verify its digest (`npm run wasm:verify`; the digest covers inline tests too).

Note: this workspace pins Rust 1.94 via root `rust-toolchain.toml` (for gdext 0.5.5 in the godot crate only; engine crate introduces no 1.94-only features and continues to build cleanly).

## Graphify

- Query `graphify-out/graph.json` before broad source searches.
- `graphify-out/` is local generated state, ignored per repo: dirty files there
  are expected and never committed. Never delete files inside it.

## Collaboration

- Prefer Grok/DeepSeek for substantial implementation/review to conserve the GPT
  budget; the primary agent owns coordination, integration, and verification.

## Changelog

- User-visible changes go in root `CHANGELOG.md`.
- Read the `changelog` skill for what qualifies.
