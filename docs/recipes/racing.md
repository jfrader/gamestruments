# Racing recipe

## What this recipe is for

Racing is the state-driven loop recipe: a game sends race state (`phase`,
`intensity`, `pressure`, `final_lap`, `finish_result`) and the engine picks one
section to crossfade to on the next bar boundary. It fits a circuit racer or any
short-loop action shape where the music tracks one live variable — position in
the pack, a fight for place, the final lap — and lands on a win or a loss.
Sections are four-bar loops (extended adds 8/16-bar stretches), all from one
seeded piece and palette.

## Phases

The composition pool in natural race order (`crates/engine/src/racing_pool.rs`,
`RACING_SECTION_IDS`). Roles and energy come from the authored `PLANS` table in
`crates/engine/src/racing.rs`; the remaining phases have metadata in the pool
and material in `crates/engine/src/racing_arrangement.rs`.

| id | label | role | base length | what it carries |
|---|---|---|---|---|
| `garage` | Garage | Intro | 4 bars | one-shot; opens a form exactly once |
| `ignition` | Ignition | Build | 8 bars | extended phase; authored 8 bars, honours them |
| `grid` | Starting Grid | Build | 4 bars | energy 70; focus/anticipation |
| `breather` | Breather | Breather | 4 bars | **drumless** (no kit lane), composed-only; may repeat |
| `cruise` | Race Flow | Groove | 4 bars | energy 82; the loop-point target |
| `switchback` | Switchback | Groove | 8 bars | energy 79; tuned-percussion question and answer |
| `slipstream` | Slipstream | Groove | 16 bars | extended phase; authored 16 bars |
| `attack` | Position Fight | Peak | 4 bars | energy 95 |
| `redline` | Redline | Peak | 16 bars | extended phase; authored 16 bars |
| `open-road` | Open Road | Groove | 16 bars | energy 68; spaced melodic phrases |
| `final-lap` | Final Lap | Peak | 4 bars | energy 100 (saturates the scale) |
| `victory` | Finish | Outro | 4 bars | one-shot; the win outro |
| `cooldown` | Cooldown | PostOutro | 8 bars | deliberate post-win release; thins to a pause, then a stable landing |

Game-signal sections carried by every composed score for cueing, but never part
of the form (`RACING_SIGNAL_IDS`): `defeat` (loss outro), `recovery` (reset
incident), `wrong-way`. These are absent from the `original` and `extended`
arrangements.

## Arrangements

`arrangement` accepts four values (`crates/engine/src/racing_arrangement.rs`,
`RacingArrangement`):

- `original` — native default. Six sections, byte-identical base generator
  output, no form. `set_race_state` selects only the six base sections.
- `extended` — the same six sections plus `ignition`/`slipstream`/`redline`/
  `cooldown`, ten-section tour. No form unless `autoplay = true`.
- `all-phases` — every pool phase once in canonical order, looping from the
  first groove (`cruise`).
- `seeded` — the seeded composer picks a form over the thirteen-phase pool: 6–12
  steps, ends on `victory` or `cooldown`, loops from a groove. This is where the
  `breather`, `switchback`, `open-road` and the game-signal sections appear. The composer is deterministic
  per seed (`racing_compose` in `racing_pool.rs`).

With `autoplay = true`, `original`/`extended`/`all-phases`/`seeded` attach a
song form and tour the arrangement automatically; the native default is
`autoplay = false` (state-driven). The Audio Lab defaults to `seeded` with
autoplay.

## Traits

Racing reads `energy`, `complexity`, `brightness`, `syncopation` (defaults
0.62 / 0.60 / 0.52 / 0.70). The base generator (`racing.rs`) maps them:

- `energy` — tempo: `112 + energy*36`, clamped 112–150 BPM. On the seeded path
  it also scales every lane's velocity (`0.75 + energy*0.5`) and adds extra hats
  on empty sixteenths (`energy*4` count). 0 is a slow sparse bed; 1 is a fast,
  loud, hi-hat-dense field.
- `complexity` — chord size (4 notes at `>= 0.62`, else 3), melody onset count
  (`3 + complexity*4`), chord gate (`0.68 + complexity*0.22`), passing-tone
  ornaments (`>= 0.5`). On the seeded path, `> 0.5` adds passing melody onsets
  (`(complexity-0.5)*8`). 0 is triads and a sparse lead; 1 is four-note chords,
  a busy melody, and ornaments.
- `brightness` — mode: `< 0.34` minor/dorian, `> 0.66` mixolydian/lydian, else
  dorian/mixolydian. Also swaps the melody timbre in fusion (`> 0.72`) and neon
  (`< 0.35`). 0 reads dark/minor; 1 reads bright/major. Not re-driven by the
  seeded surface — it is inherited from the base material.
- `syncopation` — off-beat share of melody onsets (`note_count * syncopation`),
  melody gate (`0.42 + (1-syncopation)*0.3`). On the seeded path it displaces
  `syncopation*4` even-eighth onsets a sixteenth late. 0 is on-grid; 1 is
  heavily off-grid.

## Runtime API

`GamestrumentsPlayer` (see `kit/docs/api.md` for exact selection rules):

