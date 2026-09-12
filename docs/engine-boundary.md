# Engine Boundary (Audio Lab + shared Rust engine)

Generation of `PortableScore` (sections, events, id, bpm, rules, crossfades, defaultSection):
- Owned by the shared Rust engine in `crates/engine` (`racing`, `suspense`,
  `medieval`, and score). WASM `gamestruments_score_json` accepts
  `"recipe": "racing" | "suspense" | "medieval"` (default racing).
- Medieval additionally accepts styles `court`, `minstrel`, and `chapel` and
  generates seven eight-bar scene sections. `crates/engine/src/medieval.rs` owns
  the section plans, selection rules, and `select_medieval_section`; the Rust
  `AdaptiveTransport` exposes `request_medieval_state`.
- Suspense additionally accepts `"arrangement": "original" | "extended"`.
  The lab defaults to Extended; omitting the field in the engine API retains
  Original byte-for-byte for compatibility. `suspense_arrangement` builds 16-bar Extended main beds with
  subdued felt/dusk textures, a continuous rhythm and occasional seeded tom
  details. The existing `generate_suspense` API remains Original. Godot exposes the same `arrangement`
  property, defaulting to `"original"`.
- Scan → Scan II and Breach → Breach II are independent, cueable 16-bar sections.
  The II variations begin with Anomaly-inspired textures, then develop response
  phrases instead of repeating eight bars twice. Scan and Breach retain their
  approved base material; Anomaly itself is unchanged.
- Extended retains the original motifs, with four-bar melodic spotlights only in
  Decrypt and Other Hall, and maintains its kick/hat grid through normal sections. Full drum dropouts are limited to
  the opening, Break, Disconnect and Closed Session.
- Seeded reverse-cymbal swells and air impacts are optional (at most one random
  effect per ordinary section, in addition to Breach II's quiet closing swell).
  Anomaly is a single eight-bar detour after Decrypt: staggered
  tonal pulses over the same drum grid, with an authored swell/impact pair.
- Exposed to the lab via WASM: `gamestruments_score_json` (see `crates/engine/src/wasm.rs`).
- The TypeScript `@gamestruments/studio` generator is retained only for legacy authoring fixtures and catalog research. It is not a runtime or parity authority.

Transport / section selection + transition planning:
- The lab's music selector and Cue rows enumerate the loaded score's complete
  section list, including endings. Suspense's game signals are separate,
  collapsed controls; Medieval's scene buttons cue one section per scene. Live cues wait for the next bar; the latest request made
  during a blend queues after it. Cancel removes only a waiting request, never
  an in-progress blend. Stopped selection sets the next starting section.
- For the web Audio Lab: blessed implementation is `@gamestruments/runtime` (`AdaptiveTransport` + `selectSection` over score rules + nested GameState).
- The Rust `AdaptiveTransport` + `select_section` (in `crates/engine/src/transport.rs`) mirror the behavior for Godot/games.
- This small state machine duplication is accepted and documented. Native/WASM parity covers Rust score identity and synthesis; transport implementations have their own behavior tests.
- Extended declares `form.origin = "transitionStart"`: the incoming fade counts
  toward the section duration instead of replaying two opening bars before
  the next handoff. Original omits this field to retain its checkpoint
  behavior. The Godot aligned-form renderer uses a sample-count clock,
  overlapping tonal synths and a single percussion owner through the fade.

Game-controlled form (after `generate` succeeds):
- `cue_section(id)` cues a musical section on a bar boundary. Existing blends
  finish first; this does not change the hold state.
- `set_form_hold(true)` repeats the current section instead of advancing the
  form automatically. An active blend finishes and its incoming section is held;
  an unstarted automatic cue is canceled. Explicit game/manual cues still work.
- `advance_form()` cues the next form section once, retaining the hold state.
- `set_form_hold(false)` resumes automatic progression at the current loop end,
  without rewinding or replaying overdue transitions after a long hold.
- `get_current_section()` reports the current/entering section ID, and
  `is_form_held()` reports the hold state. Endings have no next form step;
  cue a section explicitly to leave them.

The canonical IDs are `verse` (Scan), `scan-ii`, `chorus` (Breach), and
`breach-ii`. Use the calls below at their corresponding gameplay events—not
all at once in a single frame:

```gdscript
music.set_form_hold(true)
music.cue_section("verse")  # Begin scanning; keep this section available.
music.advance_form()        # On progress: enter Scan II, still held.
music.cue_section("chorus") # On exploit: enter Breach.
music.advance_form()        # On escalation: enter Breach II, still held.
music.set_form_hold(false)  # Let the current loop finish, then continue automatically.
```

The lab exposes the same behavior with Hold section, Next section and Resume
automatic. These are playback controls, not additional arrangements or gameplay
phase names. Original and Extended remain the only arrangement choices.

Audio synthesis / sound stage:
- The signed-off sound (SYNTH_VOICES: warm/glass/pulse/pluck/chip + dedicated epiano/organ/supersaw/triangle/bass + kit + full room bus + stereo imaging + compression) lives in `apps/demo/src/audio-engine.ts` (Web Audio).
- The Rust `Synth` (mono, no room) is the reference for the game engine only.
- Felt/dusk use softer attacks, low-pass shaping and longer releases. Their
  quiet delayed repeats stay inside the section's tonal/melody mix, so solo
  controls and crossfades also control the effect. The lab additionally filters
  the repeat and retains its stereo room stage.
- Noise effects use their event duration and finite cleanup tails. Reverse swells
  use the tonal fade bus in the lab so their peak/tail is not hard-muted at a
  section boundary; impacts use the percussion bus.
- Path A chosen: WASM owns generation (and could own transport math), Web Audio owns the voices. No AudioWorklet streaming of Rust synth (would lose the signed-off room sound without a full port; listening comparison impossible here).
- The Audio Lab is therefore an authoring interface, not evidence of the exact sound shipped in the Godot kit. Storefront audio and video must come from the packaged Godot runtime.

WASM loader for lab: `apps/demo/src/wasm-engine.ts` (plain instantiateStreaming + typed `generateScore` / `renderWav` wrappers; follows the ownership/alloc contract from wasm.rs and tests/wasm-parity.mjs).

See also: README, docs/procedural-generation.md, AGENTS.md (one engine, two frontends).
