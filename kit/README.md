# Gamestruments — Adaptive Racing Music for Godot 4

Gamestruments generates deterministic, sample-free racing music inside your Godot
game: no audio files, no authoring tool, no external services. One
`GamestrumentsPlayer` node owns the score and crossfades between six sections as
your race state changes.

**Start here if you bought this kit to use it in your game.** The playable demo
and full archive contents are documented further down.

## Use It in Your Game

### 1. Install the addon

Copy the archive's root `addons/gamestruments/` folder into your project so these
paths exist, then restart Godot:

```text
res://addons/gamestruments/gamestruments.gdextension
res://addons/gamestruments/bin/libgamestruments_godot.so
res://addons/gamestruments/bin/gamestruments_godot.dll
res://addons/gamestruments/bin/libgamestruments_godot.dylib
```

`GamestrumentsPlayer` then appears in the Create New Node dialog. Requires Godot
4.7.x on Linux x86_64, Windows x86_64, or macOS arm64/x86_64.

### 2. Generate a score at level load

Add a `GamestrumentsPlayer` node to your scene and configure it before calling
`generate`:

```gdscript
extends Node

@onready var music: GamestrumentsPlayer = $GamestrumentsPlayer
var music_ready := false

func _ready() -> void:
    music.project_secret = "my-game"   # stable per-title namespace, not a secret
    music.style = "neon"               # fusion | neon | funk | chip
    # Optional: melody_voice, harmony_voice, drive_voice, bass_voice,
    # energy, complexity, brightness, syncopation.
    music_ready = music.generate("level-001")  # deterministic level seed
    if not music_ready:
        push_error("Gamestruments score generation failed")
```

The same `(project_secret, seed, style, voices, traits)` always produces the same
score. You can set these properties in the inspector instead of code.

### 3. Drive it from gameplay

Call `set_race_state` as your race changes. The engine commits changes on the next
musical bar, so requests never cut a phrase:

```gdscript
func countdown_started() -> void:
    if music_ready:
        music.set_race_state("grid", 0.0, 0.0, false)

func race_updated(intensity: float, pressure: float, lap: int, total_laps: int) -> void:
    if music_ready:
        music.set_race_state(
            "race",
            clampf(intensity, 0.0, 1.0),  # speed / boost / how frantic it feels
            clampf(pressure, 0.0, 1.0),   # how close the rival is
            lap == total_laps)            # final lap

func race_finished(won: bool) -> void:
    if music_ready:
        music.set_race_state("finish", 0.0, 0.0, false, "win" if won else "loss")
```

Section selection priority:

| Your request | Section |
|---|---|
| `finish` + `"win"` | `victory` |
| `final_lap = true` | `final-lap` (beats attack) |
| `pressure >= 0.68` or `intensity >= 0.72` | `attack` |
| `phase = "race"` / `"grid"` / `"garage"` | `cruise` / `grid` / `garage` |
| any other `finish` | `victory` |

### 4. Optional audio mixing

The player routes to a Godot audio bus named `Music` when one exists and falls
back to `Master`, so a fresh project produces audio before any bus setup.

### API at a glance

| Member | Purpose |
|---|---|
| `project_secret: String` | Stable per-title namespace. Required, not a credential. |
| `style: String` | `fusion`, `neon`, `funk`, or `chip`. |
| `melody_voice`, `harmony_voice`, `drive_voice`, `bass_voice` | Optional voice overrides; empty uses the style default. |
| `energy`, `complexity`, `brightness`, `syncopation: float` | Optional traits, clamped to `0.0..1.0`. |
| `generate(seed: String) -> bool` | Generates, validates, resets transport, starts at `garage`. |
| `set_race_state(phase, intensity, pressure, final_lap, finish_result = "none") -> bool` | Requests a section; commits on a bar boundary. |

Full details, supported voices, and error behavior: `kit/docs/api.md`. A complete
walkthrough: `kit/docs/quickstart.md`.

## See It Working

Open `kit/demo/` as a Godot project (its addon copy is already installed) and
press F5. The demo is a four-circuit race series; each circuit regenerates the
score with a different shipped style, so you can hear all four before writing any
code. An intro card states the point, then the garage switches circuits with
A/D or PREV/NEXT.

- Controls and integration notes: `kit/demo/README.md`.
- The reference integration is `kit/demo/race_music.gd`; game rules live in
  `kit/demo/race_model.gd`.

## Archive Contents

- `addons/gamestruments/` — Linux x86_64, Windows x86_64, and universal macOS
  arm64/x86_64 libraries plus the GDExtension descriptor.
- `kit/demo/` — self-contained Godot demo with an identical addon copy.
- `kit/docs/` — quickstart, API, limitations, and troubleshooting documentation.
- `crates/`, `catalog/`, `Cargo.toml`, `Cargo.lock`, and
  `rust-toolchain.toml` — source, fixture, and pinned Rust rebuild inputs.
- `LICENSE.md`, `THIRD_PARTY_NOTICES.md`, and `licenses/` — first- and
  third-party license terms and attribution.
- `RELEASE-MANIFEST.json` — source identity, tool versions, workflow provenance,
  and SHA-256 hashes for each native library.
- `CHANGELOG.md` — release history.

## Rebuild and Test

From the archive root:

```sh
cargo test --workspace --locked
cargo build --workspace --all-targets --release --locked
```

Cargo downloads the exact checksummed dependencies in `Cargo.lock` unless they
are already cached or separately vendored.

## License and Provenance

The Rust runtime, addon descriptor, buyer documentation, and demo integration
files are MIT licensed; see `LICENSE.md`. Third-party terms and attribution are
in `THIRD_PARTY_NOTICES.md` and `licenses/`.

Inspect `RELEASE-MANIFEST.json` before treating an archive as a release. Its
`provenance` and `source.dirty` fields distinguish tagged release builds from
pull-request, workflow-dispatch, and local candidates.

## Support

Report reproducible defects in the public comments section on the itch.io
product page. Include the kit version, exact Godot version, operating system
and architecture, minimal reproduction steps, and complete Godot Output text.
Use itch.io's purchase-support flow for purchase-specific or private matters.
