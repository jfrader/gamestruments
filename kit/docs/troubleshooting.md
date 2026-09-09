# Troubleshooting

## `GamestrumentsPlayer` Does Not Appear

1. Confirm you are using Godot 4.7.x on a supported desktop architecture.
2. Confirm `gamestruments.gdextension` is at `res://addons/gamestruments/gamestruments.gdextension`.
3. Confirm the matching native library exists under `res://addons/gamestruments/bin/`:
   - Linux: `libgamestruments_godot.so`
   - Windows: `gamestruments_godot.dll`
   - macOS: `libgamestruments_godot.dylib`
4. Restart or reload the Godot project and inspect the complete GDExtension error in Output.
5. If paths are correct, close Godot, remove the project's `.godot` import cache, and reopen it.

On macOS, a downloaded archive may be quarantined. If macOS blocks a library from an archive you trust, remove quarantine from the extracted kit before copying it:

```sh
xattr -dr com.apple.quarantine /path/to/extracted/gamestruments-kit
```

## `generate()` Returns `false`

Read the accompanying Godot error. Common causes are:

- Empty `project_secret`.
- Unsupported `style` or voice name.
- Non-finite trait input.
- An internal score invariant failed. This should not occur with an unmodified release; report the complete error and input tuple.

Do not call `set_race_state` after failed generation.

## Silence After Successful Generation

1. Confirm `Master` is audible. The player uses `Music` when that bus exists and
   otherwise falls back to `Master`.
2. If `Music` exists, confirm Master and Music are not muted and another bus is not soloed.
3. Run the scene so `_process` can feed the internal playback buffer.
4. Try the archive's self-contained `kit/demo/` project to separate installation from game-specific bus configuration.
5. Use Godot's debugger Audio view to verify the Music bus receives signal.

The shipped synth is mono at 22050 Hz and intentionally leaner than the browser Audio Lab.

## State Does Not Change

- Check the boolean result from `set_race_state`. It returns `false` when generation has not succeeded.
- State changes are quantized to bar boundaries, so listen for several seconds rather than expecting an immediate cut.
- Use `phase = "race"` for cruise. High intensity or pressure selects attack; `final_lap = true` selects final lap; every `phase = "finish"` request selects victory, with `finish_result = "win"` representing the intended win case.

## Different Result Than a Previous Version

Determinism requires the exact same generator version, namespace, seed, style, voice overrides, and traits. Updating the kit can intentionally change a score. Pin the archive and version used by a shipped game.

## Leak or Crash During Shutdown

An unmodified release is tested through repeated instantiate, generate, play, transition, free, and quit cycles. `Leaked instance`, ObjectDB leak, panic, or crash output involving `GamestrumentsPlayer`, `AudioStreamPlayer`, or `AudioStreamGeneratorPlayback` is a defect. Report the complete log and a minimal scene rather than ignoring it.

## Rebuild From Source

The archive root contains `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and both Rust crates. From that root:

```sh
cargo build -p gamestruments-godot --release --locked
```

Use the target matching your operating system and copy the resulting native library to the filename declared in `gamestruments.gdextension`. Standard support covers the provided binaries; custom targets and modified source are best-effort.

When reporting an issue through the itch.io product page's public comments, include kit version, OS and architecture, exact Godot version, reproduction steps, and complete Output text. Do not publish private project data; `project_secret` is only a namespace, so replace it consistently in a reproduction. Use itch.io's purchase-support flow for purchase-specific or private matters.
