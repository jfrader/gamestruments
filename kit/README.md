# Gamestruments — Adaptive Music for Godot 4

**Your game generates its own soundtrack.** Gamestruments writes deterministic,
sample-free adaptive music right inside Godot — no audio files to ship, no
authoring tool, no external services. Add one node, call `generate()` when a
level loads, and tell it what's happening in the game. It takes care of the rest,
moving between musical sections on bar boundaries.

**Try it in your browser:** <https://gamestruments.gurisitos.games>

That's the Audio Lab preview — the same generator with a browser audio layer, so
you can hear the range of styles instantly. The included Godot demo is the exact
in-game sound.

## Use It in Your Game (about five minutes)

### 1. Add the addon

Copy the archive's `addons/gamestruments/` folder into your project so this file
exists, then restart Godot:

```text
res://addons/gamestruments/gamestruments.gdextension
```

`GamestrumentsPlayer` now appears in the Create New Node dialog. You need Godot
4.7.x on Linux x86_64, Windows x86_64, or macOS arm64/x86_64.

### 2. Make some music

Add a `GamestrumentsPlayer` node and point a script at it:

```gdscript
extends Node

@onready var music: GamestrumentsPlayer = $GamestrumentsPlayer
var music_ready := false

func _ready() -> void:
    music.project_secret = "my-game"   # any stable name for your title
    music.style = "neon"               # neon, funk, fusion, or chip
    music_ready = music.generate("level-001")  # any seed; same seed = same score
    if not music_ready:
        push_error("Gamestruments generation failed")
```

That's it — your level has music. The same seed and settings always produce the
same score, so your levels sound consistent between runs.

### 3. Tell it what's happening in the race

Call `set_race_state` from your game events. Changes land on the next bar, so the
music never cuts mid-phrase:

```gdscript
func countdown_started() -> void:
    if music_ready:
        music.set_race_state("grid", 0.0, 0.0, false)

func race_updated(intensity: float, pressure: float, lap: int, total_laps: int) -> void:
    if music_ready:
        music.set_race_state(
            "race",
            clampf(intensity, 0.0, 1.0),  # 0–1: how fast or frantic it feels
            clampf(pressure, 0.0, 1.0),   # 0–1: how close the rival is
            lap == total_laps)            # final lap?

func race_finished(won: bool) -> void:
    if music_ready:
        music.set_race_state("finish", 0.0, 0.0, false, "win" if won else "loss")
```

How the engine picks a section:

| You send | You hear |
|---|---|
| `finish` + `"win"` | `victory` |
| `final_lap = true` | `final-lap` (beats attack) |
| `pressure >= 0.68` or `intensity >= 0.72` | `attack` |
| `phase = "race"` / `"grid"` / `"garage"` | `cruise` / `grid` / `garage` |
| any other `finish` | `victory` |

### 4. Mixing (optional)

The player routes to a Godot audio bus named `Music` when you have one, and falls
back to `Master` when you don't — so a fresh project makes sound before you touch
the audio panel.

## Second Recipe: Suspense

The same player ships **Suspense**, a song-form recipe for long tense sessions
(infiltration, hacking, horror). Set `recipe = "suspense"` before `generate`,
choose a suspense style, and drive it with `set_trace_state`:

```gdscript
music.recipe = "suspense"
music.style = "terminal"        # terminal, cipher, or noir
music.arrangement = "original"  # or "extended" / "theme"
music.generate("chapter-001")
music.set_trace_state("scan", 0.3, 0.2, 0.1)  # phase, heat, focus, progress
```

The song form advances on its own between sections (Handshake, Scan, Breach,
Decrypt, Closed Session, and more). Gameplay can hold or move it with
`set_form_hold`, `advance_form`, `is_form_held`, and `cue_section`. The full
section list, selection rules, and trait mapping are in `kit/docs/api.md`.

## Third Recipe: Adventure

The same player ships **Adventure**, an eight-section fantasy quest arc. Set
`recipe = "adventure"` before `generate`, choose a style, and pass the area
phase plus discovery, threat, and quest progress:

```gdscript
music.recipe = "adventure"
music.style = "folk"            # folk, dark, or orchestral
music.generate("world-3")
music.set_adventure_state("explore", 0.4, 0.1, false)   # discovery >= 0.3 -> explore
music.set_adventure_state("town", 0.5, 0.1, false)      # town
music.set_adventure_state("combat", 0.2, 0.9, false)    # combat + threat >= 0.85 -> boss
music.set_adventure_state("explore", 0.9, 0.1, false)   # discovery >= 0.85 -> sanctuary
music.set_adventure_state("explore", 0.5, 0.1, true)    # quest complete -> victory
```

