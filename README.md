# Gamestruments

Gamestruments is a music addon for Godot 4. You give it a seed and a style when
a level loads, and it builds a piece with several parts. While the game runs,
you tell it what is happening and it moves to another part at the next bar line.
It runs offline. There are no sound files to load, and there is no separate
editor to learn. The addon makes every sound while the game runs.

The same seed, settings and addon version always give the same music. Change the
seed and you get another piece. Gamestruments is free and open source under the
MIT license.

- Hear it: <https://gamestruments.gurisitos.games> (the Audio Lab, the same
  engine compiled for the browser)
- Get it: [GitHub releases](https://github.com/jfrader/gamestruments/releases) or
  [itch.io](https://gurisitosgames.itch.io/gamestruments-godot) (free)
- Kinds of music: Racing, Suspense and Adventure in Godot. Folklore is in the
  Audio Lab and the Rust engine; the Godot addon cannot play it yet.

## Use it in Godot

Copy `addons/gamestruments/` from a release zip into your project, restart
Godot, and add a `GamestrumentsPlayer` node. Keep the node name. Attach a script
like this to the scene root:

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

The addon runs on Godot 4.7.x on Linux x86_64, Windows x86_64, macOS arm64 and
x86_64, and Web. For a Web export, turn Extensions Support on and Thread Support
off.

Copy-paste Racing and Suspense scripts are in
[`kit/docs/quickstart.md`](kit/docs/quickstart.md). The full API, including
`set_adventure_state`, is in [`kit/docs/api.md`](kit/docs/api.md). There are
three runnable example scenes in [`kit/examples/`](kit/examples/README.md).

## Kinds of music

- Racing: the garage, the starting grid, the race, a position fight, the final
  lap and the finish. `extended` adds four more parts. Styles: `neon`, `funk`,
  `fusion`, `chip`.
- Suspense: one long piece for stealth, hacking or horror. By default, the seed
  decides the order of its parts. Styles: `terminal`, `cipher`, `noir`,
  `trance`.
- Adventure: camp, exploration, town, dungeon, combat, a boss, a sanctuary and
  the final victory, plus eight variants. Styles: `folk`, `dark`, `orchestral`.
- Folklore: guitar and bombo (a large drum) in the Audio Lab and the Rust
  engine. The Godot player cannot play it yet.

## Build and test

Rust 1.94.0 (selected by `rust-toolchain.toml`) and Node.js 24 are required.

```bash
# Rust: the music generator and the Godot addon
cargo test -p gamestruments-engine
cargo clippy -p gamestruments-engine --all-targets -- -D warnings
cargo build -p gamestruments-godot --release
```

The last command writes the addon library to `target/release`.

```bash
# Audio Lab and browser tests
npm ci
npm run dev        # start the Audio Lab
npm run check      # tests, typecheck, build and import checks
npm run test:e2e   # browser tests
```

The Audio Lab loads the engine from the compiled browser build in
`apps/demo/public/engine`. After changing `crates/engine`, rebuild and check it:

```bash
npm run wasm:build
npm run wasm:verify
```

To try a local build in your own project, copy
`crates/godot/gamestruments.gdextension` into `addons/gamestruments/` and the
built library into its `bin/` folder. To check the example project and the doc
snippets against a real Godot binary:

```bash
node tests/godot-package-smoke.mjs --godot <godot-binary> --library target/release/libgamestruments_godot.so
```

To write a folder of WAV files from the Rust engine:

```bash
cargo run -p gamestruments-engine --example render_listen -- target/listen
```

## Repository layout

- `crates/engine`: builds the music and makes the audio. Also compiled for the
  browser.
- `crates/godot`: the Godot addon and `GamestrumentsPlayer`.
- `kit/`: docs and the example Godot project shipped in releases.
- `apps/demo`: the browser Audio Lab.
- `packages/runtime`: TypeScript helpers used by the Audio Lab.
- `docs/`: design notes and music docs.
- `tools/`: build, package and verification scripts.
- `storefront/`: store listing copy.

## Releases

Pushing a `v*` tag runs the release workflow. It builds the addon for Linux,
Windows, macOS and Web, runs it in Godot and in a browser, packages
`gamestruments-<version>-godot4.zip`, checks the archive, and opens a draft
GitHub release.

## Contributing

Bug reports, ideas and pull requests are welcome. See
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## License and donations

Everything in this repository is MIT licensed, including use in closed-source
and commercial games; see [`LICENSE.md`](LICENSE.md). Third-party terms
(godot-rust is MPL-2.0) are in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)
and `licenses/`.

Gamestruments is free. If it helps your game, you can leave a donation on
[itch.io](https://gurisitosgames.itch.io/gamestruments-godot).
