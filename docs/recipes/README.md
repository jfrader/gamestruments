# Recipe guides

How to use each gamestruments recipe in a game, and how to combine them.

Read the recipe you are integrating first, then [combining-recipes.md](combining-recipes.md)
for everything that spans more than one recipe (theme palettes, swapping
recipes mid-game, live variety, the copyable patterns and the exact limits).

| Recipe | Guide | Fits |
|---|---|---|
| Racing | [racing.md](racing.md) | lap-based racing, arcade driving, timed runs |
| Suspense | [suspense.md](suspense.md) | stealth, infiltration, pursuit, tense exploration |
| Adventure | [adventure.md](adventure.md) | quests, overworld/hub, dungeons, combat, boss |

Engine mechanics and the ABI live in [../engine-boundary.md](../engine-boundary.md);
the buyer-facing API reference is [../../kit/docs/api.md](../../kit/docs/api.md).

## The shape of every recipe

- A **score** is generated from `(recipe, style, seed, secret, traits)` and then
  played; the host never builds music itself.
- Two arrangements: **`all-phases`** (the complete song, every phase once) and
  **`seeded`** (the composer orders a form from the pool). Default is `seeded`.
- **Traits** are continuous knobs (0..1) that reshape the material; the extremes
  are meant to be used.
- **States** drive the selection (`set_race_state` / `set_trace_state` /
  `set_adventure_state`); **cues** (`cue_section`) and the form controls
  (`set_form_hold`, `advance_form`) override the automatic flow when the game
  needs to be literal.
- Generation is deterministic: the same inputs always give the same bytes.
