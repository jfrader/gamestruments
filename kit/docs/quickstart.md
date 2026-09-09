# Quickstart — Fresh Godot Project

## Prerequisites

- Godot 4.7 or newer.
- Linux x86_64, Windows x86_64, or macOS arm64/x86_64.
- An extracted `gamestruments-<version>-godot4.zip`.

## Install

Copy the archive's root `addons/gamestruments/` folder into your project so these paths exist:

```text
res://addons/gamestruments/gamestruments.gdextension
res://addons/gamestruments/bin/libgamestruments_godot.so
res://addons/gamestruments/bin/gamestruments_godot.dll
res://addons/gamestruments/bin/libgamestruments_godot.dylib
```

Restart Godot. `GamestrumentsPlayer` should appear in the Create New Node dialog without GDExtension errors in the Output panel.

## Configure

1. Add a `GamestrumentsPlayer` child to a scene.
2. Set `project_secret` to a stable, non-empty namespace for your title. It is not a security credential.
3. Set `style` to `fusion`, `neon`, `funk`, or `chip`.
4. Leave voice overrides empty for style defaults or choose a supported voice from `api.md`.
5. Ensure the project has an audible bus named `Music`.

## Drive It

Attach this script to the scene root:

```gdscript
extends Node

@onready var player: GamestrumentsPlayer = $GamestrumentsPlayer

func _ready() -> void:
    if not player.generate("level-001"):
        push_error("Gamestruments score generation failed")
        return

    await get_tree().create_timer(2.0).timeout
    player.set_race_state("grid", 0.5, 0.2, false)

    await get_tree().create_timer(4.0).timeout
    player.set_race_state("race", 0.6, 0.3, false)

    await get_tree().create_timer(4.0).timeout
    player.set_race_state("race", 0.9, 0.8, false)

    await get_tree().create_timer(4.0).timeout
    player.set_race_state("race", 1.0, 0.9, true)

    await get_tree().create_timer(4.0).timeout
    player.set_race_state("finish", 0.3, 0.0, false, "win")
```

Run the scene. The initial `garage` score starts immediately; accepted state requests commit on upcoming bar boundaries rather than cutting instantly.

For the shortest evaluation path, open the archive's `kit/demo/` folder directly as a Godot project. Its addon is already installed and its controls exercise the same public methods.

See `api.md`, `limitations.md`, and `troubleshooting.md` before shipping an integration.
