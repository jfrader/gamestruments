# 03 — Song form

Drives the Suspense recipe: trace-state requests, section cues, and hold/advance
form controls.

## What it teaches

- Configure `recipe = "suspense"`, a suspense style, and an arrangement before
  `generate()`.
- Send `set_trace_state(phase, heat, focus, progress)` from gameplay, checking
  its result and keeping numeric inputs finite and in `0..1`.
- Cue a section with `cue_section`, freeze or resume the form with
  `set_form_hold`, step it with `advance_form`, and read state with
  `is_form_held` and `get_current_section`.

## Run

Open `kit/examples/project.godot`, then `03-song-form/song_form.tscn`, and press
F6. The downloaded project includes its addon; see [setup](../README.md) if
running from source.

## Code to copy

Read [`song_form.gd`](song_form.gd) to see each control call the public API.
For your own scene, copy the complete [Suspense quickstart script](../../docs/quickstart.md#suspense-recipe-complete-script)
instead of the example's UI bindings.

## Where your game callbacks plug in

Connect your game events to the matching Suspense state or form-control call.
Use hold, advance, and cue for scripted story moments. Issue one operation per
event rather than calling all three together. The sample callback names do not
wire anything automatically.

## Expected result

Sections blend on bar boundaries. Holding the form freezes automatic progression while the music keeps
playing the held section; resuming continues from there. `get_current_section()`
is coarse section reporting during a crossfade, not a precise bar or mix
readout. Before connecting completion values, check the [native progress rule](../../docs/api.md#suspense--set_trace_state)
and its browser/native difference.