Eight sections — Camp, Explore, Town, Dungeon, Combat, Boss, Sanctuary, and
Victory — crossfade on the next bar. Camp, Dungeon, Boss, and Sanctuary are 16
bars; Explore, Town, Combat, and Victory are 32. Quest completion always wins.
Full list in `kit/docs/api.md`.

## API at a Glance

| Member | What it does |
|---|---|
| `project_secret: String` | Stable name for your title. Required, not a credential. |
| `recipe: String` | `racing` (default), `suspense`, or `adventure`. |
| `arrangement: String` | Suspense only: `original` (default), `extended`, or `theme`. |
| `autoplay: bool` | Racing and Adventure only (default `false`). When true, attaches a song form that tours the recipe's sections automatically; when false, generation is state-driven. |
| `style: String` | Racing: `neon`, `funk`, `fusion`, `chip`. Suspense: `terminal`, `cipher`, `noir`. Adventure: `folk`, `dark`, `orchestral`. |
| `melody_voice`, `harmony_voice`, `drive_voice`, `bass_voice` | Racing only. Optional voice overrides; empty uses the style default. |
| `energy`, `complexity`, `brightness`, `syncopation: float` | Optional traits from `0.0` to `1.0`. Suspense reads them as tension, heat, mystery, and pulse; Adventure as danger, mystery, wonder, and motion. |
| `generate(seed: String) -> bool` | Makes the score, resets playback, starts at the recipe's first section. Check the result. |
| `set_race_state(phase, intensity, pressure, final_lap, finish_result = "none") -> bool` | Racing: requests a section. Commits on the next bar. |
| `set_trace_state(phase, heat, focus, progress) -> bool` | Suspense: requests a section from trace state. Commits on the next bar. |
| `set_adventure_state(area_phase, discovery, threat, quest_complete) -> bool` | Adventure: requests a section for the area. Commits on the next bar. |
| `cue_section`, `set_form_hold`, `advance_form`, `is_form_held`, `get_current_section` | Form controls for scores with a form (Suspense, and Racing/Adventure when `autoplay` is on). See `kit/docs/api.md`. |

Every member, supported voice, and error case is documented in
`kit/docs/api.md`. A complete walkthrough is in `kit/docs/quickstart.md`.

## See It Working

- **Browser preview:** <https://gamestruments.gurisitos.games> — hear the styles
  without installing anything.
- **Included Godot demo:** open `kit/demo/` as a Godot project and press F5. It's
  a four-circuit race series; each circuit regenerates the score with a different
  style, so you can compare all four before writing code. An intro card explains
  the point, then A/D or PREV/NEXT switches circuits in the garage.

Controls and integration notes: `kit/demo/README.md`. The reference integration
is `kit/demo/race_music.gd`; the game rules live in `kit/demo/race_model.gd`.

## What's in the Archive

- `addons/gamestruments/` — Linux, Windows, and universal macOS libraries plus
  the GDExtension descriptor. This is what you drop into your game.
- `kit/demo/` — the playable Godot demo, with its own addon copy.
- `kit/docs/` — quickstart, API, limitations, and troubleshooting.
- `crates/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` — full source and
  pinned rebuild inputs.
- `LICENSE.md`, `THIRD_PARTY_NOTICES.md`, `licenses/` — license terms.
- `RELEASE-MANIFEST.json` — build provenance and library hashes.
- `CHANGELOG.md` — release history.

## Rebuild and Test

From the archive root:

```sh
cargo test --workspace --locked
cargo build --workspace --all-targets --release --locked
```

Cargo downloads the exact checksummed dependencies in `Cargo.lock` unless they're
already cached or vendored.

## License

The runtime, addon descriptor, docs, and demo integration are MIT licensed — use
them in closed-source commercial games. See `LICENSE.md`; third-party terms are
in `THIRD_PARTY_NOTICES.md` and `licenses/`.

## Support

Questions or bugs? Post in the public comments on the itch.io product page and
include your kit version, Godot version, OS, what you did, and the full Output
text. For purchase-specific matters, use itch.io's purchase-support flow.
