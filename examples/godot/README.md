# Godot portable-score example

This Godot 4.7 example consumes a Gamestruments portable score as plain JSON. It does not import JavaScript, Strudel, or the AGPL authoring package at runtime.

## Run

Open this directory as a Godot project and run `main.tscn`.

Use the phase buttons, intensity and pressure sliders, and final-lap toggle. The GDScript evaluates the score's exported adaptive rules, schedules the selected section at the next bar, and displays the equal-power gains across the configured two-bar crossfade.

## Regenerate the score

From the repository root:

```sh
npm run build:studio
node packages/studio/dist/cli.js \
  --seed godot-demo \
  --style fusion \
  --energy 0.7 \
  --complexity 0.62 \
  --brightness 0.55 \
  --syncopation 0.68 \
  --output examples/godot/data/pocket-circuit.score.json \
  --manifest examples/godot/data/pocket-circuit.manifest.json \
  --pretty
```

The manifest records the generator version, normalized inputs, named domain seeds, and SHA-256 checksum for the compact score JSON.

## Files

- `data/pocket-circuit.score.json`: engine-agnostic score, events, sections, and adaptive rules.
- `data/pocket-circuit.manifest.json`: deterministic generation manifest.
- `portable_score_demo.gd`: JSON loading, rule evaluation, bar quantization, transition queueing, and mix visualization.
- `main.tscn`: minimal scene whose interface is built by the script.
