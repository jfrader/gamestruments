# Suspense recipe

## What this recipe is for

Suspense is the song-form recipe for long tense sessions: music moves through a
composed song arc — intro, grooves, builds, peaks, bridges, breaks, outro, coda —
instead of looping one four-bar bed. The game drives it with trace state
(`phase`, `heat`, `focus`, `progress`) plus form controls (`cue_section`,
`set_form_hold`, `advance_form`). It fits a stealth/infiltration night-job or any
slow-burn session where tension rises and falls over minutes and the player's
progress gates the payoff.

## Phases

A 38-phase pool in canonical order (`crates/engine/src/suspense_pool.rs`,
`PHASE_POOL`). Roles and energy are authored in the pool; each phase carries a
preferred rhythm figure, and the actual figure is chosen by seeded selection
among role+energy matches (`figure_for_composition`).

| id | label | role | base length | what it carries |
|---|---|---|---|---|
| `intro` | Handshake | Intro | 8 bars | one-shot; drone + one interval; no kick/hat flow |
| `verse` | Scan | Groove | 16 bars | energy 45; the loop-point target |
| `half-time` | Half-Time | Groove | 8 bars | energy 38 |
| `scan-ii` | Scan II | Groove | 16 bars | energy 45; cueable continuation of Scan |
| `sparse` | Sparse | Groove | 4 bars | energy 22; a low wait state |
| `sub-groove` | Sub-Groove | Groove | 8 bars | energy 50 |
| `pre-chorus` | Approach | Build | 8 bars | energy 60 |
| `chorus` | Breach | Peak | 8 bars | energy 80 |
| `breach-ii` | Breach II | Peak | 16 bars | energy 85; cueable escalation of Breach |
| `syncopated` | Syncopated | Groove | 8 bars | energy 55 |
| `break` | Break | Break | 4 bars | energy 20; **deliberate drop** — no kit, thins into the drop |
| `drum-break` | Drum Break | Break | 4 bars | energy 45; drums-only break |
| `verse-b` | Second Pass | Groove | 8 bars | energy 50; no pulse lane |
| `drive` | Drive | Groove | 8 bars | energy 68 |
| `post-chorus` | Echo | Groove | 8 bars | energy 55 |
| `false-stop` | False Stop | Break | 4 bars | energy 12; **deliberate stop** — everything drops out before the return |
| `interlude` | Wait State | Groove | 4 bars | energy 30; a low wait state |
| `filter-break` | Filter Break | Break | 8 bars | energy 28; filter sweep down |
| `bridge` | Complication | Bridge | 16 bars | energy 50; the alert cue |
| `harmonic-bridge` | Harmonic Bridge | Bridge | 8 bars | energy 62 |
| `bridge-b` | Other Hall | Bridge | 8 bars | energy 55 |
| `step-up-bridge` | Step-Up Bridge | Bridge | 8 bars | energy 70 |
| `solo` | Decrypt | Build | 8 bars | energy 70; no pulse lane |
| `anomaly` | Anomaly | Build | 8 bars | one-shot; energy 60 |
| `chorus-final` | Full Breach | Peak | 8 bars | energy 90 |
| `theme-ride` | Theme Ride | Groove | 8 bars | energy 68; the Theme's rock-backbeat ride |
| `build` | Build Order | Groove | 32 bars | energy 40; match phase: kick and hats, then clap, bass, stab and a fill |
| `scout` | Recon | Groove | 16 bars | energy 35; match phase: a running line over the groove, a one-bar kick gap |
| `expand` | Expansion | Groove | 32 bars | energy 55; match phase: groove, stab, arpeggio, fill |
| `research` | Tech Up | Build | 16 bars | one-shot; energy 50; match phase: arpeggio and pad, then the kick and a roll |
| `raid` | Raid | Groove | 16 bars | energy 65; match phase: running line and bass, stab and fill |
| `tension` | Standoff | Break | 16 bars | one-shot; energy 30; match phase: breakdown into a roll |
| `siege` | Siege | Bridge | 32 bars | one-shot; energy 60; match phase: pounding kick, running line, then pad |
| `battle` | Battle | Peak | 32 bars | one-shot; energy 90; match phase: the full drop, a kick gap, fills |
| `victory` | Victory | Outro | 16 bars | one-shot; energy 40; match phase: pad and arpeggio, then the kick |
| `defeat` | Defeat | Outro | 16 bars | one-shot; energy 15; match phase: pad alone |
| `outro` | Disconnect | Outro | 4 bars | one-shot; energy 20; **drumless** (no kit flow) |
| `coda` | Closed Session | Outro | 4 bars | one-shot; energy 20; **drumless** |

