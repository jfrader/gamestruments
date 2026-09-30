# API Reference — GamestrumentsPlayer

`GamestrumentsPlayer` is the kit's only supported public class. It extends `Node` and owns the generated score, adaptive transport, synthesizer, and internal Godot audio player.

## Recipes

The player ships three recipes. Set `recipe` before calling `generate`; a generated score belongs to one recipe.

| Recipe | `recipe` | Drive it with | Styles |
|---|---|---|---|
| Racing | `"racing"` (default) | `set_race_state` | `fusion`, `neon`, `funk`, `chip` |
| Suspense (song-form) | `"suspense"` | `set_trace_state` plus form controls | `terminal`, `cipher`, `noir`, `techno`, `trance` |
| Adventure (fantasy quest) | `"adventure"` | `set_adventure_state` | `folk`, `dark`, `orchestral` |

## Exported Properties

Set these before calling `generate`. Later changes apply to the next generation call.

- `project_secret: String` — required non-empty per-title namespace. It separates otherwise identical seeds between games, but it is embedded in the game and is not a security credential.
- `recipe: String` — `racing` (default), `suspense`, or `adventure`.
- `arrangement: String` — every recipe accepts `all-phases` (each section once in canonical order) or `seeded` (the composer picks count, roles, order and loop point from the seed); `seeded` is the Audio Lab default. Racing additionally accepts `original` (its native default) and `extended` (ten-section race). Retired Suspense names (`original`/`extended`/`theme`) resolve to `seeded`.
- `autoplay: bool` — Racing and Adventure only (default `false`). When `true`, attaches a song form that tours the recipe's sections automatically; when `false`, generation is state-driven. The Audio Lab uses `true` for Racing/Adventure. Ignored by Suspense.
- `style: String` — per recipe:
  - Racing: `fusion`, `neon`, `funk`, or `chip`; unset (or default init) is `funk`. Explicit empty string for Racing fails generation.
  - Suspense: `terminal`, `cipher`, or `noir`; empty defaults to `terminal`.
  - Adventure: `folk`, `dark`, or `orchestral`; empty defaults to `folk`.
- `melody_voice: String`, `harmony_voice: String`, `drive_voice: String`, `bass_voice: String` — Racing only. Empty uses the selected style's default. Supported note voices (16): `warm`, `glass`, `pulse`, `bass`, `pluck`, `chip`, `epiano`, `organ`, `supersaw`, `triangle`, `felt`, `dusk`, `harp`, `recorder`, `vielle`, `bell`. Racing voice overrides use them; Adventure uses its own acoustic timbres internally (synthesized, not samples); Suspense ignores the voice properties and uses per-style timbres.
- `energy: float` — defaults to `0.62`.
- `complexity: float` — defaults to `0.60`.
- `brightness: float` — defaults to `0.52`.
- `syncopation: float` — defaults to `0.70`.
  - Pass finite values in `0.0..1.0`. Finite values are clamped to that range during generation. Suspense and Adventure replace non-finite traits with `0.5`; do not rely on non-finite input handling for Racing (may fail to produce a valid score).
  - Trait meaning depends on the recipe: Racing reads energy, complexity, brightness, and syncopation; Suspense reads the same four properties as tension, heat, mystery, and pulse; Adventure reads them as danger, mystery, wonder, and motion.

## `generate`

```gdscript
var generated: bool = player.generate("level-001")
var race_generated: bool = player.generate("circuit-042", "grid")
```

Generates and validates the deterministic score for the current property values. The optional second argument starts the score directly on that section; omitting it starts at the recipe's default — `garage` for Racing, `intro` (Handshake) for Suspense, `camp` for Adventure. When a score is already playing, the replacement waits for the next bar and hands off over the authored crossfade instead of restarting the output. Pass the intended opening section with the seed so the handoff does not need a second `cue_section` transition.

Returns `true` on success. Returns `false` and emits a descriptive Godot error for an empty project namespace, unsupported style/arrangement/voice, or invalid generated score. Non-finite traits are not universally rejected at generate time — pass finite 0..1 values. Do not request state changes after a failed generation.

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

Returns `false` with a Godot error when no score has been generated or when intensity/pressure are non-finite or outside `0.0..1.0`. Otherwise it returns `true` after accepting the request. The musical change commits on a bar boundary. Does not guard recipe — use only after generating a Racing score (the bool return is not a recipe check).

Selection priority:

