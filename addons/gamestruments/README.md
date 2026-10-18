# Gamestruments

Gamestruments is a music addon for Godot 4. You give it a seed and a style when
a scene loads, and it builds a piece with several parts. While the game runs,
you tell it what is happening and it moves to another part at the next bar line.
It runs offline. There are no sound files to load, and there is no separate
editor. The addon makes every sound itself.

This folder is the addon. Keep the `bin/` folder inside it so Godot can load
the library for your platform.

- Hear it: <https://gamestruments.gurisitos.games>
- Source, docs and examples (free, MIT):
  <https://github.com/jfrader/gamestruments>
- Optional donation:
  <https://gurisitosgames.itch.io/gamestruments-godot>

## Install

1. Copy this `gamestruments/` folder into your project's `addons/` directory.
   Create `addons/` next to `project.godot` if it does not exist.
2. Restart Godot and wait for the import to finish. This file should exist:

```text
res://addons/gamestruments/gamestruments.gdextension
```

`GamestrumentsPlayer` then appears in the Create New Node dialog.

The addon runs on Godot 4.7.x on Linux x86_64, Windows x86_64, macOS arm64 and
x86_64, and Web. For a Web export, turn Extensions Support on and Thread Support
off. No special server headers are needed.

## Use it

Add a `GamestrumentsPlayer` child named exactly `GamestrumentsPlayer`. Attach a
script to the scene root:

```gdscript
extends Node

@onready var music: GamestrumentsPlayer = $GamestrumentsPlayer

func _ready() -> void:
    music.project_secret = "my-game"
    music.recipe = "racing"
    music.style = "neon"
    if not music.generate("level-001"):
        push_error("Gamestruments could not build the music")

func start_race() -> void:
    music.set_race_state("grid", 0.0, 0.0, false)

func update_race(intensity: float, pressure: float, final_lap: bool) -> void:
    music.set_race_state("race", intensity, pressure, final_lap)

func finish_race(won: bool) -> void:
    music.set_race_state("finish", 0.0, 0.0, false, "win" if won else "loss")
```

`project_secret` is a short name for your game. It keeps two games that use the
same seed from getting the same music, and it is not a password. `recipe` is the
kind of music: `racing`, `suspense` or `adventure`. `style` is the sound. You can
also set `energy`, `complexity`, `brightness` and `syncopation` from 0 to 1
before calling `generate`.

`generate` builds the music and returns `true` or `false`. The three functions
above are examples, so connect them to your own signals. The `set_*_state` calls
also return `true` or `false`; `false` means the music is not ready or a value is
out of range. Changes blend in at the next bar. Adventure changes enter on the
next half bar.

The player routes to a `Music` bus when the project has one and to `Master`
otherwise.

## Kinds of music

`arrangement` controls the set and order of parts. Racing uses `original`
(default) or `extended`; Suspense uses `seeded` (default) or `all-phases`.
Adventure ignores it.

- Racing: the garage, the starting grid, the race, a position fight, the final
  lap and the finish. Styles: `neon`, `funk`, `fusion`, `chip`.
- Suspense: one long piece for stealth, hacking or horror. Styles: `terminal`,
  `cipher`, `noir`, `trance`.
- Adventure: camp, exploration, town, dungeon, combat, a boss, a sanctuary and
  the final victory, plus eight variants. Styles: `folk`, `dark`, `orchestral`.

`autoplay` is for `original`/`extended` Racing and Adventure. It defaults to
`false`; set it to `true` to tour the parts automatically. Suspense and
`all-phases`/`seeded` Racing already have an order and ignore it.

## Licence

MIT, including use in closed-source commercial games. See `LICENSE.md` in this
folder. Third-party terms are in `THIRD_PARTY_NOTICES.md` in the release root.

## Support

Questions or bugs: <https://github.com/jfrader/gamestruments/issues>. Include
the addon version, Godot version, OS and the full Output text.