The ten match phases (`crates/engine/src/match_phases.rs`) follow a strategy
game's arc and are written as 8-bar block plans, so every style plays the same
shape in its own instruments.

Phases keep their authored length only as a starting point: the composed path
re-times them per take (`phase_bars` in `suspense_arrangement.rs`) — breaks and
waits may shrink to 1–2 bars, momentum phases may stretch to 24/32 bars.

## Sound worlds

A style changes only the instruments. The piece is composed once in
Terminal's instruments; `apply_sound_world` (`crates/engine/src/suspense.rs`)
then moves every part — drone, cell, pulse, arpeggio, pad — and the kick and
snare to the style's voices, so every style plays the same notes, phases and
form.

| style | drone | cell | pulse | arp | pad | kick / snare |
|---|---|---|---|---|---|---|
| `terminal` | warm | glass | pulse | pulse | dusk | kick / snare |
| `cipher` | warm | pluck | bass | glass | dusk | kick / snare |
| `noir` | organ | epiano | bass | warm | dusk | kick / snare |
| `trance` | trance-pad | trance-lead | saw-bass | trance-lead | trance-pad | techno-kick / clap |

Saw bass and the trance voices duck under every club kick.

## Arrangements

`arrangement` accepts two values (`crates/engine/src/suspense_arrangement.rs`,
`SuspenseArrangement`); the retired `original`/`extended`/`theme` names all
resolve to `seeded`:

- `all-phases` — every pool phase once in canonical order, looping from the
  first groove (`verse`).
- `seeded` — the default. The composer (`compose` in `suspense_pool.rs`) picks
  the count, roles, order and loop point from the seed, shaped by an `Intent`:
  `loop` (5–7 steps, no outro), `arc` (8–10, outro; the default), `long`
  (11–14, outro), or `surprise` (5–14, outro half the time).

The composer applies two seeded budgets on top of the role grammar: adjacent
grooves may change at most one contrast attribute (family / subdivision /
density / register), and a groove run may change figure at most once per two
phases. `autoplay` is ignored for Suspense — the form is always present.

## Traits

Suspense reads `energy`, `complexity`, `brightness`, `syncopation` as `tension`,
`heat`, `mystery`, `pulse`. The base generator (`suspense.rs`) and the composed
path (`suspense_arrangement.rs`) both map them continuously:

- `tension` (energy) — pulse/arp velocity (`0.14 + tension*0.12`); on the
  composed path, drone/pulse/seam velocity (`1.0 + tension_dev*0.5`) and the
  development-arc layer count. 0 is a quiet bed; 1 is pressurized, loud.
- `heat` (complexity) — pulse velocity (`0.22 + heat*0.08`); on the composed
  path, kit velocity, added hats/kicks on empty sixteenths, and a final-bar tom
  fill. 0 is sparse; 1 is dense with a hot fill.
- `mystery` (brightness) — cell hold (`pulse*6` when `>= 0.6`, else `pulse*3`);
  on the composed path, stretches the melodic cell and folds it down an octave
  (dark) or up (bright), and lowers the tempo. 0 is short, bright cells; 1 is
  sustained, low, drifting cells.
- `pulse` (syncopation) — tempo: `base + pulse*8` clamped 60–88 (base 72
  terminal / 78 cipher / 64 noir); on the composed path a further widen
  (`+ pulse_dev*16`, clamped 52–100). 0 is slow; 1 is the fastest the piece goes.

Defaults are 0.62 / 0.60 / 0.52 / 0.70, but the composed path treats the Lab
preset (tension 0.62, heat 0.48, mystery 0.72, pulse 0.55) as the neutral point,
so every trait deviates from there.

## Runtime API

`GamestrumentsPlayer` (see `kit/docs/api.md`):

```gdscript
player.project_secret = "my-game"
player.recipe = "suspense"
player.style = "terminal"          # terminal | cipher | noir | trance
player.arrangement = "seeded"      # all-phases | seeded (default)
var ok: bool = player.generate("level-001")
```

Drive it with trace state and form controls:

```gdscript
player.set_trace_state(phase, heat, focus, progress)
player.cue_section("chorus")
player.set_form_hold(true)
player.advance_form()
```

