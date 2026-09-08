# Graph Report - gamestruments-GURI-639  (2026-09-08)

## Corpus Check
- 92 files · ~93,978 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1013 nodes · 1852 edges · 56 communities (46 shown, 7 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 22 edges (avg confidence: 0.84)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c78e7877`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- lantern-trail-generator.ts
- DemoAudioEngine
- audio-engine.ts
- generate-cli.ts
- scripts
- studio/package.json
- pocket-circuit-generator.ts
- AdaptiveTransport
- export-score.ts
- manifest.json
- generator.test.ts
- compilerOptions
- createDNA
- createSection
- compilerOptions
- compilerOptions
- wasm-engine.ts
- phaseMelodyOnsets
- strudel.d.ts
- createAuthoringScore
- generation-cli-smoke.mjs
- studio-import-smoke.mjs
- update-golden-fixtures.mjs
- pocket_circuit.rs
- GamestrumentsPlayer
- state.ts
- synth.rs
- render.rs
- Gamestruments
- runtime/package.json
- Gamestruments Godot 4 Kit — Product Contract
- Gamestruments Kit Opportunity Brief (2026-09-04)
- DeterministicRandom
- wasm-parity.mjs
- Unreleased
- Gamestruments Godot 4 Kit — Release Plan
- Limitations
- Troubleshooting
- Gamestruments Kit — Fran's Author QA Runbook (GURI-567)
- Gamestruments — Godot 4 Kit
- Prototype Contract
- Gamestruments Kit Demo
- Repository agent notes
- Opportunity
- API Reference — GamestrumentsPlayer
- Godot portable-score example
- Quickstart — Minimal Integration in a Fresh Godot 4 Project
- Engine Synth Voice Audit (GURI-563)
- In-game engine
- engine-boundary.md
- LICENSE.md
- gamestruments-engine
- package_kit.sh

## God Nodes (most connected - your core abstractions)
1. `DemoAudioEngine` - 48 edges
2. `generate_pocket_circuit()` - 29 edges
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
- `initializeLab()` --calls--> `AdaptiveTransport`  [EXTRACTED]
  apps/demo/src/state.ts → packages/runtime/src/transport.ts
- `activateExperiment()` --calls--> `AdaptiveTransport`  [EXTRACTED]
  apps/demo/src/state.ts → packages/runtime/src/transport.ts
- `toggleEngine()` --calls--> `AdaptiveTransport`  [EXTRACTED]
  apps/demo/src/state.ts → packages/runtime/src/transport.ts
- `generationPreset` --references--> `NormalizedMusicTraits`  [EXTRACTED]
  apps/demo/src/state.ts → packages/studio/src/pocket-circuit-generator.ts

## Import Cycles
- 1-file cycle: `crates/engine/src/synth.rs -> crates/engine/src/synth.rs`

## Communities (56 total, 7 thin omitted)

### Community 0 - "lantern-trail-generator.ts"
Cohesion: 0.05
Nodes (62): canonicalizeSeed(), clamp(), clampUnit(), createArrangementDNA(), createAuthoringScore(), createBassPattern(), createChordPattern(), createDNA() (+54 more)

### Community 1 - "DemoAudioEngine"
Cohesion: 0.15
Nodes (4): clamp(), DemoAudioEngine, deterministicUnit(), midiToFrequency()

### Community 2 - "audio-engine.ts"
Cohesion: 0.07
Nodes (46): createBitcrushCurve(), createTransitionCurve(), eventMatchesSolo(), NoteEvent, schedulingStartTick(), SectionBus, sectionSchedulingState(), SYNTH_VOICES (+38 more)

### Community 3 - "generate-cli.ts"
Cohesion: 0.09
Nodes (41): generationPreset, exportScore(), CommonGenerateCliOptions, emit(), FLAG_OPTIONS, GenerateCliOptions, GenerationRecipe, LANTERN_ONLY_OPTIONS (+33 more)

### Community 4 - "scripts"
Cohesion: 0.06
Nodes (34): dependencies, @strudel/core, @strudel/mini, devDependencies, @types/node, typescript, vite, vitest (+26 more)

### Community 5 - "studio/package.json"
Cohesion: 0.07
Nodes (28): @gamestruments/runtime, bin, gamestruments-generate, dependencies, @gamestruments/runtime, @strudel/core, @strudel/mini, engines (+20 more)

### Community 6 - "pocket-circuit-generator.ts"
Cohesion: 0.06
Nodes (36): BarMask, BASS_ONSET_MASKS, BassVoice, CHORD_ONSET_MASKS, clampUnit(), DRIVE_PHASES, HarmonyVoice, KEY_PITCH_CLASSES (+28 more)

### Community 7 - "AdaptiveTransport"
Cohesion: 0.25
Nodes (11): AdaptiveTransport, race_state_crosses_into_cruise(), Option, PortableScore, Result, Self, String, score() (+3 more)

### Community 8 - "export-score.ts"
Cohesion: 0.10
Nodes (27): AdaptiveRule, EventBase, NoteEvent, PercussionEvent, StemEvent, AuthoringLane, AuthoringScore, AuthoringSection (+19 more)

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

### Community 16 - "wasm-engine.ts"
Cohesion: 0.58
Nodes (8): allocAndWrite(), ensureLoaded(), generateScore(), GenerateScoreParams, getExports(), getMemory(), readOutput(), renderWav()

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

### Community 30 - "GamestrumentsPlayer"
Cohesion: 0.06
Nodes (26): AdaptiveCondition, AdaptiveTransport, Base, AdaptiveCondition, AdaptiveRule, GameState, MusicEvent, PortableScore (+18 more)

### Community 31 - "state.ts"
Cohesion: 0.07
Nodes (53): SoloMode, elements, requireElement(), animate(), applyGenerationRequest(), renderCurrentScore(), togglePlayback(), loopDistance() (+45 more)

### Community 32 - "synth.rs"
Cohesion: 0.14
Nodes (22): Biquad, bitcrush(), clamp_f(), compute_envelope(), compute_freq(), cruise_events_render_audible_samples(), deterministic_unit(), events_starting_at() (+14 more)

### Community 33 - "render.rs"
Cohesion: 0.12
Nodes (21): main(), produce(), Vec, main(), apply_outer_seam_fade(), decode_wav_samples(), encode_16bit_mono_wav(), envelope_variation() (+13 more)

### Community 34 - "Gamestruments"
Cohesion: 0.08
Nodes (23): Adaptive Arc, Integration Direction, Musical Identity, Pocket Circuit Music Brief, Sound Worlds, Validation Target, API, Catalog level-004 (parity test) (+15 more)

### Community 37 - "runtime/package.json"
Cohesion: 0.12
Nodes (15): engines, node, exports, files, dist, LICENSE, license, main (+7 more)

### Community 38 - "Gamestruments Godot 4 Kit — Product Contract"
Cohesion: 0.17
Nodes (11): Buyer Archive Name, Versioning, and Release Channels, Claim-to-Evidence Notes (internal), Demo Scope, Exact Inventory Buyers Receive, Explicit Non-Goals, Gamestruments Godot 4 Kit — Product Contract, License Terms, Price (+3 more)

### Community 39 - "Gamestruments Kit Opportunity Brief (2026-09-04)"
Cohesion: 0.17
Nodes (11): Compatibility and Distribution Assumptions, Evidence-Backed Pain and Existing Alternatives, Expected Acquisition Path and Proof Needed Before Launch, Gamestruments Kit Opportunity Brief (2026-09-04), Kill or Pivot Criteria, Largest Legal, Technical, Production, and Support Risks, Measurement Contract Skeleton, Price Range Hypothesis and Comparable Products (as of 2026-09-04) (+3 more)

### Community 42 - "DeterministicRandom"
Cohesion: 0.24
Nodes (6): DeterministicRandom, imul(), random_stream_is_deterministic(), Self, Vec, T

### Community 43 - "wasm-parity.mjs"
Cohesion: 0.31
Nodes (9): allocAndWrite(), __dirname, loadWasm(), main(), projectRoot, readOutput(), wasmPath, wasmRenderWav() (+1 more)

### Community 45 - "Unreleased"
Cohesion: 0.20
Nodes (9): Added, Added, Changed, Changelog, Fixed, Fixed, Fixed, Improved (+1 more)

### Community 46 - "Gamestruments Godot 4 Kit — Release Plan"
Cohesion: 0.22
Nodes (8): Claim-to-Evidence Matrix (Main Listing Claims), Clean-Room Buyer Task List (must pass with archive + docs only), Gamestruments Godot 4 Kit — Release Plan, Milestones in Gate Order, Next Steps (this branch), Post-Launch Measurement Contract Skeleton, Risk Register, Verification Profiles Selected

### Community 47 - "Limitations"
Cohesion: 0.22
Nodes (8): Audio Routing & Godot Specifics, Content Scope, Determinism and Versions, Integration Surface, Limitations, Platforms and Builds, Sections and Adaptation, Synthesis Model

### Community 48 - "Troubleshooting"
Cohesion: 0.22
Nodes (7): Different Results Than Expected / "Not the Same as Catalog", Extension Not Loaded / "GamestrumentsPlayer" Does Not Appear, Godot Errors About Leaks or "2 instances" at Quit, Rebuilding from Source (Advanced), State Changes Do Not Seem to Do Anything, Troubleshooting, Volume Too Low or "Thin" Sound

### Community 49 - "Gamestruments Kit — Fran's Author QA Runbook (GURI-567)"
Cohesion: 0.25
Nodes (7): (a) Download / extract the zip (copy-paste), (b) Open kit/demo in Godot 4.7 with the addon (copy-paste), (c) Interactive checks (copy-paste steps), (d) Claims check (tick against actual demo + shipped code), (e) Result recording template, Gamestruments Kit — Fran's Author QA Runbook (GURI-567), Notes for this run (GURI-567)

### Community 50 - "Gamestruments — Godot 4 Kit"
Cohesion: 0.25
Nodes (7): Gamestruments — Godot 4 Kit, Main Claims (Literal), Quickstart, Requirements, Scope (What Ships), Support, Verification

### Community 51 - "Prototype Contract"
Cohesion: 0.29
Nodes (6): Compatibility, Constraints, Included, Non-Goals, Promise, Prototype Contract

### Community 52 - "Gamestruments Kit Demo"
Cohesion: 0.29
Nodes (6): API gap observed while implementing, Gamestruments Kit Demo, How to use in any Godot 4 project, Notes on paths, Repo-local headless smoke test (this checkout), Verification

### Community 54 - "Repository agent notes"
Cohesion: 0.33
Nodes (5): Changelog, Linear workflow, Product, Repository agent notes, Verify

### Community 55 - "Opportunity"
Cohesion: 0.33
Nodes (5): Buyer And Job, Gap, Opportunity, Prototype Hypothesis, Risks And Decision Gate

### Community 56 - "API Reference — GamestrumentsPlayer"
Cohesion: 0.33
Nodes (5): API Reference — GamestrumentsPlayer, Exported Properties (Inspector + Code), Methods, Observable Behavior Buyers Can Rely On, What Is Not Part of the Supported Public API

### Community 58 - "Godot portable-score example"
Cohesion: 0.40
Nodes (4): Files, Godot portable-score example, Regenerate the score, Run

### Community 59 - "Quickstart — Minimal Integration in a Fresh Godot 4 Project"
Cohesion: 0.40
Nodes (4): Next, Prerequisites, Quickstart — Minimal Integration in a Fresh Godot 4 Project, Steps

### Community 61 - "Engine Synth Voice Audit (GURI-563)"
Cohesion: 0.50
Nodes (3): Audit Table, Engine Synth Voice Audit (GURI-563), Verification commands (run in this dir)

### Community 62 - "In-game engine"
Cohesion: 0.50
Nodes (3): In-game engine, Known issues, Toolchain

## Knowledge Gaps
- **361 isolated node(s):** `NoteEvent`, `SynthVoice`, `TransitionCurve`, `SectionBus`, `SynthVoiceSettings` (+356 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 446 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `PortableScore` connect `audio-engine.ts` to `lantern-trail-generator.ts`, `DemoAudioEngine`, `generate-cli.ts`, `pocket-circuit-generator.ts`, `export-score.ts`, `generator.test.ts`, `wasm-engine.ts`, `state.ts`?**
  _High betweenness centrality (0.043) - this node is a cross-community bridge._
- **Why does `DemoAudioEngine` connect `DemoAudioEngine` to `audio-engine.ts`, `state.ts`?**
  _High betweenness centrality (0.035) - this node is a cross-community bridge._
- **Why does `AdaptiveTransport` connect `audio-engine.ts` to `lantern-trail-generator.ts`, `generator.test.ts`, `state.ts`?**
  _High betweenness centrality (0.014) - this node is a cross-community bridge._
- **Are the 8 inferred relationships involving `generate_pocket_circuit()` (e.g. with `main()` and `produce()`) actually correct?**
  _`generate_pocket_circuit()` has 8 INFERRED edges - model-reasoned connections that need verification._
- **What connects `NoteEvent`, `SynthVoice`, `TransitionCurve` to the rest of the system?**
  _361 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `lantern-trail-generator.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.0539906103286385 - nodes in this community are weakly interconnected._
- **Should `audio-engine.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.0662004662004662 - nodes in this community are weakly interconnected._