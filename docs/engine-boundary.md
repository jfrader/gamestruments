# Engine Boundary (Audio Lab + shared Rust engine)

Generation of `PortableScore` (sections, events, id, bpm, rules, crossfades, defaultSection):
- Owned by the shared Rust engine in `crates/engine` (`racing`, `suspense`,
  `adventure`, `folklore`, and score). WASM `gamestruments_score_json` accepts
  `"recipe": "racing" | "suspense" | "adventure" | "folklore"` (default racing).
  Folklore offers `chacarera` (default; nine guitar-and-bombo sections) and
  `carnavalito` (five sections in 2/4 with synthesized charango, quena and bombo).
  Both attach repeating song forms. They are available in the Lab/WASM, not the
  Godot recipe selector. `generate_folklore` retains the Chacarera default;
  `generate_folklore_with_style` accepts an explicit `FolkloreStyle`.
- Adventure accepts styles `folk`, `dark`, and `orchestral` and generates
  sixteen sections: the eight selected from area phase plus discovery, threat,
  and quest progress (`camp`, `explore`, `town`, `dungeon`, `combat`, `boss`,
  `sanctuary`, `victory`), plus a combat set (`skirmish`, `assault`, `chase`)
  and a happiness set (`festival`, `reunion`, `dawn`), plus guitar-and-bombo
  alternatives (`explore-strings`, `town-strings`) in the seeded composer's pool.
  These additions are cue targets, not new area phases. `camp`, `dungeon`, `boss`, `sanctuary`, `skirmish`,
  `assault`, and `dawn` are 16 bars; the rest are 32.
  `crates/engine/src/adventure.rs` owns the plans, rules, and
  `select_adventure_section`; the Rust `AdaptiveTransport` exposes
  `request_adventure_state`.
- Racing and Adventure optionally attach a song form through
  `apply_automatic_arrangement` when `autoplay` is set (default `false`).
  Racing `original` tours garage, grid, cruise (×2), attack, final-lap, victory,
  then loops from grid; Racing `extended` tours all ten sections once each, then
  loops from grid; Adventure tours all eight sections, then loops from explore.
  With autoplay off, both recipes stay state-driven and byte-for-byte unchanged.
  The adapter attaches a form only when autoplay is explicitly true; otherwise
  the wrapper output keeps its empty (`None`) form.
- Suspense accepts `"arrangement": "all-phases" | "seeded"`. The phase pool is
  its only authority; the retired `original`/`extended`/`theme` names still
  parse but resolve to the seeded default, as does an omitted or empty field.
  `all-phases` plays every pool phase once in canonical order; `seeded` lets the
  composer choose the count, roles, order and loop point from the seed.
  Godot exposes the same `arrangement` property, defaulting to the recipe's own
  choice (Racing → original, Suspense → seeded).
- Racing additionally accepts `"arrangement": "original" | "extended"`. `original`
   (the default, and returned for an omitted or empty field) is the
   six-section race. `extended` keeps those six sections
  identical and adds four new cueable sections harvested from the same piece and
  palette — `ignition` (8 bars), `slipstream` (16 bars), `redline` (16 bars), and
  `cooldown` (8 bars) — for a ten-section order: garage, ignition, grid, cruise,
  slipstream, attack, redline, final-lap, victory, cooldown. Extended carries its
  own `-extended-v2` id and ` — Extended` title suffix; the six original
  sections, rules, and default `garage` opening are unchanged, and the new
  sections are form-tour/cue targets only — no new gameplay state switches.
  `generate_racing_arrangement` in `crates/engine/src/racing_arrangement.rs`
  dispatches Original to `generate_racing` and Extended to `generate_extended`;
  the autoplay adapter is applied afterward and leaves Original untouched.
   Native and WASM default to Original for Racing; the Audio Lab defaults every
   recipe to Seeded.
- Racing's shared All phases/Seeded pool also contains `breather`, `switchback`
  and `open-road`: thirteen form phases plus three signal-only cue targets.
  Generator `1.12.0` reworks `cruise` and `attack`; Garage, Starting Grid,
  Final Lap and Finish retain their previous musical material.
- The Suspense pool has 38 phases: the fourteen base sections, `scan-ii` (Scan II) and
  `breach-ii` (Breach II) developed past their base phase, `anomaly`, and ten
  pool-authored phases (`half-time`, `sparse`, `sub-groove`, `syncopated`,
  `drive`, `drum-break`, `false-stop`, `filter-break`, `harmonic-bridge`,
  `step-up-bridge`).