1. `phase = "finish"` and `finish_result = "win"` selects `victory`.
2. `final_lap = true` selects `final-lap`.
3. `pressure >= 0.68` or `intensity >= 0.72` selects `attack`.
4. `phase = "race"`, `"grid"`, or `"garage"` selects `cruise`, `grid`, or `garage`.
5. Any remaining `phase = "finish"` request selects `victory`.
6. Unknown phases fall back to the score's default section.

## Racing Sections

`original` contains six 4-bar sections; `extended` keeps them byte-for-byte and adds four more harvested from the same piece and palette:

| Id | Label | Bars | Arrangement |
|---|---|---|---|
| `garage` | Garage | 4 | original + extended |
| `grid` | Starting Grid | 4 | original + extended |
| `cruise` | Race Flow | 4 | original + extended |
| `attack` | Position Fight | 4 | original + extended |
| `final-lap` | Final Lap | 4 | original + extended |
| `victory` | Finish | 4 | original + extended |
| `ignition` | Ignition | 8 | extended |
| `slipstream` | Slipstream | 16 | extended |
| `redline` | Redline | 16 | extended |
| `cooldown` | Cooldown | 8 | extended |

With `autoplay = true`, the attached song form tours the arrangement in order — `original` tours garage, grid, cruise (×2), attack, final-lap, victory and loops from grid; `extended` tours all ten sections once each and loops from grid. The four new sections are cueable and appear in the tour, but they add no new state-driven rules: `set_race_state` still selects only the six original sections. Extended carries its own `-extended-v2` score id and ` — Extended` title suffix.

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

Returns `false` with a Godot error when no score has been generated or when heat, focus, or progress are non-finite or outside `0.0..1.0`. Otherwise it returns `true` after accepting the request. Does not guard recipe — use only after generating a Suspense score.

`phase` is a free string; the documented values are `boot`, `scan`, `exploit`, `alert`, `extract`, and `complete`. Unknown phases still match the numeric thresholds. `progress` is the run's overall completion (`0.0..1.0`), `heat` is danger/action intensity, and `focus` is how locked-in the player is.

Native selection priority:

| Condition | Section | Behavior |
|---|---|---|
| `phase = "complete"` or `progress >= 0.95` | `coda` (Closed Session) | Holds |
| `phase = "extract"` | `outro` (Disconnect) | Holds |
| `heat >= 0.75` | `bridge` (Complication) | One-shot cue |
| `phase = "alert"` | `bridge` (Complication) | One-shot cue |
| `phase = "exploit"` and `focus >= 0.7` | `chorus` (Breach) | One-shot cue |

- If no rule matches, the current section continues unchanged.
- One-shot cues re-arm once the form leaves the cued section, so a later `alert` can fire again.
- Holds stay until another state changes them.
- `all-phases` is the canonical tour of the pool; `seeded` composes the form from the seed. There is no per-preset rule set any more.
- Changes commit on a bar boundary.

## Form Controls

`set_form_hold`, `advance_form`, and `is_form_held` apply to any score with a
song form — Suspense (always), and Racing/Adventure generated with
`autoplay = true`. `cue_section` works on any generated score, with or without a
form. These are separate operations to call from separate game events, not a
sequence to run together. The quickstart contains complete, guarded callbacks.

```gdscript
player.set_form_hold(true)     # freeze the song form at the current step
player.advance_form()          # move to the next form step
player.cue_section("chorus")   # jump to a specific section on the next bar
var section: String = player.get_current_section()
```

- `cue_section(section: String) -> bool` — requests any section id in the score; works with or without a form. Returns `false` with a Godot error for an unknown section or before a successful `generate`.
- `set_form_hold(held: bool) -> bool` — freezes or resumes the automatic song form. Returns `false` when the score has no form.
- `advance_form() -> bool` — moves to the next form step; returns `false` at the end of the form or when there is no form.
- `is_form_held() -> bool` — whether the form is currently frozen.
- `get_current_section() -> String` — the section currently playing, or an empty string before generation.

## Suspense Sections

The Suspense pool has 27 phases. `all-phases` plays every one once in this
canonical order; `seeded` picks a subset and loop point from the seed.

