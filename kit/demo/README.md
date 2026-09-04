# Gamestruments Kit Demo

Minimal Godot 4 scene + script that exercises the public `GamestrumentsPlayer` API:

- `project_secret`, `style`, `*_voice`, trait sliders
- `generate(seed)`
- `set_race_state(phase, intensity, pressure, final_lap)`

It proves load-time deterministic generation and the bar-quantized adaptive arc (garage → grid → race cruise/attack/final-lap → finish).

## How to use in any Godot 4 project

1. Build (or obtain) the GDExtension:
   - From repo root: `cargo build -p gamestruments-godot` (release for shipping: `--release`)
   - Result: `target/debug/libgamestruments_godot.so` (or release)

2. Add the addon to your project (standard GDExtension layout):
   ```
   addons/
     gamestruments/
       gamestruments.gdextension
       bin/
         libgamestruments_godot.so
   ```
   - Copy `crates/godot/gamestruments.gdextension` → `addons/gamestruments/gamestruments.gdextension`
   - Copy the `.so` → `addons/gamestruments/bin/libgamestruments_godot.so`
   - The `.gdextension` file already references `res://addons/gamestruments/bin/...` — do not edit the library copy.

3. Add the demo (or your own scene):
   - Copy the `kit/demo/` folder into your project (it becomes `res://kit/demo/`).
   - Open `res://kit/demo/kit_demo.tscn` (or instance the scene / attach `kit_demo.gd`).
   - The scene root is a `Control` that builds its entire UI in `_ready()`.

4. Run. The demo:
   - Auto-generates with a demo secret + seed on ready (so you hear music immediately).
   - Lets you change style / seed / voices / traits then hit **Generate**.
   - Six buttons drive `set_race_state` with the exact signature from the binding:
     - Garage / Grid
     - Cruise (uses current Energy slider value as intensity)
     - Attack (race + pressure 0.8)
     - Final Lap (race + final_lap=true)
     - Victory (phase="finish")

If the class does not appear, restart the editor after placing the native library.

## Repo-local headless smoke test (this checkout)

The kit itself is a library, not a standalone game, so there is no root `project.godot`. For quick verification:

```sh
# 1. Build the native lib
cargo build -p gamestruments-godot

# 2. Prepare a throwaway project that includes the demo subtree + extension
mkdir -p /tmp/kit-smoke
cp -a kit/demo /tmp/kit-smoke/kit
cp crates/godot/gamestruments.gdextension /tmp/kit-smoke/gamestruments.gdextension
mkdir -p /tmp/kit-smoke/bin
cp target/debug/libgamestruments_godot.so /tmp/kit-smoke/bin/

# 3. Fix the .gdextension paths for this flat layout (do this only in the temp copy)
sed -i 's|res://addons/gamestruments/bin/|res://bin/|g' /tmp/kit-smoke/gamestruments.gdextension

# 4. Create a project.godot at the temp root (or use the one from kit/demo and adjust)
cat > /tmp/kit-smoke/project.godot << 'EOF'
config_version=5

[application]
config/name="Gamestruments Kit Demo (smoke)"
run/main_scene="res://kit/demo/kit_demo.tscn"
config/features=PackedStringArray("4.7")
EOF

# 5. Boot / smoke (no editor UI)
godot --path /tmp/kit-smoke --headless --quit
# or to run a specific generator test:
# godot --path /tmp/kit-smoke --headless --script res://kit/demo/tools/generate_demo_scene.gd
```

You should see no `ERROR` or `SCRIPT ERROR`, and "GamestrumentsPlayer" class must be loadable (the demo auto-generates and starts the garage section).

## Notes on paths

- The committed `gamestruments.gdextension` (in `crates/godot/`) always uses the canonical `res://addons/gamestruments/bin/...` layout. Never change it.
- Only the temporary smoke copies get path adjustments.
- When a buyer drops `kit/demo/` into a real project that also has `addons/gamestruments/`, the demo's `res://kit/demo/...` paths are correct and the extension registers under the standard addon path.

## API gap observed while implementing

`set_race_state` signature is exactly `(phase: String, intensity: float, pressure: float, final_lap: bool)`.
`finish_result` is not exposed on the GDScript side (the binding always passes "none" internally). For "victory" we use `phase = "finish"` (the transport still performs the musical release). This matches the public contract in `docs/kit-contract.md`.

## Verification

See the PR body for the exact smoke run that was performed on this machine.
