# Gamestruments — Adaptive Music for Godot 4

Gamestruments generates music inside Godot from a seed. There are no audio
files, no authoring tool and no external services. Add one node, call
`generate()` when a scene loads, then tell it what is happening in the game. It
blends between sections on bar boundaries.

## Use it in your game

### 1. Add the addon

Copy the archive's `gamestruments/` addon into your project's `addons/`
directory, next to any other addons. Create `addons/` next to `project.godot`
if it does not exist. Restart Godot and wait for the import to finish. This
file should exist:

```text
res://addons/gamestruments/gamestruments.gdextension
```

`addons/gamestruments/bin/` holds the three native libraries (Linux x86_64,
Windows x86_64, macOS x86_64+arm64) and Godot loads the one for its platform.
Copy the folder whole. `GamestrumentsPlayer` then appears in the Create New
Node dialog.

### 2. Make some music

Add a `GamestrumentsPlayer` child node named exactly `GamestrumentsPlayer`.
Attach a script to the scene's root node:

```gdscript
extends Node

@onready var music: GamestrumentsPlayer = $GamestrumentsPlayer
var music_ready := false

func _ready() -> void:
    music.project_secret = "my-game"  # non-empty stable per-title namespace
    music.recipe = "racing"
    music.style = "neon"              # neon, funk, fusion, or chip
    music.arrangement = "original"    # native default; "extended" adds four sections
    music.autoplay = false            # native default; true tours the sections
    music_ready = music.generate("level-001")
    if not music_ready:
        push_error("Gamestruments generation failed")
```

Run the scene. The same generator version and settings always produce the same
score.

### 3. Drive it from game events

Call `set_race_state` from your countdown, telemetry and finish handlers
(Racing), `set_trace_state` plus the form controls (Suspense), or
`set_adventure_state` (Adventure). Changes blend on bar boundaries. While a blend
runs, the latest music change and latest section request wait. Connect these calls to your own events
and check each bool return. Complete examples are in `kit/docs/quickstart.md`;
the selection rules are in `kit/docs/api.md`.

The player routes to a `Music` bus when the project has one and to `Master`
otherwise.

## Recipes

One player, three recipes. Racing is state-driven race loops (`racing`; styles
neon, funk, fusion, chip). Suspense is a song-form arc for tense sessions
(`suspense`; terminal, cipher, noir, trance). Adventure covers eight fantasy
quest situations with sixteen musical sections (`adventure`; folk, dark,
orchestral). Set `recipe` before
`generate`; each score belongs to one recipe.

Racing offers `original` (native default), `extended`, `all-phases`, and
`seeded` arrangements. Suspense offers `all-phases` and `seeded` (default);
Adventure ignores `arrangement` in Godot. `autoplay` defaults to `false` and
adds a section tour to Original/Extended Racing or Adventure when enabled.
Suspense and composed Racing already have a form. The Audio Lab defaults to
Seeded; the native Racing game-signals example uses state-driven Original.

## Examples

- Browser / HTML5: <https://gamestruments.gurisitos.games>.
- Native: open `kit/examples/project.godot`. F5 runs the playback reference;
  open another `.tscn` and press F6 for the others. See `kit/examples/README.md`.

## Requirements

- Godot 4.7.x on Linux x86_64, Windows x86_64 or macOS arm64/x86_64.
- Godot 4 only. Other engines and custom adapters are not supported.
- Runs offline. No external audio assets, services, middleware or telemetry.

## What is in the archive

- `addons/gamestruments/`: the addon, GDExtension plus `bin/` with all platform
  libraries.
- `kit/examples/`: a Godot project with three reference scenes and its own
  addon copy.
- `kit/docs/`: quickstart, API, limitations and troubleshooting.
- `crates/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`: full source and
  pinned rebuild inputs.
- `LICENSE.md`, `THIRD_PARTY_NOTICES.md`, `licenses/`: licence terms.
- `RELEASE-MANIFEST.json`: build provenance and library hashes.
- `CHANGELOG.md`: release history.

## Rebuild and test

From the archive root:

```sh
cargo test --workspace --locked
cargo build --workspace --all-targets --release --locked
```

Cargo fetches the checksummed dependencies in `Cargo.lock` unless they are
cached or vendored.

## Licence

The runtime, addon descriptor, docs and example integration are MIT licensed,
including use in closed-source commercial games. See `LICENSE.md`; third-party
terms are in `THIRD_PARTY_NOTICES.md` and `licenses/`.

## Support

Post bugs and questions in the comments on the itch.io product page with your
kit version, Godot version, OS and the full Output text.
