# API Reference — GamestrumentsPlayer

`GamestrumentsPlayer` is the only public class buyers instantiate and script against. It extends `Node`.

All generation and synthesis happens inside the GDExtension. The node owns an internal mono synth (22050 Hz) that feeds a Godot `AudioStreamGeneratorPlayback` on the "Music" bus.

## Exported Properties (Inspector + Code)

Set these before or between calls to `generate`. Changes to traits/palette after generate affect the next `generate` call.

- `project_secret: String`
  - Per-title secret. Must be non-empty for `generate` to succeed.
  - Example: `"my-title-secret-42"`.
  - Do not use a public string such as the game name.

- `style: String`
  - One of: `"fusion"`, `"neon"`, `"funk"`, `"chip"`.
  - Default (in code): `"funk"`.
  - Selects the base instrument kit and timbre rules.

- Voice overrides (strings; empty string means "use style default"):
  - `melody_voice: String`
  - `harmony_voice: String`
  - `drive_voice: String`
  - `bass_voice: String`

- Trait values (floats, clamped to `[0.0, 1.0]` at generation time):
  - `energy: float` — default 0.62
  - `complexity: float` — default 0.6
  - `brightness: float` — default 0.52
  - `syncopation: float` — default 0.7

## Methods

- `generate(seed: String) -> void`
  - Generates the deterministic Pocket Circuit score from the current `project_secret` + `seed` + `style` + palette overrides + traits.
  - Must be called before any `set_race_state` calls (otherwise the transport is not ready).
  - Resets the internal tick and synth. Starts transport in the "garage" section.
  - Errors (via `godot_error`): unknown style, empty `project_secret`.
  - Determinism: identical inputs (secret, seed, style, palette fingerprint, traits, generator version) always produce the same score identity and event data.

- `set_race_state(phase: String, intensity: float, pressure: float, final_lap: bool) -> void`
  - Requests a state change. The transport quantizes to the next bar boundary and performs a musical crossover.
  - `phase`: string such as `"garage"`, `"grid"`, `"race"`, `"attack"`, `"final-lap"`, `"victory"` (or any value the adaptive rules understand). The six core sections generated are: garage, grid, cruise (Race Flow), attack (Position Fight), final-lap, victory (Finish).
  - `intensity`: 0..1 (affects density/velocity targets in the transport).
  - `pressure`: 0..1 (maps to `position_pressure` in the game state).
  - `final_lap`: boolean.
  - No-op if no score has been generated yet.
  - State changes are committed on bar boundaries; new sections begin at phrase bar zero.

## Observable Behavior Buyers Can Rely On

- Generation is deterministic for the tuple `(project_secret, seed, style, palette, energy, complexity, brightness, syncopation, generator version)`.
- State changes commit on bar boundaries.
- Audio is rendered at 22050 Hz mono internally and pushed as stereo-identical frames to the Godot generator playback. Route via the "Music" bus.
- The player creates a transient `AudioStreamPlayer` child named `"LiveStream"` (do not rely on its exact name in your own code beyond what the docs guarantee).

## What Is Not Part of the Supported Public API

- Any internal Rust modules, types (`PortableScore`, `AdaptiveTransport`, etc.), or symbols not exposed via the `GamestrumentsPlayer` GDScript surface.
- Exact byte-for-byte output across Godot/gdext patch releases (semantic + musical identity is the contract).
- Any editor plugins, UI, or authoring tools (none ship).

See `kit-contract.md` for the full buyer contract, non-goals, and compatibility notes.
