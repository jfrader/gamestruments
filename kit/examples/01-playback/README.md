# 01 — Playback

Generates a Suspense **seeded** pool score for title music and plays it through
the native addon.

## What it teaches

- Set `project_secret` to a non-empty, stable namespace.
- Set `recipe` and `style` **before** calling `generate()`.
- Check the boolean result of `generate()` and stop if it fails.
- Restart playback and read the current section with `get_current_section()`.

## Run

Open `kit/examples/project.godot` and press F5. Alternatively, open
`01-playback/playback.tscn` and press F6. The downloaded project includes its
addon; source-checkout setup is covered in [the examples README](../README.md).

## Code to copy

Read [`playback.gd`](playback.gd) to see configuration, checked generation, and
restart. For your own scene, use the complete [Suspense quickstart script](../../docs/quickstart.md#suspense-recipe-complete-script)
and set `arrangement = "seeded"` (the default). You do not need the example's UI bindings.

## Where your game callbacks plug in

Nowhere in this example: it is standalone playback. For gameplay-driven music
see `02-game-signals/` and `03-song-form/`.

## Expected result

Title music starts after a successful `generate()`. Restart replays from the
first section (`intro`). The section readout is coarse: during a
crossfade it may report the entering section rather than a precise bar or mix
position. The small controls demonstrate playback; the separate browser Audio
Lab is the full interactive showcase.
