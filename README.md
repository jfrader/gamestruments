# Gamestruments Audio Lab

Gamestruments Audio Lab is a standalone playground for deterministic procedural
and adaptive music in games. A level seed and generation parameters create a
stable musical identity on the authoring side; games receive portable musical
data and a small independent runtime that plans beat-aware transitions.

The prototype focuses on one question: can game parameters move between
melodies and feelings without sounding like unrelated tracks were abruptly
swapped?

## Development

Requires Node.js 24.

```bash
npm install
npm run dev
```

Open the local URL printed by Vite, choose a level seed, sound world, and
generation traits, then start audio and change race phase, speed intensity,
position pressure, and final-lap state. Generation creates the level identity;
runtime changes are committed on bar boundaries and overlap through a musical
crossover.

The Audition controls isolate melody or backing, jump directly to any section,
and compare two level seeds while preserving the section being reviewed.

The app has URL-addressable views:

- `#lab` runs the active adaptive score;
- `#games` organizes experiments by game interaction model;
- `#genres` compares musical interpretations and opens them in the lab.

```bash
npm run check
```

## Architecture

`packages/studio` uses Strudel to author and export patterns. It is kept outside
the game runtime and is AGPL-3.0-or-later. Its Pocket Circuit and Lantern Trail
generators derive independent harmony, motif, rhythm, timbre, arrangement, and
ornament sub-seeds, then develop one shared musical identity across six adaptive
sections.

`packages/runtime` consumes only portable score data and produces transition
and gain-envelope instructions. It is independently written, has no Strudel
dependency, and is MIT licensed.

`apps/demo` proves the full authoring-to-runtime idea in a browser. Its Web
Audio renderer includes synthesized lead, electric-piano, bass, drum, room,
compression, and transition stages, but remains demonstration code rather than
a production synthesizer.

The first collection follows Pocket Circuit's audio brief: compact energetic
racing loops, a lower-intensity garage state, and a distinct final-lap lift.
It currently spans electronic fusion, synthwave, pocket funk, and chiptune.
Every level seed is reproducible for a fixed generator version and supports
energy, complexity, brightness, and syncopation parameters. See
[`docs/pocket-circuit-music-brief.md`](docs/pocket-circuit-music-brief.md).

Lantern Trail proves the recipe boundary with a separate adventure identity,
trait vocabulary, section arc, and adaptive rules. It moves through camp,
exploration, clues, danger, sanctuary, and quest completion using wonder,
danger, mystery, and motion parameters.

## Procedural API

```ts
import { generatePocketCircuitLevel } from "@gamestruments/studio";

const level = generatePocketCircuitLevel({
  seed: "level-42",
  style: "funk",
  traits: {
    energy: 0.7,
    complexity: 0.6,
    brightness: 0.5,
    syncopation: 0.85,
  },
});

// Engine-neutral, JSON-compatible data with no generator or Strudel dependency.
const score = level.portableScore;
```

Lantern Trail is available from the same package:

```ts
import { generateLanternTrailAdventure } from "@gamestruments/studio";

const adventure = generateLanternTrailAdventure({
  seed: "forest-7",
  traits: { wonder: 0.8, danger: 0.4, mystery: 0.7, motion: 0.55 },
});
```

Generation happens when a level loads. Runtime parameters select and crossfade
the generated garage, grid, cruise, attack, final-lap, and victory arrangements;
they do not reseed the composition. See
[`docs/procedural-generation.md`](docs/procedural-generation.md).

Exported music still depends on the licenses of any samples used to create it.
This repository currently uses synthesized demonstration voices rather than a
redistributed sample library.

## Generation CLI

Build Studio, then export a portable score and reproducibility manifest. The
manifest records the recipe, generator version, normalized inputs, named domain
seeds, score identity, and compact-JSON SHA-256 checksum.

```bash
npm run build:studio

node packages/studio/dist/cli.js \
  --recipe pocket-circuit \
  --seed level-42 \
  --style funk \
  --output pocket-circuit.score.json \
  --manifest pocket-circuit.manifest.json

node packages/studio/dist/cli.js \
  --recipe lantern-trail \
  --seed forest-7 \
  --wonder 0.8 \
  --danger 0.4 \
  --output lantern-trail.score.json \
  --manifest lantern-trail.manifest.json
```

`@gamestruments/runtime` and `@gamestruments/studio` ship ESM JavaScript and
TypeScript declarations. Manifest helpers are available from
`@gamestruments/studio/manifest` so the Node-only checksum implementation does
not enter browser-oriented Studio imports.

## Godot Example

[`examples/godot`](examples/godot) is a standalone Godot 4.7 project that loads
an exported score as JSON, evaluates its adaptive rules, schedules changes at
bar boundaries, and visualizes the configured crossover. It does not require
JavaScript, Strudel, or the Godot MCP plugin at runtime.

## Prototype Limits

- The demo is browser-only and instrumental-only.
- The synthesized audio renderer remains browser-only; the Godot example is a
  portable-score transport and mix visualization, not a native synthesizer.
- No Unity or Unreal adapter exists yet.
- No vocal, speech, lyric, AI, cloud, or telemetry feature is planned.
- The project does not claim Strudel syntax compatibility beyond the official
  APIs imported by the studio adapter.
- Public release policy, support guarantees, and commercial terms remain
  undecided.
