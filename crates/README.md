# In-game engine

Godot games call `GamestrumentsPlayer.generate(game_id, seed, style)` at
level load. The same user seed is a different piece per `gameId`.

TypeScript Studio is not used at runtime.

```bash
cargo test -p gamestruments-engine
cargo build -p gamestruments-godot
```

`godot` crate currently targets gdext 0.4.5 (rustc 1.93). Point a
`.gdextension` file at `target/debug/libgamestruments_godot.so`.
