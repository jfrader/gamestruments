# Gamestruments Kit Demo

This self-contained Godot 4.7 project exercises the exact `GamestrumentsPlayer` API and native synth shipped to buyers.

## Run the Packaged Demo

1. Extract the buyer archive.
2. Open `kit/demo/` as a Godot project.
3. Press Play.

The demo addon already contains Linux, Windows, and universal macOS libraries at `res://addons/gamestruments/`. It generates a score on startup and provides controls for style, seed, built-in voices, traits, and all six racing sections.

Use the root `addons/gamestruments/` copy for your own project. Do not copy both locations into one project.

## Demonstrated API

- `project_secret`, `style`, voice overrides, and generation traits.
- `generate(seed) -> bool`.
- `set_race_state(phase, intensity, pressure, final_lap, finish_result) -> bool`.
- Garage, grid, cruise, attack, final-lap, and victory selection.
- Bar-quantized crossover and in-process sample-free synthesis.

The demo checks generation failure before continuing. Its victory control uses `phase = "finish"` and `finish_result = "win"`.

## Repository Smoke Test

After building the host library, run the isolated package smoke harness:

```sh
cargo build -p gamestruments-godot --release
node tests/godot-package-smoke.mjs \
  --godot "$(command -v godot)" \
  --library target/release/libgamestruments_godot.so
```

The harness stages a fresh project, loads the extension, performs two generate/play/transition/free cycles, rejects runtime errors and leaked-object messages, and requires an explicit pass marker.

See `../docs/quickstart.md` for buyer integration and `../docs/api.md` for the supported contract.
