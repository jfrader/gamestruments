# API Reference — GamestrumentsPlayer

`GamestrumentsPlayer` is the kit's only supported public class. It extends `Node` and owns the generated score, adaptive transport, synthesizer, and internal Godot audio player.

## Exported Properties

Set these before calling `generate`. Later changes apply to the next generation call.

- `project_secret: String` — required non-empty per-title namespace. It separates otherwise identical seeds between games, but it is embedded in the game and is not a security credential.
- `style: String` — `fusion`, `neon`, `funk`, or `chip`; defaults to `funk`.
- `melody_voice: String`
- `harmony_voice: String`
- `drive_voice: String`
- `bass_voice: String`
  - Empty uses the selected style's default.
  - Supported note voices: `warm`, `glass`, `pulse`, `bass`, `pluck`, `chip`, `epiano`, `organ`, `supersaw`, `triangle`.
- `energy: float` — defaults to `0.62`.
- `complexity: float` — defaults to `0.60`.
- `brightness: float` — defaults to `0.52`.
- `syncopation: float` — defaults to `0.70`.
  - Finite trait values are clamped to `0.0..1.0` during generation.

## `generate`

```gdscript
var generated: bool = player.generate("level-001")
```

Generates and validates the deterministic racing score for the current property values, resets transport and synthesis, and starts at `garage`.

Returns `true` on success. Returns `false` and emits a descriptive Godot error for an empty project namespace, unsupported style or voice, non-finite data, or invalid generated score. Do not request state changes after a failed generation.

## `set_race_state`

```gdscript
var accepted: bool = player.set_race_state(
    "race",
    0.8,
    0.4,
    false,
    "none",
)
```

Signature:

```text
set_race_state(
  phase: String,
  intensity: float,
  pressure: float,
  final_lap: bool,
  finish_result: String = "none",
) -> bool
```

Returns `false` with a Godot error when no score has been generated or when intensity/pressure are non-finite or outside `0.0..1.0`. Otherwise it returns `true` after accepting the request. The musical change commits on a bar boundary.

Selection priority:

1. `phase = "finish"` and `finish_result = "win"` selects `victory`.
2. `final_lap = true` selects `final-lap`.
3. `pressure >= 0.68` or `intensity >= 0.72` selects `attack`.
4. `phase = "race"`, `"grid"`, or `"garage"` selects `cruise`, `grid`, or `garage`.
5. Unknown phases fall back to the score's default section.

## Observable Contract

- Generation is deterministic for a specific generator version and input tuple.
- Generated scores contain `garage`, `grid`, `cruise`, `attack`, `final-lap`, and `victory`.
- State changes are quantized to bar boundaries and new sections start at phrase bar zero.
- Audio is synthesized at 22050 Hz mono and pushed as identical left/right frames to an internal `AudioStreamPlayer` routed to the `Music` bus.

Internal Rust types, child-node names, score serialization, and exact bytes across different generator versions are not supported public API.
