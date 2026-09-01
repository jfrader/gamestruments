# Pocket Circuit Music Brief

## Validation Target

The first Gamestruments score is for Pocket Circuit Online's Phase 0 Kitchen
track: a top-down 2D arcade race with tiny vehicles in an oversized everyday
environment. Races eventually run for two to five minutes.

The game specification establishes these constraints:

- music is energetic and built from compact loops;
- the final lap adds a distinct dynamic layer;
- the garage and hub use lower intensity;
- engine, tire, collision, boost, surface, and hazard sounds take mix priority.

## Musical Identity

The score should feel polished, technical, optimistic, and fast. The working
palette combines jazz-fusion harmony with precise electronic rhythm: electric
keys, clean synthetic lead, controlled bass, and compact drum-machine parts.
This references a broad tradition of sophisticated racing soundtracks without
copying any existing melody, recording, arrangement, sound library, or branding.

All prototype voices are synthesized in the browser. No third-party samples
ship with the demonstration.

## Sound Worlds

| Score | Genre | Distinguishing language |
|---|---|---|
| Countertop Velocity | Electronic fusion | Electric-piano lead, organ drive pads, locked eighth-note pulse |
| Neon Hairpin | Synthwave | Supersaw lead, pulse pads, dotted-eighth echo |
| Tiny Torque | Pocket funk | Clav/pluck lead and stabs; Race Flow uses the denser Grid groove |
| Micro Motor Panic | Chiptune | Bitcrushed squares, triangle bass, arcade vibrato |

Every score implements the same six-state adaptive arc and carries a four-bar
lead motif through each state. Seeded alternate takes change safe ornament
notes while preserving tempo, harmony, transition rules, and the score's
melodic identity.

Countertop Velocity adds 10 BPM to the shared generated tempo range so its
fusion groove carries the extra pace expected from an arcade race.

Audition mode can isolate the melody or backing, jump directly to a section,
and toggle between adjacent seeded takes without changing the section under
review. Direct jumps temporarily override game-state selection until a race
control changes.

## Adaptive Arc

| State | Input | Musical response |
|---|---|---|
| Garage | `racePhase=garage` | Half-time keys, sparse kit, low melodic density |
| Starting grid | `racePhase=grid` | Repeating pulse and firmer bass |
| Race flow | `racePhase=race` | Full groove and procedural lead choices |
| Position fight | pressure ≥ 0.68 or intensity ≥ 0.72 | Denser drums, darker harmony, stronger lead |
| Final lap | `finalLap=1` | Maximum density, lifted lead, dedicated stem marker |
| Victory | finish + win | Major-color release without changing the shared pulse |

Transitions start on a bar boundary and use a two-bar equal-power crossover.
Within each score, sections share tempo, four-beat bars, four-bar loop length,
and a common harmonic center so overlapping parts remain intentional.

## Integration Direction

Pocket Circuit is Godot 4.7, but the prototype remains outside its repository.
After musical validation, a tracked Pocket Circuit issue should add a Godot
adapter that maps race-domain signals to the portable runtime. Music buses must
leave headroom for gameplay audio and respect the game's separate volume
controls. No integration should land before vehicle feel remains the project's
priority.
