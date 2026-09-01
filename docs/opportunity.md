# Opportunity

## Buyer And Job

Gamestruments targets solo and small-team game developers who want instrumental
music to react to gameplay without adopting a full audio middleware workflow.
The job is to move between melodies, energy levels, and emotional states at
musically sensible boundaries while preserving continuity.

## Gap

Static music packs are easy to ship but cannot respond deeply to game state.
FMOD, Wwise, Elias, and engine-specific systems are powerful, but their setup
and authoring models can be disproportionate for a small game. Live-coding
systems are expressive, but they are not normally packaged around a portable
game-runtime contract.

Observed indie comparables span roughly $29-$90, with free tools at one end and
large middleware licenses at the other. Relevant alternatives include Strudel,
TidalCycles, Tone.js, BarelyMusician, FMOD, Wwise, Elias, Mowjera, and multiple
engine-specific adaptive stem managers.

## Prototype Hypothesis

A Strudel-based, open authoring environment can export portable instrumental
scores to a small independent runtime. The runtime can then quantize state
changes, overlap compatible sections, and expose gain envelopes to any engine
without shipping Strudel inside the game.

The first proof must demonstrate:

- coherent transitions between at least three emotional states;
- deterministic responses to intensity and biome parameters;
- preserved beat, meter, and harmonic continuity during crossover;
- an exported format that contains no executable Strudel code;
- a runtime package with no Strudel dependency.

Gamestruments Audio Lab is a standalone testing surface that may eventually be
published. Pocket Circuit Online is its first collection and validation target.
The racing collection narrows the first proof to garage, starting-grid,
race-flow, position-pressure, final-lap, and victory states while the catalog
and page views leave room for other game types and genres.

## Risks And Decision Gate

Strudel is AGPL-3.0-or-later, so the authoring side must remain AGPL-compatible.
The runtime must be independently implemented and reviewed before claiming it
is safe for closed-source games. Samples require separate redistribution rights.

Continue beyond the prototype only if a developer can hear meaningful adaptive
variation within five minutes and the same exported score can be consumed by a
second runtime without authoring dependencies. Stop or pivot if transitions
sound like unrelated tracks crossfading, the portable contract leaks Strudel
implementation details, or acceptable output requires a composer-only workflow.