| Id | Label |
|---|---|
| `intro` | Handshake |
| `verse` | Scan |
| `half-time` | Half-Time |
| `scan-ii` | Scan II |
| `sparse` | Sparse |
| `sub-groove` | Sub-Groove |
| `pre-chorus` | Approach |
| `chorus` | Breach |
| `breach-ii` | Breach II |
| `syncopated` | Syncopated |
| `break` | Break |
| `drum-break` | Drum Break |
| `verse-b` | Second Pass |
| `drive` | Drive |
| `post-chorus` | Echo |
| `false-stop` | False Stop |
| `interlude` | Wait State |
| `filter-break` | Filter Break |
| `bridge` | Complication |
| `harmonic-bridge` | Harmonic Bridge |
| `bridge-b` | Other Hall |
| `step-up-bridge` | Step-Up Bridge |
| `solo` | Decrypt |
| `anomaly` | Anomaly |
| `chorus-final` | Full Breach |
| `outro` | Disconnect |
| `coda` | Closed Session |

Phase lengths are chosen per take (4 / half / authored / double), so the same
pool breathes differently on each seed and version. Breaks and wait states may
shrink to 1–2 bars, and momentum phases may stretch to 24/32 bars — the same
longer arc Adventure and Racing use — instead of every phase resolving at 16.

## Adventure — `set_adventure_state`

```gdscript
var accepted: bool = player.set_adventure_state("explore", 0.4, 0.2, false)
```

- `set_adventure_state(area_phase: String, discovery: float, threat: float, quest_complete: bool) -> bool` — requests the section for the current area. `discovery` and `threat` must be within `0.0..1.0`.
- A high `discovery` (`>= 0.85`) or the `sanctuary` phase resolves to `sanctuary`; a high `threat` (`>= 0.7`) resolves to `combat`, and a `combat` phase with `threat >= 0.85` escalates to `boss`; `quest_complete` always resolves to `victory`. Unknown phases fall back to `camp`.
- Like every state change, the new section commits on the next bar boundary.

## Adventure Sections

Sections are 16 or 32 bars; each develops its material across phrases rather than repeating copied halves.

| Id | Label | Bars | Development |
|---|---|---|---|
| `camp` | Trailhead Camp | 16 | hearthlight / the road ahead |
| `explore` | The Old Forest | 32 | open paths / old wonders |
| `town` | Hearth and Hall | 32 | market dance / crowded tables |
| `festival` | The Green Market | 32 | dancing feet / raised cups |
| `reunion` | Homecoming Hearth | 32 | warm embraces / old names |
| `dungeon` | The Deep Halls | 16 | cold stone / distant steps |
| `skirmish` | Steel in the Brush | 16 | blades flash / first blood |
| `combat` | Steel and Shadow | 32 | measured pursuit / battle joined |
| `chase` | Pursuit | 32 | hearts pound / ground blurs |
| `boss` | No Retreat | 16 | ancient dread / final challenge |
| `assault` | The Red Charge | 16 | full charge / no quarter |
| `sanctuary` | The Hidden Glade | 16 | clear water / shelter found |
| `dawn` | First Light | 16 | soft gold / the long night breaks |
| `victory` | Lanterns at Dawn | 32 | homecoming / earned release |

The six added phases (a combat set — skirmish, assault, chase — and a happiness
set — festival, reunion, dawn) join the pool the seeded composer draws from:
combat phases share the Peak role with combat/boss, the happiness phases ride the
groove and break bands with town/sanctuary, so the seeded arrangement interleaves
them with their matching scenes. State-driven selection is unchanged and still
returns only the eight original sections above.

## Observable Contract

- Generation is deterministic for a specific generator version and input tuple.
- Racing scores contain `garage`, `grid`, `cruise`, `attack`, `final-lap`, and `victory`; `extended` adds `ignition`, `slipstream`, `redline`, and `cooldown`.
- Suspense scores contain the 27 pool phases above; `all-phases` and `seeded` are the only arrangements.
- Adventure scores contain the fourteen sections above and default to `camp`; `camp`, `dungeon`, `boss`, `sanctuary`, `skirmish`, `assault`, and `dawn` are 16 bars, and `explore`, `town`, `festival`, `reunion`, `combat`, `chase`, and `victory` are 32.
- State changes are quantized to bar boundaries (bar-aligned crossfades) and new sections start at phrase bar zero. (A bar may occur inside a phrase; this is not a mid-phrase hard cut.)
- Audio is synthesized at 48000 Hz mono and pushed as identical left/right frames to an internal `AudioStreamPlayer`. It uses the `Music` bus when present and otherwise falls back to `Master`.

Internal Rust types, child-node names, score serialization, and exact bytes across different generator versions are not supported public API.
