# In-game engine

Godot games set a **project secret** and an **instrument palette** on
`GamestrumentsPlayer`, then call `generate(seed)` at level load.

Uniqueness is `secret` + `seed` + palette. Buyers of a future itch kit fill
those in the inspector so two games never share a default sound.

TypeScript Studio is not used at runtime.

```bash
cargo test -p gamestruments-engine
cargo build -p gamestruments-godot
```

`godot` crate targets gdext 0.5.5 (API 4.7) with repo-scoped `rust-toolchain.toml` pinning `channel = "1.94.0"`. Point a
`.gdextension` file at `target/debug/libgamestruments_godot.so`. Engine crate builds cleanly on the pinned toolchain (no 1.94-only features used).

## Known issues

(none; see changelog for gdext 0.5.5 upgrade)

### Toolchain

The workspace root has `rust-toolchain.toml` pinning Rust 1.94.0 (with rustfmt/clippy) so that `gamestruments-godot` (gdext 0.5.5) builds while `gamestruments-engine` and other machine projects remain on stable 1.93. `cargo` commands inside the worktree auto-select the pinned toolchain via rustup. Engine tests/clippy remain green.
