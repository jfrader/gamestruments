# Adventure recipe

## What this recipe is for

Adventure is the long-form fantasy-quest recipe: eight state-selectable sections
— camp, explore, town, dungeon, combat, boss, sanctuary, victory — plus six
composer-only phases (a combat set and a happiness set) that fill out a seeded
song form. A game drives it with `set_adventure_state(area_phase, discovery,
threat, quest_complete)`; sections are long (16 or 32 bars) and develop their
material phrase by phrase rather than repeating copied halves. Three styles —
`folk`, `dark`, `orchestral` — change the ensemble and the modes.

## Phases

The pool in natural quest order (`crates/engine/src/adventure/pool.rs`,
`ADVENTURE_SECTION_IDS`; plans in `adventure/composition.rs`, `SECTION_PLANS`).
Roles and energy are derived from each section's `Scene`.

| id | label | role | base length | what it carries |
|---|---|---|---|---|
| `camp` | Trailhead Camp | Intro | 16 bars | one-shot; energy 25; opens the form |
| `explore` | The Old Forest | Groove | 32 bars | energy 48; the loop-point target |
| `town` | Hearth and Hall | Groove | 32 bars | energy 58 |
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

The score always carries all fourteen sections. `set_adventure_state` selects
only the eight original scenes (camp/explore/town/dungeon/combat/boss/sanctuary/
victory); the six added phases (skirmish/assault/chase/festival/reunion/dawn)
are pool members for the seeded composer and are reachable via `cue_section` and
the `all-phases`/`seeded` tours.

## Arrangements

`arrangement` accepts three values (`crates/engine/src/adventure/arrangement.rs`,
`AdventureArrangement`):

- `original` — native default. The base generator output: fourteen sections, no
  form. State-driven only.
- `all-phases` — the same fourteen sections byte-identical, plus a form that
  tours them in canonical order, looping from the first groove (`explore`).
- `seeded` — the seeded composer picks a form over the fourteen-phase pool:
  8–12 steps, always ending on the outro (`victory`), looping from a groove.
  `"composed"` is an alias. The seeded path also re-times sections by role
  (builds/grooves/peaks may stretch one extra 16-bar movement), applies a
  development arc, seam gestures with a shared tonic pitch class, and a
  register ceiling.

With `autoplay = true`, `original`/`all-phases`/`seeded` attach a tour form; the
native default is `autoplay = false`. The Audio Lab defaults to `seeded`.

## Musical design

Each seed composes one two-bar **quest theme** (`adventure/theme.rs`), chosen
from many candidate contours for how well it sings: mostly steps, one climax,
leaps recovered by step. Every section states it in its own way, over the
scene's mode, so the whole quest is audibly one piece:

| scenes | treatment |
|---|---|
| camp, explore, town, festival, reunion, victory | the theme, answered by a continuation to a cadence (Folk and Dark camps open with it on the harp) |
| sanctuary, dawn (and Orchestral camp and reunion) | augmented: the theme's head at half speed |
| dungeon | fragmented: the head alone, echoed a step lower across silence |
| combat, skirmish, chase, assault, boss | diminished: the whole theme in one bar, then sequenced |

Phrases are four bars (antecedent, consequent, development, return, cadence).
Open phrases arrive on the mode's dominant, closed ones on the tonic, always on
the last bar's downbeat, with a pickup that steps into the next phrase.

Harmony (`adventure/harmony.rs`) stays diatonic to the scene's mode. Colour
comes from the mode's own signature chord (Lydian II, Mixolydian bVII, Dorian
IV, Aeolian bVI, Phrygian bII), from suspended and open-fifth chords, and from
4–3 suspensions into Dark and Orchestral cadences. Calm scenes change chord
every two bars, walking scenes every bar, and battles rock between the tonic
and the signature chord. Strong beats of the melody are chord tones; a note
that would rub a semitone against the chord only passes through briefly.

