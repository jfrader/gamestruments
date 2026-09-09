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
- TypeScript Studio (`packages/studio`, AGPL) is the Audio Lab only. Do not
  ship Strudel into games.
- In-game engine is Rust: `crates/engine` (MIT generator + transport + synth)
  and `crates/godot` (GDExtension). Games generate at level load from a
  project secret, instrument palette, and seed. Do not pre-bake every
  procedural race to WAV. Recipes: Pocket Circuit (racing loops) and
  Suspense (song-form; Arkhos).
- `@gamestruments/runtime` is the TypeScript MIT transport used by the lab.
- The buyer demo is a playable game integration: actual racing drives music.
  Neither a generator parameter panel nor a copy of the Audio Lab UI satisfies
  demo acceptance. Keep music integration separate from game presentation.

## Verify

```bash
npm run check
cargo test -p gamestruments-engine
cargo clippy -p gamestruments-engine --all-targets -- -D warnings
```

Requires Node.js 24. Final checks run in CI after push.

Note: this workspace pins Rust 1.94 via root `rust-toolchain.toml` (for gdext 0.5.5 in the godot crate only; engine crate introduces no 1.94-only features and continues to build cleanly).

## Graphify

- Query `graphify-out/graph.json` before broad source searches.
- `graphify-out/` is local generated state, ignored per repo: dirty files there
  are expected and never committed. Never delete files inside it.

## Changelog

- User-visible changes go in root `CHANGELOG.md`.
- Read the `changelog` skill for what qualifies.