- Scan → Scan II and Breach → Breach II are independent, cueable 16-bar sections.
  Their response phrases come from the set's shared harmonic arc, so a join is a
  cadence rather than a reset.
- The pool's drum kits are seeded: most phases play the pool groove, and a seeded
  mode borrows the full kick–snare–kick–snare backbeat. Fills and licks vary per
  seed and per reel take.
- Seeded reverse-cymbal swells and air impacts stay optional; the transition pass
  picks a fill, riser, lift or tail — plus a landing — for every join, so no two
  unions are treated identically. Layers stay under the approved mix ceilings.
- Exposed to the lab via WASM: `gamestruments_score_json` (see `crates/engine/src/wasm.rs`).
- The Rust engine is the single generation authority; there is no TypeScript generator pipeline. `@gamestruments/runtime` is the Lab's TS transport only.

Transport / section selection + transition planning:
- The lab's music selector and Cue rows enumerate the loaded score's complete
  section list, including endings. Suspense's game signals are separate,
  collapsed controls; Adventure's area-phase buttons each cue one section. Live cues wait for the next bar. `LivePlayer` runs one
  blend at a time, a section blend or a music blend; a request made during one
  waits, keeping only the latest music change and the latest section. A waiting
  music change and section become one blend that opens the new music on that
  section, and the clock keeps counting through it. Cancel removes only a
  waiting section, never an in-progress blend. Stopped selection sets the next starting section.
- The Rust `AdaptiveTransport` + `select_section` (in `crates/engine/src/transport.rs`) decide sections and transitions for Godot and for the Audio Lab, which runs them through `LivePlayer`. `@gamestruments/runtime` keeps a TypeScript transport for other consumers; the Lab no longer uses it.
- A sounding section keeps its phrase origin (where it began fading in): a finished blend or a form repeat moves only the form's count, so the renderer never restarts a section that is still sounding.
- Pool forms leave `form.origin` unset: the incoming fade overlaps the previous
  section instead of replaying two opening bars before the next handoff. The
  Godot aligned-form renderer uses a sample-count clock, overlapping tonal synths
  and a single percussion owner through the fade.

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

The lab exposes the same behavior for every score that has a form — Suspense,
and Racing/Adventure when autoplay is on — with a "Hold auto tour" / "Resume
auto tour" toggle, a "Next" button that names the upcoming section, and
per-section Cue buttons. These are playback controls, not additional
  arrangements or gameplay phase names. The pool is Suspense's only authority;
  the retired Original/Extended/Theme presets are gone, and autoplay is a
  separate opt-in flag.

The lab's arrangement selector is per recipe and remembered separately: Racing,
Suspense and Adventure each offer All phases/Seeded (default Seeded), so a
Suspense pool switch never leaks into Racing. When a Racing score switches from
Seeded to Original while a new phase (`ignition`, `slipstream`, `redline`, or
`cooldown`) is playing, the lab falls back to `garage`, since Original has no
such section.

Audio synthesis / sound stage:
- The Rust `Synth` (mono, no room) renders the game and the Lab.
- `LivePlayer` limits the final mixed output as well as each voice, including
  music handoffs. The Godot adapter and Lab worklet split its mono output across
  stereo channels at equal power, preserving its loudness rather than doubling
  the channel energy.
  Acoustic voices (harp/recorder/vielle/bell/nylon-guitar/charango/quena/marimba plus
  frame-drum/tambourine/bombo/bombo-rim) are synthesized, not sample recordings.
- Felt/dusk use softer attacks, low-pass shaping and longer releases. Their
  quiet delayed repeats stay inside the section's tonal/melody mix, so solo
  controls and crossfades also control the effect.
- Noise effects use their event duration and finite cleanup tails.
- The Lab plays the engine itself (decision A of GURI-822, Decide how the Lab stays faithful to the engine): `LivePlayer` (`crates/engine/src/live.rs`, the player the Godot addon runs) renders in WASM on the page and `apps/demo/src/pcm-queue-worklet.ts` plays its mono output, so the Lab sounds like the shipped kit. The Audition controls use the engine's `Solo`. Storefront audio and video still come from the packaged Godot runtime.

WASM loader for lab: `apps/demo/src/wasm-engine.ts` (plain instantiateStreaming + typed `generateScore` / `renderWav` / `createPlayer` wrappers; follows the ownership/alloc contract from wasm.rs and tests/wasm-parity.mjs). The live player's JSON commands are documented in `crates/engine/src/wasm.rs`.

See also: README, docs/procedural-generation.md, AGENTS.md (one engine, two frontends).
