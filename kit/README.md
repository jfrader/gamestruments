# Gamestruments — Adaptive Music for Godot 4

**Procedural music that adapts to your game.** Gamestruments generates
deterministic, sample-free music inside Godot — no audio files, no authoring
tool, no external services. Add one node, call `generate()` when a scene loads,
then tell it what's happening; it blends between sections on bar boundaries.

Try the Audio Lab preview at <https://gamestruments.gurisitos.games> — same
engine as the native addon, parity-tested, including HTML5. Open
`kit/examples/` to run it inside Godot.

## Use It in Your Game

### 1. Add the addon

Place the archive's `gamestruments/` addon inside your project's `addons/`
directory, preserving other addons. Create `addons/` next to `project.godot` if
needed. Restart Godot and wait for import to finish. This file should exist:

```text
res://addons/gamestruments/gamestruments.gdextension
```

Godot loads only the platform-matching binary from the folder. The shipped
`addons/gamestruments/bin/` contains the three native libraries (Linux x86_64,
Windows x86_64, macOS x86_64+arm64). Keep the folder intact when copying it.

`GamestrumentsPlayer` now appears in the Create New Node dialog. Requires Godot
4.7.x on Linux x86_64, Windows x86_64, or macOS arm64/x86_64.

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

Run the scene to hear music. A fixed generator version and identical settings reproduce the same score.

### 3. Drive it from game events

Call `set_race_state` from your countdown, telemetry, and finish handlers
(Racing), `set_trace_state` plus form controls (Suspense), or
`set_adventure_state` (Adventure). Changes commit on the next bar boundary
(bar-aligned crossfade). These calls are not auto-wired by name — connect them
to your own events and check each bool return. Complete guarded examples are in
`kit/docs/quickstart.md`; full selection rules are in `kit/docs/api.md`.

The player routes to a Godot `Music` bus when you have one, and falls back to
`Master` when you don't, so a fresh project makes sound before you touch audio
settings.

## Recipes

One player, three recipes. **Racing** is state-driven race loops (`racing`;
styles neon, funk, fusion, chip). **Suspense** is a song-form arc for tense
sessions (`suspense`; terminal, cipher, noir). **Adventure** is an eight-section
fantasy quest (`adventure`; folk, dark, orchestral). Set `recipe` before
`generate`; each generated score belongs to one recipe. Sections and selection
rules are in `kit/docs/api.md`; complete Racing and Suspense scripts are in
`kit/docs/quickstart.md`.

`arrangement` is `original` (default) or `extended` for Racing, and `original`
(default), `extended`, or `theme` for Suspense; Adventure ignores it. `autoplay`
is Racing/Adventure only, defaults to `false` (state-driven), and when `true`
tours the recipe's sections. The Audio Lab uses `autoplay = true` with Racing
`extended`; the native examples use the state-driven defaults.

## See It Working

- **Browser / HTML5:** <https://gamestruments.gurisitos.games> — same engine
  as the native addon, parity-tested.
- **Native examples:** open `kit/examples/project.godot`. F5 runs the playback
  reference by design; open another `.tscn` and press F6. See
  `kit/examples/README.md`.

## Requirements and Platforms

- Godot 4.7.x on Linux x86_64, Windows x86_64, or macOS arm64/x86_64.
- Godot 4 only; other engines and custom adapters are not supported.
- Offline runtime; no external audio assets, services, middleware, or telemetry.

## What's in the Archive

One ZIP contains the addon, native examples, documentation, and source.

- `addons/gamestruments/` — the full directory (GDExtension + bin/ with all
  platform libraries). Copy the entire folder; Godot loads the matching one.
- `kit/examples/` — a self-contained Godot project with three reference scenes
  and its own addon copy.
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

Cargo downloads the exact checksummed dependencies in `Cargo.lock` unless
they're already cached or vendored.

## License

The runtime, addon descriptor, docs, and example integration are MIT licensed —
use them in closed-source commercial games. See `LICENSE.md`; third-party terms
are in `THIRD_PARTY_NOTICES.md` and `licenses/`.

## Support

Questions or bugs? Post in the public comments on the itch.io product page with
your kit version, Godot version, OS, and full Output text.
