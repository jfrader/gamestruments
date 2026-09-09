# Graph Report - gamestruments-GURI-644  (2026-09-09)

## Corpus Check
- 102 files · ~99,014 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1063 nodes · 1925 edges · 64 communities (47 shown, 9 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 23 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `fb797bd0`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- lantern-trail-generator.ts
- DemoAudioEngine
- runtime/src/index.ts
- generate-cli.ts
- scripts
- studio/package.json
- pocket-circuit-generator.ts
- AdaptiveTransport
- manifest.json
- generator.test.ts
- compilerOptions
- createDNA
- createSection
- compilerOptions
- compilerOptions
- godot-package-smoke.mjs
- phaseMelodyOnsets
- verify_kit_archive.sh
- strudel.d.ts
- createAuthoringScore
- generation-cli-smoke.mjs
- studio-import-smoke.mjs
- update-golden-fixtures.mjs
- pocket_circuit.rs
- score.rs
- state.ts
- synth.rs
- render.rs
- Gamestruments
- build-wasm.mjs
- runtime/package.json
- Gamestruments Adaptive Racing Music for Godot 4 — Product Contract
- Gamestruments Kit Opportunity Brief (updated 2026-09-08)
- DeterministicRandom
- wasm-parity.mjs
- Third-Party Notices
- Unreleased
- Gamestruments Godot 4 Kit — Release Plan
- Limitations
- Troubleshooting
- Gamestruments Kit — Author QA Runbook
- Gamestruments — Adaptive Racing Music for Godot 4
- Prototype Contract
- Gamestruments Kit Demo
- verify-third-party-notices.mjs
- Repository agent notes
- Opportunity
- API Reference — GamestrumentsPlayer
- Godot portable-score example
- Quickstart — Fresh Godot Project
- Engine Synth Voice Audit (GURI-563)
- In-game engine
- engine-boundary.md
- Licensing
- gamestruments-engine
- package_kit.sh

## God Nodes (most connected - your core abstractions)
1. `DemoAudioEngine` - 48 edges
2. `generate_pocket_circuit()` - 31 edges
3. `build_section_faithful()` - 28 edges
4. `AdaptiveTransport` - 22 edges
5. `PortableScore` - 21 edges
6. `GamestrumentsPlayer` - 18 edges
7. `compilerOptions` - 18 edges
8. `SectionId` - 16 edges
9. `clamp()` - 15 edges
10. `Style` - 15 edges

## Surprising Connections (you probably didn't know these)
- `DemoAudioEngine` --references--> `PortableScore`  [EXTRACTED]
  apps/demo/src/audio-engine.ts → packages/runtime/src/types.ts
- `generationPreset` --references--> `NormalizedMusicTraits`  [EXTRACTED]
  apps/demo/src/state.ts → packages/studio/src/pocket-circuit-generator.ts
- `generationPreset` --references--> `PocketCircuitStyle`  [EXTRACTED]
  apps/demo/src/state.ts → packages/studio/src/pocket-circuit-generator.ts
- `initializeLab()` --calls--> `AdaptiveTransport`  [EXTRACTED]
  apps/demo/src/state.ts → packages/runtime/src/transport.ts
- `activateExperiment()` --calls--> `AdaptiveTransport`  [EXTRACTED]
  apps/demo/src/state.ts → packages/runtime/src/transport.ts

## Import Cycles
- 1-file cycle: `crates/engine/src/synth.rs -> crates/engine/src/synth.rs`

## Communities (64 total, 9 thin omitted)

### Community 0 - "lantern-trail-generator.ts"
Cohesion: 0.05
Nodes (63): canonicalizeSeed(), clamp(), clampUnit(), createArrangementDNA(), createAuthoringScore(), createBassPattern(), createChordPattern(), createDNA() (+55 more)

### Community 1 - "DemoAudioEngine"
Cohesion: 0.09
Nodes (25): clamp(), createBitcrushCurve(), createTransitionCurve(), DemoAudioEngine, deterministicUnit(), eventMatchesSolo(), midiToFrequency(), NoteEvent (+17 more)

### Community 2 - "runtime/src/index.ts"
Cohesion: 0.09
Nodes (37): allocAndWrite(), ensureLoaded(), generateScore(), GenerateScoreParams, getExports(), getMemory(), readOutput(), renderWav() (+29 more)

### Community 3 - "generate-cli.ts"
Cohesion: 0.05
Nodes (62): AdaptiveRule, AuthoringLane, AuthoringScore, AuthoringSection, clampVelocity(), exportLane(), exportScore(), exportSection() (+54 more)

### Community 4 - "scripts"
Cohesion: 0.05
Nodes (38): dependencies, @strudel/core, @strudel/mini, devDependencies, @playwright/test, @types/node, typescript, vite (+30 more)

### Community 5 - "studio/package.json"
Cohesion: 0.07
Nodes (28): @gamestruments/runtime, bin, gamestruments-generate, dependencies, @gamestruments/runtime, @strudel/core, @strudel/mini, engines (+20 more)

### Community 6 - "pocket-circuit-generator.ts"
Cohesion: 0.06
Nodes (36): BarMask, BASS_ONSET_MASKS, BassVoice, CHORD_ONSET_MASKS, clampUnit(), DRIVE_PHASES, HarmonyVoice, KEY_PITCH_CLASSES (+28 more)

### Community 7 - "AdaptiveTransport"
Cohesion: 0.25
Nodes (11): AdaptiveTransport, race_state_crosses_into_cruise(), Option, PortableScore, Result, Self, String, score() (+3 more)

### Community 9 - "manifest.json"
Cohesion: 0.08
Nodes (25): checksum, algorithm, encoding, source, value, domainSeeds, arrangement, harmony (+17 more)

### Community 10 - "generator.test.ts"
Cohesion: 0.12
Nodes (12): createPocketCircuitMusicalDNA(), DEFAULT_POCKET_CIRCUIT_TRAITS, POCKET_CIRCUIT_DNA_SEED_VERSION, POCKET_CIRCUIT_GENERATOR_DOMAINS, POCKET_CIRCUIT_GENERATOR_VERSION, POCKET_CIRCUIT_SECTION_PLANS, BALANCED_TRAITS, firstBarDurationSignature() (+4 more)

### Community 11 - "compilerOptions"
Cohesion: 0.06
Nodes (32): apps/**/*.ts, DOM, DOM.Iterable, packages/runtime/src/index.ts, packages/**/*.ts, tests/**/*.ts, vite.config.ts, vite.studio.config.ts (+24 more)

### Community 12 - "createDNA"
Cohesion: 0.34
Nodes (9): createArrangementDNA(), createDNA(), createHarmonyDNA(), createMotifDNA(), createOrnamentDNA(), createRhythmDNA(), createTimbreDNA(), DeterministicRandom (+1 more)

### Community 13 - "createSection"
Cohesion: 0.26
Nodes (13): arrangementGroove(), clamp(), createBassPattern(), createChordPattern(), createLiftPattern(), createSection(), createVictorySparklePattern(), formatBarPattern() (+5 more)

### Community 14 - "compilerOptions"
Cohesion: 0.12
Nodes (16): compilerOptions, allowImportingTsExtensions, declaration, declarationMap, emitDeclarationOnly, lib, module, moduleResolution (+8 more)

### Community 15 - "compilerOptions"
Cohesion: 0.10
Nodes (20): compilerOptions, allowImportingTsExtensions, declaration, declarationMap, emitDeclarationOnly, module, moduleResolution, noEmit (+12 more)

### Community 16 - "godot-package-smoke.mjs"
Cohesion: 0.20
Nodes (8): addonRoot, args, demoRoot, godot, godotArgument, library, libraryArgument, projectRoot

### Community 17 - "phaseMelodyOnsets"
Cohesion: 0.28
Nodes (9): breakConsecutiveRuns(), createMelodyPattern(), createPercussionPattern(), distributedSteps(), fillSteps(), melodyDegreesForBar(), percussionOnsets(), phaseMelodyOnsets() (+1 more)

### Community 19 - "strudel.d.ts"
Cohesion: 0.22
Nodes (5): @strudel/mini, StrudelFraction, StrudelHap, StrudelPattern, StrudelSpan

### Community 21 - "createAuthoringScore"
Cohesion: 0.29
Nodes (7): canonicalizeSeed(), createAuthoringScore(), derivePocketCircuitDomainSeeds(), derivePocketCircuitSubSeed(), generatePocketCircuitAuthoringScore(), hashText(), scoreId()

### Community 29 - "pocket_circuit.rs"
Cohesion: 0.07
Nodes (71): a_id(), arrangement_groove(), ArrangementDna, b_id(), bass_mask(), break_consecutive_runs(), build_section_faithful(), chord_mask() (+63 more)

### Community 30 - "score.rs"
Cohesion: 0.11
Nodes (21): AdaptiveCondition, AdaptiveCondition, AdaptiveRule, GameState, MusicEvent, PortableScore, PortableSection, rejects_a_pitch_outside_midi_range() (+13 more)

### Community 31 - "state.ts"
Cohesion: 0.06
Nodes (57): SoloMode, elements, requireElement(), animate(), applyGenerationRequest(), renderCurrentScore(), togglePlayback(), cssNumber() (+49 more)

### Community 32 - "synth.rs"
Cohesion: 0.07
Nodes (35): AdaptiveTransport, AudioStreamPlayer, Base, Biquad, bitcrush(), clamp_f(), compute_envelope(), compute_freq() (+27 more)

### Community 33 - "render.rs"
Cohesion: 0.11
Nodes (24): AsRef, main(), produce(), Vec, main(), apply_outer_seam_fade(), decode_wav_samples(), encode_16bit_mono_wav() (+16 more)

### Community 34 - "Gamestruments"
Cohesion: 0.08
Nodes (23): Adaptive Arc, Integration Direction, Musical Identity, Pocket Circuit Music Brief, Sound Worlds, Validation Target, API, Catalog level-004 (parity test) (+15 more)

### Community 35 - "build-wasm.mjs"
Cohesion: 0.22
Nodes (9): destination, destinationDir, projectRoot, result, rustflags, source, sourceDigest(), sourceFiles() (+1 more)

### Community 37 - "runtime/package.json"
Cohesion: 0.12
Nodes (15): engines, node, exports, files, dist, LICENSE, license, main (+7 more)

### Community 38 - "Gamestruments Adaptive Racing Music for Godot 4 — Product Contract"
Cohesion: 0.20
Nodes (9): Buyer Archive, Buyer-Reliable Behavior, Explicit Non-Goals, Gamestruments Adaptive Racing Music for Godot 4 — Product Contract, License and Support, Product Scope, Supported Environment, Supported Public API (+1 more)

### Community 39 - "Gamestruments Kit Opportunity Brief (updated 2026-09-08)"
Cohesion: 0.17
Nodes (11): Compatibility and Distribution Assumptions, Evidence-Backed Pain and Existing Alternatives, Expected Acquisition Path and Proof Needed Before Launch, Gamestruments Kit Opportunity Brief (updated 2026-09-08), Kill or Pivot Criteria, Largest Legal, Technical, Production, and Support Risks, Measurement Contract Skeleton, Price Range Hypothesis and Comparable Products (as of 2026-09-04) (+3 more)

### Community 42 - "DeterministicRandom"
Cohesion: 0.24
Nodes (6): DeterministicRandom, imul(), random_stream_is_deterministic(), Self, Vec, T

### Community 43 - "wasm-parity.mjs"
Cohesion: 0.32
Nodes (10): allocAndWrite(), __dirname, loadWasm(), main(), projectRoot, readOutput(), readResponse(), wasmPath (+2 more)

### Community 44 - "Third-Party Notices"
Cohesion: 0.29
Nodes (6): MIT, MIT or Apache License 2.0, MIT or Unlicense, Mozilla Public License 2.0, Third-Party Notices, Unicode License v3

### Community 45 - "Unreleased"
Cohesion: 0.20
Nodes (9): Added, Added, Changed, Changelog, Fixed, Fixed, Fixed, Improved (+1 more)

### Community 46 - "Gamestruments Godot 4 Kit — Release Plan"
Cohesion: 0.29
Nodes (6): Claim-to-Evidence Matrix, Clean-Room Buyer Tasks, Gamestruments Godot 4 Kit — Release Plan, Gate Order, Post-Launch Measurement, Risks

### Community 47 - "Limitations"
Cohesion: 0.29
Nodes (6): Audio, Determinism and Persistence, Integration and Support, Limitations, Product Scope, Runtime and Platforms

### Community 48 - "Troubleshooting"
Cohesion: 0.22
Nodes (8): Different Result Than a Previous Version, `GamestrumentsPlayer` Does Not Appear, `generate()` Returns `false`, Leak or Crash During Shutdown, Rebuild From Source, Silence After Successful Generation, State Does Not Change, Troubleshooting

### Community 49 - "Gamestruments Kit — Author QA Runbook"
Cohesion: 0.29
Nodes (6): Automated Evidence, Claims Audit, Extracted Demo, Gamestruments Kit — Author QA Runbook, Release Packet Header, Result

### Community 50 - "Gamestruments — Adaptive Racing Music for Godot 4"
Cohesion: 0.29
Nodes (6): Gamestruments — Adaptive Racing Music for Godot 4, Literal Product Claims, Quickstart, Requirements, Support, What Ships

### Community 51 - "Prototype Contract"
Cohesion: 0.29
Nodes (6): Compatibility, Constraints, Included, Non-Goals, Promise, Prototype Contract

### Community 52 - "Gamestruments Kit Demo"
Cohesion: 0.40
Nodes (4): Demonstrated API, Gamestruments Kit Demo, Repository Smoke Test, Run the Packaged Demo

### Community 53 - "verify-third-party-notices.mjs"
Cohesion: 0.50
Nodes (3): metadata, [metadataPath, noticesPath], missing

### Community 54 - "Repository agent notes"
Cohesion: 0.33
Nodes (5): Changelog, Linear workflow, Product, Repository agent notes, Verify

### Community 55 - "Opportunity"
Cohesion: 0.33
Nodes (5): Buyer And Job, Gap, Opportunity, Prototype Hypothesis, Risks And Decision Gate

### Community 56 - "API Reference — GamestrumentsPlayer"
Cohesion: 0.33
Nodes (5): API Reference — GamestrumentsPlayer, Exported Properties, `generate`, Observable Contract, `set_race_state`

### Community 58 - "Godot portable-score example"
Cohesion: 0.40
Nodes (4): Files, Godot portable-score example, Regenerate the score, Run

### Community 59 - "Quickstart — Fresh Godot Project"
Cohesion: 0.33
Nodes (5): Configure, Drive It, Install, Prerequisites, Quickstart — Fresh Godot Project

### Community 61 - "Engine Synth Voice Audit (GURI-563)"
Cohesion: 0.50
Nodes (3): Audit Table, Engine Synth Voice Audit (GURI-563), Verification commands (run in this dir)

## Knowledge Gaps
- **381 isolated node(s):** `NoteEvent`, `SynthVoice`, `TransitionCurve`, `SectionBus`, `SynthVoiceSettings` (+376 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 472 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **9 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `PortableScore` connect `runtime/src/index.ts` to `lantern-trail-generator.ts`, `DemoAudioEngine`, `generate-cli.ts`, `pocket-circuit-generator.ts`, `generator.test.ts`, `state.ts`?**
  _High betweenness centrality (0.034) - this node is a cross-community bridge._
- **Why does `DemoAudioEngine` connect `DemoAudioEngine` to `runtime/src/index.ts`, `state.ts`?**
  _High betweenness centrality (0.024) - this node is a cross-community bridge._
- **Why does `AdaptiveTransport` connect `runtime/src/index.ts` to `lantern-trail-generator.ts`, `generator.test.ts`, `state.ts`?**
  _High betweenness centrality (0.014) - this node is a cross-community bridge._
- **Are the 8 inferred relationships involving `generate_pocket_circuit()` (e.g. with `main()` and `produce()`) actually correct?**
  _`generate_pocket_circuit()` has 8 INFERRED edges - model-reasoned connections that need verification._
- **What connects `NoteEvent`, `SynthVoice`, `TransitionCurve` to the rest of the system?**
  _381 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `lantern-trail-generator.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.05220288781932617 - nodes in this community are weakly interconnected._
- **Should `DemoAudioEngine` be split into smaller, more focused modules?**
  _Cohesion score 0.08593396653098145 - nodes in this community are weakly interconnected._