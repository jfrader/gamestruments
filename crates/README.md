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

`godot` crate currently targets gdext 0.4.5 (rustc 1.93). Point a
`.gdextension` file at `target/debug/libgamestruments_godot.so`.
