# Adventure recipe

## What this recipe is for

Adventure is the long-form fantasy-quest recipe: eight state-selectable sections
— camp, explore, town, dungeon, combat, boss, sanctuary, victory — plus eight
cueable alternatives: a combat set, a happiness set and guitar-led Explore/Town
variants. A game drives it with `set_adventure_state(area_phase, discovery,
threat, quest_complete)`; sections are long (16 or 32 bars) and develop their
material phrase by phrase rather than repeating copied halves. Three styles —
`folk`, `dark`, `orchestral` — change the ensemble and the modes.

## Phases

The pool follows the authored plans in `crates/engine/src/adventure/composition.rs`,
`SECTION_PLANS`.
Roles and energy are derived from each section's `Scene`.

| id | label | role | base length | what it carries |
|---|---|---|---|---|
| `camp` | Trailhead Camp | Intro | 16 bars | one-shot; energy 25; opens the form |
| `explore` | The Old Forest | Groove | 32 bars | energy 48; the loop-point target |
| `explore-strings` | The Winding Trail | Groove | 32 bars | energy 48; spacious guitar-led Explore alternative |
| `town` | Hearth and Hall | Groove | 32 bars | energy 58 |
| `town-strings` | Courtyard Dance | Groove | 32 bars | energy 58; guitar-and-bombo Town alternative |
| `festival` | The Green Market | Groove | 32 bars | energy 62; composer-only happiness phase |
| `reunion` | Homecoming Hearth | Groove | 32 bars | energy 54; composer-only happiness phase |
| `dungeon` | The Deep Halls | Build | 16 bars | one-shot; energy 20; sparse, low |
| `skirmish` | Steel in the Brush | Peak | 16 bars | energy 76; composer-only combat phase |
| `combat` | Steel and Shadow | Peak | 32 bars | energy 82 |
| `chase` | Pursuit | Peak | 32 bars | energy 80; composer-only combat phase |
| `boss` | No Retreat | Peak | 16 bars | one-shot; energy 90 (the climax) |
| `assault` | The Red Charge | Peak | 16 bars | one-shot; energy 87; composer-only combat phase |
| `sanctuary` | The Hidden Glade | Break | 16 bars | one-shot; energy 34; the quiet respite |
| `dawn` | First Light | Break | 16 bars | one-shot; energy 36; composer-only bright release |
| `victory` | Lanterns at Dawn | Outro | 32 bars | one-shot; energy 70; the outro |

The score always carries all sixteen sections. `set_adventure_state` selects
only the eight original scenes (camp/explore/town/dungeon/combat/boss/sanctuary/
victory); the other sections are pool members for the seeded composer and are
reachable via `cue_section`. The Lab's `all-phases` tour includes every section.

The guitar variants borrow nylon phrasing and bombo skin/rim accents from
Folklore, but keep Adventure's tonic, scene-specific mode, tempo and 4/4 meter.
They do not replace the original Explore or Town music.

## Arrangements

The Rust/WASM arrangement selector accepts three values (`crates/engine/src/adventure/arrangement.rs`,
`AdventureArrangement`):

- `original` — native default. The base generator output: sixteen sections, no
  form. State-driven only.
- `all-phases` — the same sixteen sections byte-identical, plus a form that
  tours them in canonical order, looping from the first groove (`explore`).
- `seeded` — the seeded composer picks a form over the sixteen-phase pool:
  8–12 steps, always ending on the outro (`victory`), looping from a groove.
  `"composed"` is an alias. The seeded path also re-times sections by role
  (builds/grooves/peaks may stretch one extra 16-bar movement), applies a
  development arc, seam gestures with a shared tonic pitch class, a continuous
  trait response, and a register ceiling.

The Audio Lab defaults to `seeded`. Godot currently uses the base generator and
ignores Adventure's `arrangement` property. Its optional `autoplay = true` tours
the eight state-selectable sections; the default is `false`. Use `cue_section`
to play an alternative in Godot.

## Traits

Adventure exposes the four generation traits as `danger` (energy), `mystery`
(complexity), `wonder` (brightness) and `motion` (syncopation). The base
generator (`adventure/composition.rs`) and the seeded surface
(`adventure/arrangement.rs`) both map them:

- `wonder` (brightness) — harmony velocity (`0.13 + wonder*0.06`), harp velocity
  (`0.15 + wonder*0.08`), melody register (`+ wonder*2` semitones) and velocity.
  On the seeded path it brightens the harmony layer (`1.0 + wonder_dev*0.6`).
  0 is quiet, low; 1 is bright, high, loud.
- `danger` (energy) — bass velocity (`+ danger*0.04`) and percussion velocity
  (`+ danger*0.08`). On the seeded path it adds percussion density (`danger*6`),
  pushes the bass, and folds melody/harmony/harp down an octave when `> 0.66`,
  and raises the tempo. 0 is gentle; 1 is driving, dark, percussive.
- `mystery` (complexity) — tempo (`- mystery*5`). On the seeded path it adds a
  pedal drone (`mystery*4`), bell accents (`mystery*3`), thins the harmony, and
  pulls the tempo down further. 0 is fast, full; 1 is slow, drone-heavy, sparse.
