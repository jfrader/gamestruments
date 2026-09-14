# Troubleshooting

## `GamestrumentsPlayer` Does Not Appear

1. Confirm you are using Godot 4.7.x on a supported desktop architecture.
2. Confirm the *entire* `addons/gamestruments/` directory was copied to matching placement (`res://addons/gamestruments/gamestruments.gdextension` next to your `project.godot`).
3. Godot loads only the platform-matching binary; the folder ships all three (linux .so, windows .dll, mac .dylib for x86_64+arm64). Restart Godot and wait for import.
4. Confirm the matching native library exists under `res://addons/gamestruments/bin/` for your OS.
5. Restart or reload the Godot project and inspect the complete GDExtension error in Output.
6. If paths are correct, close Godot, remove the project's `.godot` import cache, and reopen it.

On macOS, a downloaded archive may be quarantined. If macOS blocks a library from an archive you trust, remove quarantine from the extracted kit before copying it:

```sh
xattr -dr com.apple.quarantine /path/to/extracted/gamestruments-kit
```

## An Example Scene Reports the Addon Is Missing

The examples in `kit/examples/` create the player with
`ClassDB.instantiate("GamestrumentsPlayer")` and show an in-scene error instead
of crashing when the class is unavailable.

1. Confirm the entire `addons/gamestruments/` dir (matching placement) exists in
   the examples project.
2. A repository checkout may not include the native libraries; use the archive's
   `kit/examples/addons/gamestruments/` copy (copy whole dir, restart, wait import).
3. Godot loads the platform's binary only. If the error persists, follow
   `GamestrumentsPlayer` Does Not Appear above.

## `generate()` Returns `false`

Read the accompanying Godot error. Common causes are:

- Empty `project_secret`.
- Unsupported `style` (empty string for Racing), arrangement, or voice name.
- Non-finite trait input can fail (Racing may reject later; Suspense and Adventure fall back 0.5 for non-finite).
- An internal score invariant failed. This should not occur with an unmodified release; report the complete error and input tuple.

Do not call `set_race_state` / `set_trace_state` / `set_adventure_state` after failed generation. Pass finite 0..1 traits.

## Silence After Successful Generation

1. Confirm `Master` is audible. The player uses `Music` when that bus exists and
   otherwise falls back to `Master`.
2. If `Music` exists, confirm Master and Music are not muted and another bus is not soloed.
3. Run the scene so `_process` can feed the internal playback buffer.
4. Try the archive's self-contained `kit/examples/` project to separate installation from game-specific bus configuration.
5. Use Godot's debugger Audio view to verify the Music bus receives signal.

The shipped synth is mono at 22050 Hz by design.

## State Does Not Change

- Check the boolean result from `set_race_state` / `set_trace_state` / `set_adventure_state`. Returns `false` when no successful generate or bad args. These methods do not guard recipe — ensure you use the matching recipe.
- State changes are quantized to bar boundaries (bar-aligned blend), so listen for several seconds rather than expecting an immediate cut. (Bars can sit inside phrases.)
- Use `phase = "race"` for cruise. High intensity or pressure selects attack; `final_lap = true` selects final lap; every `phase = "finish"` request (win or loss) selects `victory`. No separate loss music is provided; both outcomes use the current victory section (see example 02 notes).
- `set_race_state` / `set_trace_state` / `set_adventure_state` bool return does not act as a recipe gate.
- Form controls (`set_form_hold` etc.) return false for Racing/Adventure unless `autoplay` was true at generate (they apply to any score that has a form).

## Different Result Than a Previous Version

Determinism requires the exact same generator version + namespace + seed + style + palette/voice overrides + traits together. Seed alone does not determine output. Updating the kit can intentionally change a score. Pin the archive and version used by a shipped game.

## Leak or Crash During Shutdown

An unmodified release is tested through repeated instantiate, generate, play, transition, free, and quit cycles. `Leaked instance`, ObjectDB leak, panic, or crash output involving `GamestrumentsPlayer`, `AudioStreamPlayer`, or `AudioStreamGeneratorPlayback` is a defect. Report the complete log and a minimal scene rather than ignoring it.

## Rebuild From Source

The archive root contains `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and both Rust crates. From that root:

```sh
cargo build -p gamestruments-godot --release --locked
```

Use the target matching your operating system and copy the resulting native library to the filename declared in `gamestruments.gdextension`. Standard support covers the provided binaries; custom targets and modified source are best-effort.

When reporting an issue through the itch.io product page's public comments, include kit version, OS and architecture, exact Godot version, reproduction steps, and complete Output text. Do not publish private project data; `project_secret` is only a namespace, so replace it consistently in a reproduction. Use itch.io's purchase-support flow for purchase-specific or private matters.
