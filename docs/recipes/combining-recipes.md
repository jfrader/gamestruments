# Combining recipes in one game

One game can hold many musical identities at once, and most of the interesting
moves come from combining the pieces rather than from any single knob. This doc
is the "how far can it go" map. Per-recipe mechanics live in
[racing.md](racing.md), [suspense.md](suspense.md), and [adventure.md](adventure.md);
the API surface in [`kit/docs/api.md`](../../kit/docs/api.md) and the engine
boundary in [`engine-boundary.md`](../engine-boundary.md). Nothing here adds new
API; every call below is real.

## 1. The building blocks

| Block | Who sets it | What it controls |
|---|---|---|
| `recipe` | host, before `generate` | which generator: `racing` / `suspense` / `adventure`. Engine-owned. |
| `arrangement` | host, before `generate` | section pool + form. Per recipe: racing `original`/`extended`/`all-phases`/`seeded`; suspense `all-phases`/`seeded`; adventure `original`/`all-phases`/`seeded`. |
| `style` | host, before `generate` | the sound world: instrument pool + lead voice (+ modes). Never affects the form. |
| `seed` | host, one string to `generate(seed)` | hashed with `project_secret` into the compose seed and the material domains. Determines form and identity. |
| version / take | engine only | the generator version is pinned by the shipped binary; Suspense "takes" are index-addressed internally from the seed. No host knob — change the seed string for a new take. |
| traits | host, before `generate` | four floats `energy`/`complexity`/`brightness`/`syncopation`, re-read per recipe (racing: energy/complexity/brightness/syncopation; suspense: tension/heat/mystery/pulse; adventure: danger/mystery/wonder/motion). Baked at generate, not live. |
| cues / holds | host, at game events | `set_race_state` / `set_trace_state` / `set_adventure_state` + `cue_section`, `set_form_hold`, `advance_form`. Transport selects and commits on bar boundaries. |
| generated score + audio | engine | the `PortableScore` and the in-process 48 kHz mono synth. The host only keeps the node in the tree. |

Engine-side: recipe, arrangement, style, seed, version/take, traits, and the
score + audio. Host-side: the four trait floats, the seed string, and every
runtime cue/hold call. The host owns *when* and *what*; the engine owns *how*.

## 2. One recipe, many moments

A single generated score already spans a whole session. Map game moments to the
recipe's own state calls rather than inventing new phases:

| Moment | Racing (`set_race_state`) | Suspense (`set_trace_state` + form) | Adventure (`set_adventure_state`) |
|---|---|---|---|
| Menu / boot | `garage` (menu), `grid` (countdown) | hold `intro` (`boot`), then cue `verse` | `camp` |
| Main gameplay | `race` (→ `cruise`), `pressure` rides the fight | `scan` + form flow (`verse`/`scan-ii`) | `explore` / `town` |
| High-stakes / boss | `attack` via `pressure >= 0.68`, `final-lap` flag | `exploit`+`focus` → `chorus`/`breach-ii`, `alert` → `bridge` | `combat`, `threat >= 0.85` → `boss` |
| Breather | `breather` (drumless, seeded/all-phases only) or `cooldown` | `break` / `false-stop` / `interlude` / `sparse` | `sanctuary` (high `discovery`) or `dungeon` |
| Results | `finish` + `win` → `victory`; `loss`/`dnf` → `defeat` (seeded/all-phases only) | `extract` → `outro` (holds), `complete` → `coda` (holds) | `quest_complete = true` → `victory` |
| Credits | let `victory` loop, or run a `seeded` tour (`autoplay`) | `outro`/`coda` hold (drumless) | let `victory` loop (32 bars) |

## 3. Combining recipes in one game

Each generated score belongs to one recipe; there is no recipe switch on a live
score. To move from an Adventure hub to a Suspense stealth level to a Racing
minigame, generate a *new* score for the new recipe and crossfade at the host
audio level:

- Generate the incoming player's score while the outgoing still plays.
- Crossfade the two players' buses (the engine's 2-bar crossfade only joins
  sections *inside one score*; it never bridges two players).
- Free the outgoing player after the fade.

What carries over: `project_secret` (the per-title namespace) and whatever
style/trait/seed conventions you author as data. Nothing musical carries across
recipes — each recipe derives its own harmony, motif, rhythm, and palette
domains, so the Racing minigame never quotes the Adventure overworld, by design.

What resets: the score, transport, synth, and current section. `generate` starts
at the recipe's first section — `garage`, `intro` (Handshake), or `camp`.

```gdscript
# Two players, generated ahead of time; swap by crossfading bus volumes.
world.generate("hub-001")       # adventure, folk, seeded
stealth.generate("vault-001")   # suspense, terminal
race.generate("circuit-001")    # racing, funk

func enter_zone(next: GamestrumentsPlayer, seed: String) -> void:
    next.generate(seed)                 # generate while the old still plays
    crossfade_buses(current, next)      # host-side volume automation
    current = next
```

## 4. A different sound world per theme

The same recipe can sound like a different world per zone: `style` changes the
sound world (instruments, lead voice, modes) while `seed` changes the form, and
traits change the feel. The compose seed depends only on `project_secret` and
`seed`, so a style swap keeps the form and changes the timbre. Author theme →
(recipe, style, seed, traits) as data:

```gdscript
const THEMES := {
    "ice":     { "recipe": "racing",    "style": "chip",   "seed": "zone-ice",     "energy": 0.45, "complexity": 0.5, "brightness": 0.85, "syncopation": 0.8 },
    "desert":  { "recipe": "racing",    "style": "funk",   "seed": "zone-desert",  "energy": 0.8,  "complexity": 0.6, "brightness": 0.5,  "syncopation": 0.75 },
    "night":   { "recipe": "suspense",  "style": "noir",   "seed": "zone-night",   "energy": 0.4,  "complexity": 0.55, "brightness": 0.35, "syncopation": 0.5 },
    "boss":    { "recipe": "adventure", "style": "dark",   "seed": "zone-boss",    "energy": 0.85, "complexity": 0.8, "brightness": 0.3,  "syncopation": 0.7 },
    "hub":     { "recipe": "adventure", "style": "folk",   "seed": "hub",          "energy": 0.5,  "complexity": 0.5, "brightness": 0.7,  "syncopation": 0.6 },
}

func apply_theme(player: GamestrumentsPlayer, key: String) -> void:
    var t := THEMES[key]
    player.recipe = t.recipe
    player.style = t.style
    player.energy = t.energy
    player.complexity = t.complexity
    player.brightness = t.brightness
    player.syncopation = t.syncopation
    player.generate(t.seed)
```

Real style names only: Racing `fusion`/`neon`/`funk`/`chip`; Suspense
`terminal`/`cipher`/`noir`; Adventure `folk`/`dark`/`orchestral`. Trait glosses
follow each recipe's documented mapping (see the recipe docs): for Racing,
`brightness` is the mode (minor/dorian ↔ mixolydian/lydian); for Adventure, `dark`
already bends dangerous scenes phrygian/aeolian, so a dark boss zone needs little
extra `danger`.

## 5. Dynamic and live ideas

- **Seeds for variety**: new seed string per level/run/circuit gives a new form
  with the same rules. `racing_compose_seed` and `adventure_compose_seed` are
  deterministic per (secret, seed), so a level is reproducible.
- **Versions/takes**: not a live knob. A "different take" is a different seed
  string (or, for Suspense, the engine's internal take index — reachable only by
  changing the seed).
- **Trait sweeps**: traits are baked at `generate`, so a sweep is generate +
  crossfade, not a continuous dial. Do it at zone changes, not mid-zone.
