# Night Circuit — Gamestruments Gameplay Demo

A small playable three-lap race against one rival, using the exact
`GamestrumentsPlayer` API and native synth shipped with the kit. Music responds
to the race; there are no manual music-section buttons or generator panels.

## Play

1. Extract the buyer archive and open `kit/demo/project.godot` in Godot 4.7.x.
2. Let the initial extension scan finish, then press Play (F6 runs the current
   scene; F5 runs this project's main scene).
3. Press Enter or click Race. Hold W through the countdown to accelerate.

| Control | Action |
|---|---|
| W / Up | Throttle |
| S / Down | Brake (overrides throttle) |
| A, D / Left, Right | Steer left / right (grip-assisted) |
| Space | Boost while accelerating; release to recharge/rearm |
| Escape / Pause button | Pause or resume race and music |
| R | Restart the race |
| Enter / Race Again | Race from the title or finish screen |

The car steers with grip assist: hold A/D to turn, release to straighten out.
You control speed, racing line, and boost timing. Pass the orange rival without
contact; leaving the road, hitting the barrier, or touching the rival slows you
down. Finish three laps first to win. Focus loss pauses the race. Keyboard and
mouse are supported; no gamepad or touch controls are claimed.
The 960×620 layout scales and letterboxes when the window is resized.

## How the Game Uses the Library

- `race_model.gd`: race rules, grip-assisted steering, rival motion, boost,
  contact, lap counting, and gameplay telemetry. It has no dependency on the
  music library or UI.
- `race_music.gd`: the small integration adapter to reuse as a reference. It
  creates one player, configures the title namespace/style, and checks
  `generate(level_seed)` on level load. `sync_race` checks `set_race_state`.
- `kit_demo.gd`: keyboard input, track/car drawing, HUD, and pause/restart. It
  passes telemetry to the adapter five times a second while the race runs.

| Gameplay | Music request |
|---|---|
| Title screen | Garage |
| Three-second countdown | Grid |
| Ordinary racing away from rival | Cruise |
| Nearby rival or boosting/high speed | Attack |
| Third lap | Final lap (takes priority over pressure) |
| Finish, with the real win/loss result | Victory/outro |

The readout shows the **requested** section; audible changes wait for musical
bar boundaries. The library currently uses the same victory section for both
finish outcomes. Restart resets the race, requests grid, and reuses the loaded
score. To try a different score, change `level_seed` or `music_style` in
`race_music.gd` and reload the scene. Voice/trait customization is described in
`../docs/api.md`, not exposed as gameplay controls.

The packaged addon is already installed at `res://addons/gamestruments/`.
For your own game, copy the archive's root `addons/gamestruments/` instead.
Do not copy both locations into one project. A missing extension produces a
visible warning; the race still runs silently, but that is not a passing music test.

## Repository Verification

After building the host native library:

```sh
cargo build -p gamestruments-godot --release
node tests/godot-package-smoke.mjs \
  --godot "$(command -v godot)" \
  --library target/release/libgamestruments_godot.so
```

The harness stages a fresh copy, tests native lifecycle and the complete race
through keyboard-driven throttle/boost, verifies gameplay music requests,
pause/restart, win/loss, contact, off-road penalties, and charge exhaustion.
Both runtime and gameplay pass markers are required; engine errors and leaks
fail the check. Add `--screenshots <directory>` on a machine with a display to
capture title, countdown, pause, race, final-lap, and finish states. Automated
simulation does not replace real-time human play and listening acceptance.
