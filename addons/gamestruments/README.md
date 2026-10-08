# Gamestruments — Adaptive Music for Godot 4

Procedural music that adapts to your game. Gamestruments generates
deterministic, sample-free music inside Godot — no audio files, no authoring
tool, no external services. Add one node, call `generate()` when a scene loads,
then tell it what's happening; it blends between sections on bar boundaries.

This folder is the addon. Keep it intact so Godot can load the platform-matching
binary from `bin/`.

**Hear it in the browser:** <https://gamestruments.gurisitos.games>
(Godot and HTML5, one engine.)

**Source, docs and native examples** (free, MIT):
<https://github.com/jfrader/gamestruments>. Optional donations:
<https://gurisitosgames.itch.io/gamestruments-godot>

## Install

1. Copy this `gamestruments/` folder into your project's `addons/` directory
   (create `addons/` next to `project.godot` if needed).
2. Restart Godot and wait for import to finish. This file should exist:

```text
res://addons/gamestruments/gamestruments.gdextension
```

`GamestrumentsPlayer` then appears in the Create New Node dialog. Requires
Godot 4.7.x on Linux x86_64, Windows x86_64, macOS arm64/x86_64, or Web.

For Web, set **Extensions Support** on and **Thread Support** off in the export
preset. No special server headers are needed.

## Use it

Add a `GamestrumentsPlayer` child named exactly `GamestrumentsPlayer`. Attach a
script to the scene root:

```gdscript
extends Node

@onready var music: GamestrumentsPlayer = $GamestrumentsPlayer
var music_ready := false

func _ready() -> void:
    music.project_secret = "my-game"  # non-empty stable per-title namespace
    music.recipe = "racing"
    music.style = "neon"              # neon, funk, fusion, or chip
    music.arrangement = "original"    # native default; "extended" adds sections
    music.autoplay = false            # native default; true tours the sections
    music_ready = music.generate("level-001")
    if not music_ready:
        push_error("Gamestruments generation failed")
```

Drive it from your own game events and check each bool return:

- Racing: `set_race_state(...)`
- Suspense: `set_trace_state(...)` plus form controls
- Adventure: `set_adventure_state(...)`

Changes commit on the next bar boundary. The player routes to a `Music` bus
when you have one, and falls back to `Master` when you don't.

## Recipes

One player, three recipes. Set `recipe` before `generate`.

| Recipe | Id | Styles | Drive with |
|---|---|---|---|
| Racing | `racing` | neon, funk, fusion, chip | `set_race_state` |
| Suspense | `suspense` | terminal, cipher, noir | `set_trace_state` |
| Adventure | `adventure` | folk, dark, orchestral | `set_adventure_state` |

`arrangement` is `original` (default) or `extended` for Racing, and `original`,
`extended`, or `theme` for Suspense. Adventure ignores it. `autoplay` is
Racing/Adventure only and defaults to `false`.

## Requirements

- Godot 4.7.x on Linux x86_64, Windows x86_64, macOS arm64/x86_64, or Web.

For Web, set **Extensions Support** on and **Thread Support** off in the export
preset. No special server headers are needed.
- Offline runtime: no samples, network, middleware, or telemetry.

## License

MIT — use in closed-source commercial games. See `LICENSE.md` in this folder.

## Support

Questions or bugs: <https://github.com/jfrader/gamestruments/issues>
with your version, Godot version, OS, and full Output text.
