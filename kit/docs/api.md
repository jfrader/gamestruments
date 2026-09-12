# API Reference — GamestrumentsPlayer

`GamestrumentsPlayer` is the kit's only supported public class. It extends `Node` and owns the generated score, adaptive transport, synthesizer, and internal Godot audio player.

## Recipes

The player ships two recipes. Set `recipe` before calling `generate`; a generated score belongs to one recipe.

| Recipe | `recipe` | Drive it with | Styles |
|---|---|---|---|
| Racing | `"racing"` (default) | `set_race_state` | `fusion`, `neon`, `funk`, `chip` |
| Suspense (song-form) | `"suspense"` | `set_trace_state` plus form controls | `terminal`, `cipher`, `noir` |

## Exported Properties

Set these before calling `generate`. Later changes apply to the next generation call.

- `project_secret: String` — required non-empty per-title namespace. It separates otherwise identical seeds between games, but it is embedded in the game and is not a security credential.
- `recipe: String` — `racing` (default) or `suspense`.
- `arrangement: String` — Suspense only: `original` (default), `extended` (gameplay form with every phase), or `theme` (additive title bed: hats enter early, layers stay, drop holds). Ignored by Racing.
- `style: String` — per recipe:
  - Racing: `fusion`, `neon`, `funk`, or `chip`; defaults to `funk`.
  - Suspense: `terminal`, `cipher`, or `noir`; empty defaults to `terminal`.
- `melody_voice: String`, `harmony_voice: String`, `drive_voice: String`, `bass_voice: String` — Racing only. Empty uses the selected style's default. Supported note voices: `warm`, `glass`, `pulse`, `bass`, `pluck`, `chip`, `epiano`, `organ`, `supersaw`, `triangle`. Suspense ignores these and uses its own per-style timbres.
- `energy: float` — defaults to `0.62`.
- `complexity: float` — defaults to `0.60`.
- `brightness: float` — defaults to `0.52`.
- `syncopation: float` — defaults to `0.70`.
  - Finite trait values are clamped to `0.0..1.0` during generation.
  - Trait meaning depends on the recipe: Racing reads energy, complexity, brightness, and syncopation; Suspense reads the same four properties as tension, heat, mystery, and pulse.

## `generate`

```gdscript
var generated: bool = player.generate("level-001")
```

Generates and validates the deterministic score for the current property values, resets transport and synthesis, and starts at the recipe's first section — `garage` for Racing, `intro` (Handshake) for Suspense.

Returns `true` on success. Returns `false` and emits a descriptive Godot error for an empty project namespace, unsupported style, arrangement, or voice, non-finite data, or an invalid generated score. Do not request state changes after a failed generation.

## Racing — `set_race_state`

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
5. Any remaining `phase = "finish"` request selects `victory`.
6. Unknown phases fall back to the score's default section.

## Suspense — `set_trace_state`

```gdscript
var accepted: bool = player.set_trace_state("scan", 0.4, 0.2, 0.1)
```

Signature:

```text
set_trace_state(
  phase: String,
  heat: float,
  focus: float,
  progress: float,
) -> bool
```

Returns `false` with a Godot error when no score has been generated or when heat, focus, or progress are non-finite or outside `0.0..1.0`. Otherwise it returns `true` after accepting the request.

`phase` is a free string; the documented values are `boot`, `scan`, `exploit`, `alert`, `extract`, and `complete`. Unknown phases still match the numeric thresholds. `progress` is the run's overall completion (`0.0..1.0`), `heat` is danger/action intensity, and `focus` is how locked-in the player is.

Selection priority:

| Condition | Section | Behavior |
|---|---|---|
| `phase = "complete"` or `progress >= 0.95` | `coda` (Closed Session) | Holds |
| `phase = "extract"` | `outro` (Disconnect) | Holds |
| `heat >= 0.75` | `bridge` (Complication) | One-shot cue |
| `phase = "alert"` | `bridge` (Complication) | One-shot cue |
| Extended only: `progress >= 0.8` | `outro` (Disconnect) | Holds |
| `phase = "exploit"` and `focus >= 0.7` | `chorus` (Breach) | One-shot cue |

- One-shot cues re-arm once the form leaves the cued section, so a later `alert` can fire again.
- Holds stay until another state changes them.
- `original` (default) has no progress-based `outro`; `extended` adds it and a longer arrangement; `theme` is a title bed that does not use Break and loops on the drop.
- Changes commit on a bar boundary.

## Suspense — Form Controls

```gdscript
player.set_form_hold(true)     # freeze the song form at the current step
player.advance_form()          # move to the next form step
player.cue_section("chorus")   # jump to a specific section on the next bar
var section: String = player.get_current_section()
```

- `cue_section(section: String) -> bool` — requests any section id in the score. Returns `false` with a Godot error for an unknown section or before a successful `generate`.
- `set_form_hold(held: bool) -> bool` — freezes or resumes the automatic song form. Returns `false` when the score has no form (Racing).
- `advance_form() -> bool` — moves to the next form step; returns `false` at the end of the form or when there is no form.
- `is_form_held() -> bool` — whether the form is currently frozen.
- `get_current_section() -> String` — the section currently playing, or an empty string before generation.

## Suspense Sections

Base sections (`original`):

| Id | Label |
|---|---|
| `intro` | Handshake |
| `verse` | Scan |
| `pre-chorus` | Approach |
| `chorus` | Breach |
| `break` | Break |
| `verse-b` | Second Pass |
| `post-chorus` | Echo |
| `interlude` | Wait State |
| `bridge` | Complication |
| `bridge-b` | Other Hall |
| `solo` | Decrypt |
| `chorus-final` | Full Breach |
| `outro` | Disconnect |
| `coda` | Closed Session |

`extended` keeps every base section and adds `scan-ii` (Scan II), `breach-ii` (Breach II), and `anomaly` (Anomaly), with longer beds for verse, verse-b, chorus, chorus-final, bridge, and solo.

## Observable Contract

- Generation is deterministic for a specific generator version and input tuple.
- Racing scores contain `garage`, `grid`, `cruise`, `attack`, `final-lap`, and `victory`.
- Suspense scores contain the base sections above; `extended` adds `scan-ii`, `breach-ii`, and `anomaly`.
- State changes are quantized to bar boundaries and new sections start at phrase bar zero.
- Audio is synthesized at 22050 Hz mono and pushed as identical left/right frames to an internal `AudioStreamPlayer`. It uses the `Music` bus when present and otherwise falls back to `Master`.

Internal Rust types, child-node names, score serialization, and exact bytes across different generator versions are not supported public API.
