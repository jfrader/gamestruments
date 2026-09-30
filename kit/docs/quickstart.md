# Quickstart — Fresh Godot Project

Get a generated score playing in a new Godot 4.7 project, then drive it from
gameplay. The two scripts below are complete, standalone, and copy-pastable.

Install the addon first, then choose either the Racing or Suspense script.
Each player generates one recipe at a time.

Callbacks shown are samples. They are not auto-wired by name. In your game,
connect your existing events (or call the functions) to the appropriate
`set_race_state` / `set_trace_state` / form calls. Check every `bool` return.

## Prerequisites

- Godot 4.7.x.
- Linux x86_64, Windows x86_64, or macOS arm64/x86_64.
- An extracted `gamestruments-<version>-godot4.zip`.

## Install

Place the archive's `gamestruments/` addon inside your project's `addons/`
directory. Create `addons/` next to `project.godot` if needed; preserve any other
addons already there. Restart Godot and wait for import to finish. The resulting
paths are:

```text
res://addons/gamestruments/gamestruments.gdextension
res://addons/gamestruments/bin/libgamestruments_godot.so
res://addons/gamestruments/bin/gamestruments_godot.dll
res://addons/gamestruments/bin/libgamestruments_godot.dylib
```

Godot loads only the binary for your current platform from that folder; the
folder contains the three libraries for the packaged targets.

`GamestrumentsPlayer` should appear in the Create New Node dialog without
GDExtension errors. If not, see `troubleshooting.md`.

## Racing recipe (complete script)

1. In a new Godot 4.7 project create a scene with a root Node.
2. Add a child `GamestrumentsPlayer` (exact name).
3. Attach this complete script to the *root* node.
4. Run with F6.

```gdscript
extends Node

@onready var music: GamestrumentsPlayer = $GamestrumentsPlayer
var music_ready := false

func _ready() -> void:
    # Required: non-empty, stable namespace for this title.
    music.project_secret = "my-game"
    # Set recipe and supported style BEFORE generate().
    music.recipe = "racing"
    music.style = "neon"  # neon, funk, fusion, or chip
    music.arrangement = "original"
    music.autoplay = false

    music_ready = music.generate("level-001")
    if not music_ready:
        push_error("Gamestruments generation failed. Check the Output panel.")
        return
    # Starts at default initial section: "garage". Do not call game callbacks here.

func countdown_started() -> void:
    if music_ready and not music.set_race_state("grid", 0.0, 0.0, false):
        push_warning("set_race_state was rejected")

func race_updated(intensity: float, pressure: float, lap: int, total_laps: int) -> void:
    if not music_ready:
        return
    var accepted := music.set_race_state(
        "race",
        clampf(intensity, 0.0, 1.0),
        clampf(pressure, 0.0, 1.0),
        lap == total_laps)
    if not accepted:
        push_warning("set_race_state was rejected")

func race_finished(won: bool) -> void:
    if not music_ready:
        return
    var result := "win" if won else "loss"
    if not music.set_race_state("finish", 0.0, 0.0, false, result):
        push_warning("set_race_state was rejected")
```

The example uses the native default (original, six state-driven phases). To
match the Audio Lab Racing tour set `arrangement = "extended"` and `autoplay = true`
before `generate()` (Lab default for Racing is the 10-phase autoplay form).

`generate` result `true` means the score is ready (initial garage section). All
`set_race_state` calls are guarded and check their bool return. Intensity and
pressure are clamped to 0..1. Finish always sends 0/0/false + the win/loss
result (both outcomes intentionally select the same victory section). State
changes commit on bar boundaries using bar-aligned crossfades (bars can occur
inside phrases; no hard mid-phrase cut).

Use only with Racing recipe. `set_race_state` does not internally guard the
recipe — call it only on a Racing player.

## Suspense recipe (complete script)

Use a separate scene/player from any Racing usage. Same setup: root + exact-name
child + attach this full script to root + F6.

```gdscript
extends Node

@onready var music: GamestrumentsPlayer = $GamestrumentsPlayer
var music_ready := false

func _ready() -> void:
    music.project_secret = "my-game"
    music.recipe = "suspense"
    music.style = "terminal"        # terminal, cipher, noir, techno, or trance
    music.arrangement = "all-phases"  # all-phases (canonical tour) or seeded (composer)

    music_ready = music.generate("chapter-001")
    if not music_ready:
        push_error("Gamestruments generation failed. Check the Output panel.")
        return
    # Starts at "intro". No game callbacks invoked from _ready.

func scan_started() -> void:
    if music_ready and not music.set_trace_state("scan", 0.3, 0.2, 0.1):
        push_warning("set_trace_state was rejected")

func alarm_raised() -> void:
    if music_ready and not music.set_trace_state("alert", 0.9, 0.5, 0.4):
        push_warning("set_trace_state was rejected")

func chapter_finished() -> void:
    if music_ready and not music.set_trace_state("complete", 0.2, 0.3, 1.0):
        push_warning("set_trace_state was rejected")

func set_hold(held: bool) -> void:
    if not music_ready:
        return
    if not music.set_form_hold(held):
        push_warning("set_form_hold rejected")

func advance() -> void:
    if not music_ready:
        return
    if not music.advance_form():
        push_warning("advance_form rejected")

func cue(section: String) -> void:
    if not music_ready:
        return
    if not music.cue_section(section):
        push_warning("cue_section rejected")
```

Each public call is independently guarded and its bool return checked. Do not
combine hold + advance + cue inside one callback. The form auto-advances;
`set_form_hold(true)` freezes it, `advance_form` steps it, `cue_section` jumps
(and works on any generated score, form or not). `set_trace_state` accepts any
phase string (known ones affect selection); numeric args are 0..1 finite. See
`api.md` for exact native selection rules (there is no `progress`-only outro
rule; `extract`/`complete` or `progress >= 0.95` drive the endings).

`get_current_section()` is coarse (may report target during crossfade). No time
guarantees (no "X minutes", no "never mid-phrase" beyond the bar-aligned rule).

## Run the examples

`kit/examples/` is a self-contained Godot project (three independent reference
scenes, shared addon copy, no browser). Open `kit/examples/project.godot`; F5
runs only the playback example by design. To run 02 or 03, open its .tscn and
press F6. See `kit/examples/README.md`.

See `api.md`, `limitations.md`, and `troubleshooting.md` before shipping.

Determinism: the combination of seed + project_secret + style + palette/voices +
traits + generator version produces the identical score. Not seed alone.
