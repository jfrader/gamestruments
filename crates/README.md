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

## Known issues

### Shutdown leak of AudioStreamGeneratorPlayback (1 extra ObjectDB instance)

Running headless `--script` tests that instantiate `GamestrumentsPlayer` (e.g.
`gamestruments_player_test.gd`) reports "N ObjectDB instances were leaked at
exit", where N is 1 higher than a minimal noop `SceneTree` script baseline in
the same project (baseline typically 1; gamestruments test shows 2).

- Leaked objects are always `AudioStreamGeneratorPlayback` (refcount 1 at exit),
  never the `AudioStreamGenerator` itself.
- Code uses only transient `Gd<>` lookups (no stored `Gd<AudioStreamPlayer>` or
  `Gd<AudioStreamGeneratorPlayback>` fields), `stop()`, `set_stream(null)`,
  `remove_child` + `free()` of the "LiveStream" child inside `exit_tree`, plus
  a no-op `Drop`. Both `exit_tree` and `Drop` paths were added.
- The leak occurs per `GamestrumentsPlayer` instance (autoloader creates one;
  test script creates a second).
- Root cause: engine-side, due to Godot version mismatch. gdext 0.4.5 is built
  against Godot 4.5 API (see init banner: "API v4.5.stable.official, runtime
  v4.7.2"); refcounting / lifetime of internal `AudioStreamGeneratorPlayback`
  created by `AudioStreamPlayer.play()` on a generator stream is not fully
  released on player free / stream clear / node exit in 4.7.2.
- No downgrade of Godot or gdext performed (gdext 0.4.5+ requires rustc 1.94;
  machine has 1.93.1).
- The transient lookup + explicit child free in exit_tree was the final
  minimization attempt; count did not drop from 2→1.

This is documented rather than hacked around further. Real-game use (with
full scene tree lifetime) may not exhibit user-visible effects.
