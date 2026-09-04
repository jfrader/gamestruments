# Quickstart — Minimal Integration in a Fresh Godot 4 Project

This produces a working scene using only the shipped kit archive and these docs.

## Prerequisites

- Godot 4.7+ (or any 4.x that matches the `.gdextension` minimum).
- The `gamestruments-<version>-godot4.zip` archive (or equivalent layout).

## Steps

1. Create a new empty Godot 4 project (or open an existing one).

2. Extract the archive. Locate the addon contents (the tree that contains `gamestruments.gdextension` and a `bin/` directory with the `.so`).

3. Copy the `addons/gamestruments/` directory (the folder containing `gamestruments.gdextension`) into your project's `res://addons/` so the final path is:
   ```
   res://addons/gamestruments/gamestruments.gdextension
   res://addons/gamestruments/bin/libgamestruments_godot.so
   ```
   (Create the `addons/` and `bin/` folders if they do not exist.)

4. In the Godot editor, open the FileSystem dock and confirm Godot imported the `.gdextension` without errors (no red errors in Output).

5. Create a new scene (`main.tscn`). Add a root `Node` (or `Node2D`).

6. Add a child `GamestrumentsPlayer` node (search for it in the "Create New Node" dialog; it appears once the extension is loaded).

7. Select the `GamestrumentsPlayer` and set in the Inspector:
   - **Project Secret**: `my-title-secret-42` (any non-empty string; treat as per-title key).
   - **Style**: `funk` (or `fusion`, `neon`, `chip`).
   - Leave voice fields empty (style defaults will be used).
   - Leave trait sliders at their defaults or set e.g. `energy = 0.7`.

8. Attach a new GDScript to the root node (or to the player). Replace its contents with:

   ```gdscript
   extends Node

   @onready var player: GamestrumentsPlayer = $GamestrumentsPlayer

   func _ready() -> void:
       # Generate the deterministic score for this level seed.
       player.generate("level-001")

       # Example: after a short delay or on input, drive state changes.
       # These commit on the next bar boundary.
       await get_tree().create_timer(2.0).timeout
       player.set_race_state("grid", 0.65, 0.3, false)

       await get_tree().create_timer(4.0).timeout
       player.set_race_state("race", 0.85, 0.4, false)

       await get_tree().create_timer(6.0).timeout
       player.set_race_state("attack", 0.95, 0.8, false)

       await get_tree().create_timer(4.0).timeout
       player.set_race_state("final-lap", 1.0, 0.9, true)
   ```

9. Make sure your project has a bus named "Music" (Project Settings → Audio → Buses). The player creates an internal `AudioStreamPlayer` that routes to it. If you want to hear it, either leave the bus at 0 dB or route the Master bus appropriately.

10. Run the scene (`F5` or the play button).

You should hear synthesized music start. Changing the seed, style, or traits in the inspector and re-running (or calling generate again) produces a different but deterministic result for the same inputs.

## Next

- See `api.md` for the exact exported properties, ranges, defaults, and method signatures.
- See `limitations.md` for what the engine does **not** do.
- See `troubleshooting.md` for common first-run issues ("no sound", extension not loaded, etc.).
- The demo scene shipped in the archive exercises the same calls via buttons and sliders.

All behavior above is implemented in the shipped GDExtension and Rust crates. No additional assets or plugins are required.