Selection (`select_trace_section` in `suspense.rs`, serialized into the score's
own rules):

| condition | section | behavior |
|---|---|---|
| `phase == "complete"` or `progress >= 0.95` | `coda` (Closed Session) | holds |
| `phase == "extract"` | `outro` (Disconnect) | holds |
| `heat >= 0.75` | `bridge` (Complication) | one-shot cue |
| `phase == "alert"` | `bridge` (Complication) | one-shot cue |
| `phase == "exploit"` and `focus >= 0.7` | `chorus` (Breach) | one-shot cue |

If no rule matches, the current section continues. One-shot cues re-arm once the
form leaves the cued section, so a later `alert` can fire again. `heat`, `focus`,
`progress` must be finite and within 0.0..1.0. `phase` is a free string; the
documented values are `boot`, `scan`, `exploit`, `alert`, `extract`, `complete`.

## How to use it well

- Use the holds for the deliberate states: `phase = "scan"` begins scanning,
  `phase = "complete"` and `phase = "extract"` hold the ending. Everything else
  is a one-shot cue that hands back to the form.
- Cue the four canonical beats from separate events, not one frame
  (`docs/engine-boundary.md`): hold + `cue_section("verse")` to begin scanning;
  `advance_form()` on progress into `scan-ii`; `cue_section("chorus")` on an
  exploit; `advance_form()` on escalation into `breach-ii`; `set_form_hold(false)`
  to resume the auto-tour.
- Let the form flow when nothing is happening. Only hold when the game demands a
  specific section stay present, and release the hold promptly.
- The `break`/`false-stop`/`filter-break`/`outro`/`coda` are the deliberate
  drumless or near-drumless moments: don't fight them with heat spikes that add
  kit density over the top.

## Variety and dynamism

- Seeds and takes: `take_seed(secret, seed, domain, index)` is index-addressed
  (not chained), so any take is reachable directly. The take drives the material,
  the figures and the surface — a different take is a different song, not a
  re-ordered one.
- The composer is deterministic and high-variety: across 250 seeds per intent it
  produces >200 distinct forms; the four intents change the shape (count band and
  outro preference).
- Per-seed transitions: `apply_transition_pass` picks a fill, riser, lift or tail
  plus a landing for every join, so no two joins are treated identically.
- To keep one session alive: generate one score, hold/advance through it, and
  pick a new take (seed index) per mission. The `Intent` (loop/arc/long/surprise)
  is the coarse shape knob; traits are the fine continuous one.

## Example implementation (generic)

A stealth/infiltration night-job. `music` is a `GamestrumentsPlayer` generated
once with `recipe = "suspense"`.

```gdscript
# Boot: hold the intro while the job loads
music.set_trace_state("boot", 0.0, 0.0, 0.0)

# Begin scanning: hold + cue the verse, keep it available
music.set_form_hold(true)
music.cue_section("verse")
music.set_trace_state("scan", heat, focus, progress)

# Progress: step into Scan II, still held
music.advance_form()

# Exploit: cue the breach when focused
if focus >= 0.7:
    music.cue_section("chorus")
    music.set_trace_state("exploit", heat, focus, progress)

# Alert: cue the complication (also fires when heat >= 0.75 on its own)
music.set_trace_state("alert", heat, focus, progress)

# Escalation: step into Breach II
music.advance_form()

# Release the hold and let the form resume
music.set_form_hold(false)

# Extract / complete: hold the ending
music.set_trace_state("extract", 0.0, 0.0, 1.0)
music.set_trace_state("complete", 0.0, 0.0, 1.0)
```

Drive `heat` and `focus` from your stealth loop (detection level, how locked-in
the player is) and `progress` from overall completion. The section changes land
on bar boundaries.

## Pitfalls

- Cueing every beat yourself: Suspense is a song form; cue only the four
  canonical beats and let `advance_form` and the auto-tour handle the rest.
- Using traits as on/off: the knobs are continuous magnitudes around the Lab
  preset; a trait at 0 or 1 is an extreme, not a toggle.
- Expecting a phase to be drumless when it isn't: `intro`, `break`, `outro`,
  `coda` drop the kit flow; `drum-break` is drums-only; the others carry the kit.
- Re-generating per state change: generate once; the form and takes are the
  variety, not regeneration.
- Holding forever: `set_form_hold(true)` with no release leaves the piece stuck
  on one section; the hold state never resets on its own.