```gdscript
player.project_secret = "my-game"   # non-empty stable per-title namespace
player.recipe = "racing"
player.style = "neon"               # fusion | neon | funk | chip
player.arrangement = "seeded"       # original | extended | all-phases | seeded
player.autoplay = false             # true attaches a tour form
var ok: bool = player.generate("level-001")
```

Drive it with:

```gdscript
player.set_race_state(phase, intensity, pressure, final_lap, finish_result = "none")
```

Selection order (`select_section` in `crates/engine/src/transport.rs`):

1. `phase == "finish"` + `finish_result == "win"` → `victory`.
2. `phase == "finish"` + `finish_result` in `loss`/`dnf` → `defeat` **if the
   score carries it** (seeded/all-phases); otherwise falls back to `victory`.
3. `phase == "recovery"` → `recovery`; `phase == "wrong-way"` → `wrong-way`
   (only when the score carries them).
4. `final_lap == true` → `final-lap`.
5. `pressure >= 0.68` or `intensity >= 0.72` → `attack`.
6. `phase` in `race`/`grid`/`garage` → `cruise`/`grid`/`garage`.
7. anything else → the score's default section (`garage`).

`intensity` and `pressure` must be finite and within 0.0..1.0 or the call
returns `false`. Form controls (`cue_section`, `set_form_hold`, `advance_form`,
`is_form_held`, `get_current_section`) work on any Racing score generated with
`autoplay = true`; `cue_section` also works on the signal sections of a seeded
score.

## How to use it well

- Send `set_race_state` only when the game state actually changes, not every
  frame; the engine accepts the request and commits on the next bar boundary.
- Map gameplay to the state fields, not to raw section names: drive `phase` with
  `garage`/`grid`/`race`, `pressure` with how close the fight is, `intensity`
  with overall action, `final_lap` with the last-lap flag, and `finish_result`
  with the outcome. The thresholds (`0.68` pressure, `0.72` intensity) are the
  built-in attack gate — cross them deliberately.
- For a loss or a reset, use `finish_result = "loss"`/`"dnf"` and `phase =
  "recovery"`/`"wrong-way"` on a seeded/all-phases score so the incident
  sections exist. On `original`/`extended` they do not exist and a loss falls
  back to `victory`.
- The `breather` is drumless: if your game has a genuinely calm moment (a pause,
  a pit stop), cue it explicitly — but it is only in the `seeded`/`all-phases`
  pool, not `original`/`extended`.
- Let the form flow when `autoplay = true`; don't cue every section yourself.
  Cue only for the incidents and the finish.

## Variety and dynamism

- Seeds: the compose seed is `racing_compose_seed(secret, seed)` — a hash of the
  project secret and level seed, so every level is a different form with a
  different step count, order and loop point, all rule-legal.
- The composer is deterministic and high-variety: across 250 seeds it produces
  >20 distinct forms, and every pool phase is reachable.
- Per-seed surface: the seeded path adds a development arc, seam gestures, and a
  register ceiling; `energy`/`complexity`/`syncopation` bias it continuously.
- To keep one game alive over a long session: change the level seed per circuit
  or per difficulty tier; sweep `energy`/`syncopation` for a different feel
  without a new seed; switch `style` per biome or zone. Do not re-generate per
  lap — generate once at level load.

## Example implementation (generic)

A micro circuit racer. `music` is a `GamestrumentsPlayer` generated once with
`recipe = "racing"`, `arrangement = "seeded"`.

```gdscript
# Countdown
music.set_race_state("grid", 0.0, 0.0, false)

# Laps: state per frame, committed on bar boundaries
func _physics_process(delta):
    var intensity := clamp(speed / max_speed, 0.0, 1.0)
    var pressure := clamp(1.0 - gap_to_car_ahead / lead_distance, 0.0, 1.0)
    var lap := lap_number >= total_laps
    music.set_race_state("race", intensity, pressure, lap)

# Position fight: pressure crosses 0.68 on its own -> attack

# Wrong-way / reset
music.set_race_state("wrong-way", 0.0, 0.0, false)   # on a wrong-way flag
music.set_race_state("recovery", 0.0, 0.0, false)    # after a reset to track

# Finish
func finish(placement):
    if placement == 1:
        music.set_race_state("finish", 0.0, 0.0, false, "win")
    else:
        music.set_race_state("finish", 0.0, 0.0, false, "loss")
```

The section switches happen on bar boundaries; don't gate gameplay on the music
completing the switch.

## Pitfalls

- Cueing every section yourself instead of sending state: the state fields exist
  so the engine chooses; calling `cue_section` for `cruise`/`attack`/`final-lap`
  fights the selection rules.
- Expecting `defeat`/`recovery`/`wrong-way` to exist on `original`/`extended`:
  they only exist on `seeded`/`all-phases`.
- Using traits as on/off: the knobs are continuous magnitudes. Set `energy = 1`
  only when you want the fastest, densest field.
- Treating `set_race_state` as frame-accurate: it accepts the request but the
  change lands on the next bar boundary (a bar may sit inside a phrase).
- Expecting the `breather` to have drums: it is drumless by construction and
  never gains a kit, even at high `energy`.