Folk town and reunion lilt; Folk festival and victory, and Orchestral festival,
dance as jigs. Around the melody: a countermelody in thirds and sixths after the
first statement (recorders in Folk and Orchestral, a vielle in Dark), rolled harp
chords (Folk) or bowed strings (Dark, Orchestral), a bass that walks into
distant chords, harp figuration, Dark's open-fifth drone, a string gallop under
Orchestral battles, horns that double the theme when it returns and carry it in
the boss fights, and timpani on the tonic and dominant. The harp lets chord
tones ring until the harmony changes and damps its lines at the next note.

## Traits

Adventure exposes the four generation traits as `danger` (energy), `mystery`
(complexity), `wonder` (brightness) and `motion` (syncopation). The generator
composes them into the music (`adventure/composition.rs`, `harmony.rs`,
`theme.rs`); each is a continuous magnitude, and every trait-scaled choice uses
its own keyed draw, so raising a knob only ever adds its effect:

- `wonder` (brightness) — harmony and harp velocity, melody register (up to two
  semitones), how often the countermelody joins, and bells on phrase arrivals.
  0 is quiet, low and bare; 1 is bright, high and in two voices.
- `danger` (energy) — which colour chords a phrase chooses (darker as it rises),
  bass and percussion velocity, walking basses that break into the driving
  figure, ghost strokes in the percussion, and Orchestral timpani outside the
  fights. 0 is gentle; 1 is driving and percussive.
- `mystery` (complexity) — suspended and open-fifth chords, silences in the
  phrase continuations (never in the theme), pedal tones, stranger bells, and a
  slower tempo (`- mystery*5`). 0 is plain and full; 1 is clouded and sparse.
- `motion` (syncopation) — tempo (`+ motion*20`, base 92 folk / 78 dark / 100
  orchestral, clamped 70–126), busier continuation rhythms, folk ornaments,
  percussion ghost strokes, the Orchestral gallop (below 0.3 it relaxes to an
  eighth-note pulse), and folk explore drums (from 0.45). 0 is slow and sparse;
  1 is fast and busy.

The seeded path widens the tempo further: `base + motion_dev*16 + danger_dev*12
- mystery_dev*12`, clamped 48–130.

## Runtime API

`GamestrumentsPlayer` (see `kit/docs/api.md`):

```gdscript
player.project_secret = "my-game"
player.recipe = "adventure"
player.style = "folk"              # folk | dark | orchestral
player.arrangement = "seeded"      # original | all-phases | seeded
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
- The six composer-only phases are not selectable by state: reach them with
  `cue_section` or let the seeded/all-phases tour place them. Don't pass
  `"skirmish"`/`"festival"`/etc. as `area_phase` and expect them to be selected.

## Variety and dynamism

- Seeds: the compose seed is `adventure_compose_seed(secret, seed)`, a hash of
  the project secret and level seed. Every quest is a different 8–12-step form,
  all rule-legal and ending on victory.
- The composer is deterministic and high-variety: across 250 seeds it produces
  >20 distinct forms, and across 2000 seeds every pool phase (including the six
  added ones) is placed at least once.
- Styles change the whole ensemble: folk (recorder/vielle leads over harp,
  lilts and jigs), dark (bowed vielle over an open-fifth drone, phrygian/aeolian
  in the dangerous scenes), orchestral (bowed strings lead, with horns,
  timpani and a battle gallop). Switch style per region or campaign for a
  different sound without a new seed.
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

- Passing a composer-only phase (`skirmish`, `assault`, `chase`, `festival`,
  `reunion`, `dawn`) as `area_phase`: state selection never returns them; use
  `cue_section` or the seeded tour.
- Treating `quest_complete` as a per-area flag: it always resolves to `victory`.
- Using traits as on/off: the knobs are continuous; `danger = 1` darkens the
  harmony, drives every walking bass and fills the percussion with ghost
  strokes.
- Expecting `set_adventure_state` to change instantly: the switch lands on the
  next bar boundary (sections are 16/32 bars, so it can feel slower than Racing).
- Cueing every section yourself on a seeded score: the composer's form is the
  arc; let it flow and reserve `cue_section` for the rare explicit beats.
