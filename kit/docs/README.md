# Gamestruments — Adaptive Music for Godot 4

Gamestruments generates deterministic, sample-free adaptive music inside a Godot game. Configure one `GamestrumentsPlayer`, call `generate(seed)` at level load, then drive bar-quantized changes through the sections your recipe defines — the Racing racing recipe covers garage, grid, cruise, attack, final lap, and victory.

New to the kit? The archive root `README.md` has a copy-paste integration with the exact properties and calls. This folder is the reference set. Prefer to hear it first? Try the browser preview at <https://gamestruments.gurisitos.games>.

The Rust core is MIT licensed and its complete rebuildable source is included.

## Requirements

- Godot 4.7.x.
- Linux x86_64, Windows x86_64, or macOS arm64/x86_64.
- No external audio assets, services, middleware, or telemetry.

## Quickstart

1. Copy `addons/gamestruments/` from the archive into your project at `res://addons/gamestruments/`.
2. Restart Godot and add a `GamestrumentsPlayer` node.
3. Set a non-empty `project_secret`, choose `fusion`, `neon`, `funk`, or `chip`, and optionally adjust voices and traits.
4. Call `generate("your-level-seed")` and check its boolean result.
5. Call `set_race_state(phase, intensity, pressure, final_lap, finish_result)` as gameplay changes.
6. Add an audible `Music` audio bus when you want separate music mixing. Until
   it exists, the player safely routes audio to `Master`.

See `quickstart.md` for a complete integration and `api.md` for the supported surface.

## What Ships

- One cross-platform addon with Linux, Windows, and universal macOS native libraries.
- Full Rust source, workspace manifests, lockfile, and pinned toolchain.
- Buyer documentation, changelog, MIT license, third-party notices, and full dependency license texts.
- Night Circuit, a self-contained playable four-circuit Godot race series with a rival,
  boost, and gameplay-driven music, as an optional reference integration (your game
  only needs the addon). A separate `race_music.gd` demonstrates the integration.
  See `../demo/README.md` for keyboard controls.

No samples, Strudel code, browser Audio Lab, authoring UI, pre-rendered audio, or external services ship in the kit.

## Literal Product Claims

- The same generator version and `(project_secret, seed, style, palette, traits)` tuple produces the same score.
- Every generated score is validated inside the Rust engine before it reaches Godot.
- Runtime state changes cross between six generated racing sections on bar boundaries.
- The shipped runtime is a 22050 Hz mono synthesizer routed through Godot's audio buses.
- The addon works offline on the three supported desktop platform families.

The browser preview at <https://gamestruments.gurisitos.games> uses a different Web Audio presentation layer with stereo room processing. It previews the musical range, not the exact kit mix; buyer-facing audio and video must be captured from the included Godot demo.

## Support

Best-effort support is available through the public comments section on the itch.io product page for reproducible problems within the advertised scope. Include your OS, Godot version, kit version, minimal reproduction, and complete Output-panel error. Use itch.io's purchase-support flow for purchase-specific or private matters.

Standard price: $12.99.
