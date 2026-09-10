# Quickstart — Fresh Godot Project

## Prerequisites

- Godot 4.7.x.
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
5. Optionally add an audible bus named `Music` in Godot's Audio panel for
   separate music mixing. A fresh project without that bus falls back to
   `Master`.

## Drive It

Attach this script to the scene root and call the three race callbacks from
your game's countdown, telemetry updates, and finish event:

```gdscript
extends Node

@onready var player: GamestrumentsPlayer = $GamestrumentsPlayer
var music_ready := false

func _ready() -> void:
    music_ready = player.generate("level-001")
    if not music_ready:
        push_error("Gamestruments score generation failed")

func countdown_started() -> void:
    if music_ready:
        player.set_race_state("grid", 0.0, 0.0, false)

func race_updated(intensity: float, pressure: float, lap: int, total_laps: int) -> void:
    if music_ready:
        player.set_race_state("race", clampf(intensity, 0.0, 1.0),
            clampf(pressure, 0.0, 1.0), lap == total_laps)

func race_finished(won: bool) -> void:
    if music_ready:
        player.set_race_state("finish", 0.0, 0.0, false, "win" if won else "loss")
```

Run the scene. The initial `garage` score starts immediately; accepted state requests commit on upcoming bar boundaries rather than cutting instantly.

For a complete playable integration, open `kit/demo/` as a Godot project and
race using the controls in `kit/demo/README.md`. Its addon is already installed.
The demo ships four circuits, one per engine style (neon, pocket funk, fusion,
micro motor); switch circuits in the garage to hear each regenerate. Read
`race_model.gd` for deriving intensity/pressure from gameplay and
`race_music.gd` for generation and checked state requests. No timers or manual
section buttons simulate the race. The music readout reports a request, not the
current audible bar.

See `api.md`, `limitations.md`, and `troubleshooting.md` before shipping an integration.
