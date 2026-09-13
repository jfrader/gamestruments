# 02 — Game signals

Maps simulated race events to `set_race_state` requests on the Racing recipe (original arrangement, state-driven).
The controls stand in for game events; there is no driving physics here.

## What it teaches

- Configure `recipe = "racing"` and a racing style before `generate()`.
- Translate phase, intensity, pressure, and final-lap state into
  `set_race_state(...)`, checking the result and keeping numeric inputs in `0..1`.
- Requested and currently playing sections are different: changes commit on bar
  boundaries (bar-aligned), so the readout can show a new request before you hear it.
- Clear race pressure and final-lap state when the race finishes.

## Run

Open `kit/examples/project.godot`, then `02-game-signals/game_signals.tscn`, and
press F6. The downloaded project includes its addon; see [setup](../README.md)
if running from source.

## Code to copy

Read [`game_signals.gd`](game_signals.gd), especially `_request_state` and
`_finish`. For your own scene, copy the complete [Racing quickstart script](../../docs/quickstart.md#racing-recipe-complete-script)
rather than the example's UI bindings.

## Where your game callbacks plug in

Call `set_race_state` from your own countdown, telemetry tick, and finish
handlers. Connect or call those handlers from your game; their names do not
wire them automatically. Use `set_race_state` with Racing, and `set_trace_state`
with Suspense. Both require successful generation first.

## Expected result

Racing music plays; the controls request grid, cruise, attack, final lap, and
victory. Not every request is audible immediately — wait for the next bar. The
example reports the requested section separately from the currently playing
section.

**Finish behavior:** this example uses the same `victory` section for both win
and loss. Its finish handlers clear intensity, pressure, and final-lap inputs,
which would otherwise take precedence. Pressing Apply afterward keeps the
finish state. There is no separate losing theme.
