# Gamestruments — Adaptive Racing Music for Godot 4

Gamestruments generates deterministic, sample-free racing music inside a Godot game. Configure one `GamestrumentsPlayer`, call `generate(seed)` at level load, then drive bar-quantized changes from garage through grid, racing pressure, final lap, and victory.

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
- A self-contained Godot demo that exercises generation and all six racing sections.
- Buyer documentation, changelog, MIT license, third-party notices, and full dependency license texts.

No samples, Strudel code, browser Audio Lab, authoring UI, pre-rendered audio, or external services ship in the kit.

## Literal Product Claims

- The same generator version and `(project_secret, seed, style, palette, traits)` tuple produces the same score.
- Every generated score is validated inside the Rust engine before it reaches Godot.
- Runtime state changes cross between six generated racing sections on bar boundaries.
- The shipped runtime is a 22050 Hz mono synthesizer routed through Godot's audio buses.
- The addon works offline on the three supported desktop platform families.

The public browser Audio Lab uses a different Web Audio presentation layer with stereo room processing. It is an authoring surface, not an exact audio preview of this kit. Buyer-facing audio and video must be captured from the included Godot demo.

## Support

Best-effort support is available through the public comments section on the itch.io product page for reproducible problems within the advertised scope. Include your OS, Godot version, kit version, minimal reproduction, and complete Output-panel error. Use itch.io's purchase-support flow for purchase-specific or private matters.

Standard price: $24.99.
