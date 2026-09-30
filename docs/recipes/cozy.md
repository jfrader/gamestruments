# Cozy recipe

## What this recipe is for

Cozy is the life-sim recipe: a day in the village for farming, café, crafting
and other slow-living games. Eight sections follow the clock and the place,
with rain as its own weather. A game drives it with
`set_cozy_state(hour, place, rain)`. One seeded key and theme carry through
the day. Three styles change the ensemble: `acoustic`, `lofi`, `bossa`.

## Phases

The pool in the order of a day (`crates/engine/src/cozy/composition.rs`,
`SECTION_PLANS`).

| id | label | role | length | what it carries |
|---|---|---|---|---|
| `dawn` | First Light Coffee | Intro | 16 bars | energy 20; no drums, a held pad, the theme in pieces |
| `morning` | Morning Chores | Groove | 32 bars | energy 50; bright pop loops, light drums |
| `market` | Market Day | Groove | 32 bars | energy 64; secondary dominants, full groove |
| `noon` | Sunny Fields | Groove | 32 bars | energy 56; the working-day loop |
| `rain` | Rain on the Roof | Break | 16 bars | energy 30; the relative minor, brushes |
| `evening` | Golden Hour | Groove | 32 bars | energy 44; plagal and borrowed-iv colours |
| `festival` | Harvest Festival | Peak | 32 bars | energy 80; everything in, bells |
| `night` | Lanterns Out | Outro | 16 bars | energy 16; two bars per chord, no drums |

## Arrangements

`crates/engine/src/cozy/arrangement.rs`, `CozyArrangement`:

- `original` — native default: the eight sections, no form. State-driven. With
  `autoplay = true` it tours dawn → morning → noon → market → evening → night
  and loops back to the morning.
- `all-phases` — the same sections, toured in the order above, looping from
  `morning`.
- `seeded` — the seeded composer picks a 6–9-step day that opens at dawn, ends
  at night, and loops from one of the working-day grooves. Each section's layers
  enter and leave across its blocks.

## Traits

The four generation traits read as `bustle` (energy), `jazz` (complexity),
`warmth` (brightness) and `swing` (syncopation). Each is a continuous
magnitude; every trait-scaled choice uses its own keyed draw, so raising a knob
only adds its effect.

- `warmth` — melody register, a softer and fuller comp and pad, and bells on
  phrase openings.
- `bustle` — tempo (`+ bustle*14`, base 92 acoustic / 74 lo-fi / 116 bossa),
  drum ghost notes, comp and bass level.
- `jazz` — ninths on the chords, dominants turned into ii–V pairs, and bass
  lines that walk into the next root from a semitone below.
- `swing` — how far behind the beat each off-beat lands (lo-fi leans furthest,
  bossa stays nearly straight) and how often the melody anticipates a downbeat.

## Musical design

- **Harmony** is jazz-pop seventh chords in a major key: diatonic loops in the
  morning and fields, secondary dominants at the market and festival, the
  relative minor in the rain, and IV–iv–I and backdoor bVII7 colours for the
  evening and night. The comp voices chords rootless and leads each voice by
  the smallest step. The melody rests on chord tones that do not rub a
  neighbouring chord tone (never the root under a major seventh).
- **Melody**: one relaxed, syncopated two-bar theme per seed. Each phrase states
  it (lifted a third in the contrasting phrase). The second half either
  continues to a held arrival or is left to an answering instrument.
- **Ensembles**:
  - `acoustic` — a strummed guitar (fingerpicked in the quiet scenes), a
    glockenspiel-like lead, felt piano pads, frame drum and shaker.
  - `lofi` — electric piano chords, a felt piano lead with echo, a round sub
    bass, and a soft swung kit.
  - `bossa` — syncopated nylon-guitar comping, a flute lead, the root-and-fifth
    bossa bass, and hi-hat with a rim-click clave.

## Runtime API

```gdscript
player.project_secret = "my-game"
player.recipe = "cozy"
player.style = "lofi"              # acoustic | lofi | bossa
var ok: bool = player.generate("farm-day-001")
```

Drive it with:

```gdscript
player.set_cozy_state(hour, place, rain)
```

Selection (`select_cozy_section` in `crates/engine/src/cozy.rs`; ranges include
both ends and the earlier row wins a shared boundary):

| condition | section |
|---|---|
| `place == "festival"` | `festival` |
| `rain >= 0.5` | `rain` |
| `hour >= 21` or `hour <= 5` | `night` |
| `5 <= hour <= 8` | `dawn` |
| `17 <= hour <= 21` | `evening` |
| `place == "town"` | `market` |
| `8 <= hour <= 11` | `morning` |
| `11 <= hour <= 17` | `noon` |

`hour` must be within 0.0..24.0 and `rain` within 0.0..1.0. `place` is a free
string; only `"town"` and `"festival"` change the selection.

## Example implementation (generic)

```gdscript
func _on_clock_tick(hour: float) -> void:
    music.set_cozy_state(hour, _current_place(), weather.rain_strength)

func _on_entered_town() -> void:
    music.set_cozy_state(clock.hour, "town", weather.rain_strength)

func _on_festival_started() -> void:
    music.set_cozy_state(clock.hour, "festival", 0.0)
```

Every change commits on the next bar boundary, so calling it on each clock
tick is safe: it only moves when the section would change.

## Pitfalls

- Treating `rain` as on/off with small values: selection flips at `0.5`; send
  the weather's strength and let a drizzle stay in the day's section.
- Expecting the market at night: the clock wins after 17:00, so a town at
  evening plays the golden hour and at night plays night.
- Driving `hour` backwards across midnight: night covers 21:00–5:00 on both
  sides, so wrapping the clock is seamless.