- **Holds and cues for scripted beats**: Suspense's canonical four beats
  (`cue_section("verse")` → `advance_form()` → `cue_section("chorus")` →
  `advance_form()`), and Racing's `wrong-way`/`recovery` for scripted resets.
- **Deliberate drumless phases**: cue `breather` (Racing), `break`/`false-stop`/
  `filter-break`/`outro`/`coda` (Suspense), `sanctuary`/`dawn` (Adventure) for a
  genuine drop; don't pile heat/danger on top of them.
- **Defeat / recovery / wrong-way**: Racing only, and only on `seeded`/
  `all-phases` — they don't exist on `original`/`extended`.
- **Per-seed transitions**: every join gets a seeded gesture (fill, riser, lift,
  or tail + a landing), so no two handoffs sound the same.
- **Seam / crossfade**: state changes commit on a bar boundary, land at phrase
  bar zero, and crossfade over 2 bars. Sections are 4/8/16/32 bars, so the
  switch lands faster in Racing than in Adventure's 16/32-bar sections.

## 6. Patterns

**World director** — one node owns a player per zone and swaps them.

```gdscript
func enter(key: String) -> void:
    apply_theme(next_player, key)
    next_player.generate(THEMES[key].seed)
    crossfade_buses(current, next_player)
    current, next_player = next_player, current
```

**Minigame swap with carryover** — keep one `project_secret` and a shared seed
prefix so the world reads as one title even though the recipe changes.

```gdscript
race.project_secret = world.project_secret
race.generate(world.zone_id + "/race")   # same title, new recipe
```

**Theme palette as data** — the `THEMES` dictionary above is the whole palette;
add a zone by adding one row, not new code.

**Tension director** — a Suspense driver mapping detection to the form controls.

```gdscript
func on_detected():
    music.set_trace_state("alert", heat, focus, progress)   # → bridge
func on_breach():
    music.cue_section("chorus"); music.set_trace_state("exploit", heat, 0.8, progress)
func on_clear():
    music.set_form_hold(false)                              # resume the auto-tour
```

**Credit suite** — hold the ending; the outro/coda are already the release.

```gdscript
music.set_trace_state("complete", 0.0, 0.0, 1.0)   # coda holds through credits
```

## 7. Limits

- **No live trait changes.** The four traits are read at `generate` and baked;
  only the state methods and form controls are live.
- **No recipe switch on a live score.** A score belongs to one recipe; switching
  means generating a new score (and crossfading host-side).
- **No cross-recipe crossfade.** The engine's crossfade (2 bars) joins sections
  within one score only. Between two players it is your bus automation.
- **No version/take knob.** `generate(seed)` takes a seed string only; the
  generator version and take index are internal.
- **No arbitrary state graph.** The three state models (`race`/`trace`/`adventure`)
  are the whole adaptive surface; there is no custom game-state authoring.
- **No sample import, MIDI/WAV export, pattern editor, or FMOD/Wwise.** Godot
  4.7.x only; no web/mobile/console.
- `defeat`/`recovery`/`wrong-way` exist only on Racing `seeded`/`all-phases`;
  Adventure's six composer-only phases (`skirmish`/`assault`/`chase`/`festival`/
  `reunion`/`dawn`) are cue/tour-only, never state-selectable.

## 8. Pitfalls

- **Regenerating per lap or per state change.** Generate once at level load; the
  state methods and the form are the variety. Regeneration reloads from the
  first section and discards the crossfade.
- **Treating traits as live.** Set them, then `generate`. A "sweep" is a
  regeneration + crossfade, not a running knob.
- **Expecting the engine to crossfade between recipes.** Two players need your
  bus-level fade; the engine only crossfades inside one score.
- **Designing around `defeat`/`recovery`/`wrong-way` on Racing `original` or
  `extended`.** They don't exist there; a loss falls back to `victory`.
- **Holding forever.** `set_form_hold(true)` with no release pins the piece to
  one section; release it on the beat that ends the scripted moment.