- `motion` (syncopation) — tempo (`+ motion*20`, base 92 folk / 78 dark / 100
  orchestral, clamped 70–126), melody onset density (`< 0.3` removes a note,
  `> 0.78` adds one in folk), and some percussion gating (folk explore drums at
  `> 0.45`). On the seeded path it widens tempo further. 0 is slow, sparse; 1 is
  fast, busy.

The seeded tempo widen is `base + motion_dev*16 + danger_dev*12 - mystery_dev*12`,
clamped 48–130.

## Runtime API

`GamestrumentsPlayer` (see `kit/docs/api.md`):

```gdscript
player.project_secret = "my-game"
player.recipe = "adventure"
player.style = "folk"              # folk | dark | orchestral
var ok: bool = player.generate("level-001")
```

Drive it with:

```gdscript
player.set_adventure_state(area_phase, discovery, threat, quest_complete)
```

Selection (`select_adventure_section` in `crates/engine/src/adventure.rs`):

| condition | section |
|---|---|
| `quest_complete` or `area_phase == "victory"` | `victory` |
| `area_phase == "boss"`, or `area_phase == "combat"` and `threat >= 0.85` | `boss` |
| `area_phase == "combat"` or `threat >= 0.7` | `combat` |
| `discovery >= 0.85` or `area_phase == "sanctuary"` | `sanctuary` |
| `area_phase == "dungeon"` | `dungeon` |
| `area_phase == "town"` | `town` |
| `area_phase == "explore"` or `discovery >= 0.3` | `explore` |
| otherwise | `camp` |

`discovery` and `threat` must be finite and within 0.0..1.0. `area_phase` is a
free string; the documented values are the eight scene ids. `quest_complete`
always resolves to `victory`.

## How to use it well

- Send the area phase on area transitions, and let `discovery`/`threat` ride the
  player's local state: high `threat` escalates combat→boss, high `discovery`
  resolves to sanctuary, low discovery falls back to explore/camp.
- Use `quest_complete = true` for the final victory, not as a per-area flag.
- The `dungeon` and `sanctuary`/`dawn` sections are the quiet, sparse moments;
  don't push `danger` up through them or they lose their contrast.
- The eight musical alternatives are not selectable by state: reach them with
  `cue_section` or the Lab's seeded/all-phases forms. Don't pass
  `"skirmish"`/`"festival"`/etc. as `area_phase` and expect them to be selected.

## Variety and dynamism

- Seeds: the compose seed is `adventure_compose_seed(secret, seed)`, a hash of
  the project secret and level seed. Every quest is a different 8–12-step form,
  all rule-legal and ending on victory.
- The composer is deterministic and high-variety: across 250 seeds it produces
  >20 distinct forms, and across 2000 seeds every added pool phase is placed at
  least once.
- Styles change the whole ensemble: folk (recorder/vielle leads), dark (bowed
  vielle, drone-driven, phrygian/aeolian in the dangerous scenes), orchestral
  (bowed strings lead the original sections). The two guitar variants keep a
  nylon lead across styles, with the scene's mode and supporting orchestration.
  Switch style per region or campaign for a different sound without a new seed.
- To keep one game alive: new level seed per area, sweep `wonder`/`danger` for a
  different brightness/drive, and switch `style` per biome. Generate once at
  level load.

## Example implementation (generic)

An overworld quest loop. `music` is a `GamestrumentsPlayer` generated once with
`recipe = "adventure"`.

```gdscript
# Camp (opening)
music.set_adventure_state("camp", 0.0, 0.0, false)

# Explore the overworld; discovery climbs as the player finds points of interest
func on_area_enter(area):
    match area.kind:
        KIND_FIELD:
            music.set_adventure_state("explore", discovery, threat, false)
        KIND_TOWN:
            music.set_adventure_state("town", discovery, threat, false)
        KIND_DUNGEON:
            music.set_adventure_state("dungeon", discovery, threat, false)

# Combat: threat drives combat -> boss escalation
func on_combat_started():
    music.set_adventure_state("combat", discovery, threat, false)

func on_boss_appeared():
    music.set_adventure_state("combat", discovery, 0.9, false)  # threat >= 0.85 -> boss

# Sanctuary: high discovery resolves here
func on_sanctuary_found():
    music.set_adventure_state("sanctuary", 0.9, threat, false)

# Victory
func on_quest_finished():
    music.set_adventure_state("camp", discovery, threat, true)   # quest_complete -> victory
```

Every change commits on the next bar boundary. `discovery` and `threat` update
from the player's exploration and danger state; only the area transitions and the
finish need explicit calls.

## Pitfalls

- Passing a musical alternative (`skirmish`, `assault`, `chase`, `festival`,
  `reunion`, `dawn`, `explore-strings`, `town-strings`) as `area_phase`: state selection never returns them; use
  `cue_section` or the seeded tour.
- Treating `quest_complete` as a per-area flag: it always resolves to `victory`.
- Using traits as on/off: the knobs are continuous; `danger = 1` also folds the
  whole section down an octave and piles on percussion.
- Expecting `set_adventure_state` to change instantly: the switch lands on the
  next bar boundary (sections are 16/32 bars, so it can feel slower than Racing).
- Cueing every section yourself on a seeded score: the composer's form is the
  arc; let it flow and reserve `cue_section` for the rare explicit beats.
