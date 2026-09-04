# Gamestruments Godot 4 Kit — Release Plan

**Date**: 2026-09-04  
**Status**: Buyer docs + plan gate in progress. Fran approved standard price $24.99 and scope as drafted in kit-opportunity.md + kit-contract.md.  
**Branch**: jfrader/guri-564-plan-buyer-docs (off main at 1e6f717)  
**Scope protection**: smallest sellable (Linux GDExtension + source + minimal demo + docs + licenses) protected from feature creep.

## Milestones in Gate Order

Gates follow the game-kit-release lifecycle. Work returns to prior gate on failure. Current work adds the plan and buyer documentation (kit/docs/).

1. **Plan** (current)
   - Opportunity brief and product contract drafted and reviewed.
   - Milestones, acceptance criteria, claim-to-evidence matrix, verification profiles, clean-room tasks, risk register, and post-launch measurement contract defined in this document.
   - Price pinned (see kit-contract.md).
   - **Acceptance criteria**:
     - All main listing claims have explicit evidence paths (see matrix below).
     - Every verification profile has selected or explicit N/A + reason.
     - Clean-room buyer task list is self-contained and matches shipped archive + docs only.
     - Risks listed with owners/mitigations where known.
     - Measurement contract skeleton references opportunity brief kill/pivot criteria; baselines marked unknown.

2. **Demo**
   - Minimal Godot 4 scene exercising `project_secret` + palette + `generate(seed)` + `set_race_state(...)` (demo agent works separately; not part of this docs-only pass).
   - **Acceptance criteria** (for when demo lands):
     - Demo opens in Godot 4.7+, plays synthesized audio on "Music" bus after generate.
     - Inspector and code paths both work for secret/palette/seed/traits.
     - State changes (e.g. garage → grid → cruise → attack → final-lap → victory) produce audible bar-quantized transitions.
     - No samples or Strudel in the demo tree.

3. **Buyer docs** (this work)
   - `kit/docs/` added to repo: README.md, quickstart.md, api.md, limitations.md, troubleshooting.md.
   - Contract and opportunity updated to reference them.
   - All claims kept strictly literal to shipped behavior in `crates/godot/src/lib.rs` and `crates/engine/src/pocket_circuit.rs`.
   - **Acceptance criteria**:
     - Docs describe only implemented behavior (e.g. 6 sections, mono 22050 Hz synth, voice overrides after kit defaults, start at "garage", Linux .so only today).
     - Quickstart produces working integration in a fresh Godot 4 project using only the archive + docs.
     - Inventory in kit-contract.md lists the buyer doc paths.
     - CHANGELOG updated under Unreleased.

4. **Clean-room test**
   - Independent tester receives only the candidate zip + listing copy + buyer docs (no repo access, no verbal hints).
   - **Acceptance criteria**:
     - Tester identifies Godot 4.x + Linux requirement.
     - Opens demo or creates minimal scene, sets secret/palette/seed/traits in inspector or code, calls generate(seed), hears audio.
     - Triggers at least one state change via set_race_state and observes transition.
     - Confirms archive contains `kit/docs/` (or equivalent), LICENSE.md, CHANGELOG excerpt, no audio assets.
     - Time-to-first-sound and error messages are usable; failures lead to doc or product fixes + re-package.
     - Claims review passes (see matrix).

5. **Release candidate**
   - Source landed at immutable remote SHA/tag.
   - Archive built from pinned checkout; SHA-256 recorded.
   - All prior gates re-verified against the exact artifact.
   - **Acceptance criteria**:
     - Reproducible build (Linux .so + source) from the tag.
     - Headless + interactive smoke on Godot 4.7.2 succeeds.
     - Clean-copy extraction + verification matches candidate.
     - No secrets, private paths, or non-shipped content in archive.
     - Listing copy, screenshots, and docs match the artifact exactly.

6. **Launch**
   - itch.io upload + page published only after explicit Fran approval of the full release packet (artifact digest, price, settings, rollback).
   - No auto-publish.
   - **Acceptance criteria**:
     - Upload type set to Source code / Documentation (re-verify after replace).
     - Public page shows correct price $24.99, requirements, limitations, demo link (when available), and links to docs.
     - Public QA (unauthenticated) confirms copy, price, and absence of superseded claims.
     - Post-launch measurement contract activated.

## Claim-to-Evidence Matrix (Main Listing Claims)

| Claim | Evidence Path (must be demonstrable from archive + docs alone) | Verification |
|-------|----------------------------------------------------------------|--------------|
| seed + palette + secret → unique adaptive score | Different (secret, seed, palette, traits) tuples produce different score.id and different event pitches/voices (see engine parity test + generate_pocket_circuit) | Parity test (reserved take) + demo run with A/B seeds/palettes |
| zero samples | No .wav, .ogg, .mp3 or sample files in archive; synth is pure Rust voices (epiano, supersaw, pluck, chip, triangle, etc.) | `unzip -l` of final archive shows zero audio assets; source scan of crates/engine |
| Godot 4.x support | .gdextension declares compatibility_minimum = 4.2; code builds/runs against 4.7+; public API is Node subclass | Headless smoke on Godot 4.7.2 (project open + instantiate + generate); interactive run |
| bar-quantized state changes (set_race_state) | set_race_state(phase, intensity, pressure, final_lap) requests state; transport advances and crosses on bar boundaries (see AdaptiveTransport + rules) | Demo: call during play; observe section label changes on phrase boundaries; code review of process + request_state |
| deterministic output for (secret + seed + style + palette + traits) | Same inputs always produce identical PortableScore (id, events, bpm) | Reserved-take test (empty secret + level-004 + funk) reproduces catalog id; multiple runs in clean Godot |
| mono synth, no authoring UI | Internal synth at 22050 Hz mono; only GamestrumentsPlayer Node + two methods exposed; no editor tools or Strudel | API surface audit (only listed exports/methods); no UI nodes in addon tree |
| MIT + source included | crates/engine + crates/godot under MIT; full source + Cargo files + notices in archive | LICENSE.md + Cargo.toml license fields + third-party notices present in zip |

**Narrowed claims** (none invented; some were tightened during source audit):
- "adaptive score" narrowed to "bar-quantized crossovers between the six generated sections using the shipped adaptive rules" (no general "mood" or arbitrary state machine).
- Voice/palette overrides documented as post-kit overrides (code applies them after style_kit defaults).
- No claim of "Windows/macOS support" or "web export" (explicitly N/A for v1; see contract).
- "unique per level/playthrough" is true only within the deterministic tuple + generator version; buyers must supply the secret.

## Verification Profiles Selected

- **source/DCC**: N/A — no DCC / authored assets / 3D models / textures. This is a code + runtime synth kit. (No editable source in the DCC sense.)
- **engine/runtime**: Godot 4.7.2 headless (project import + script smoke) + interactive (run scene, inspector changes, state calls, listen on Music bus). Primary profile.
- **web demo**: N/A for v1 kit — planned later (see opportunity exclusions); current web demo is the separate Audio Lab (not shipped in runtime kit).
- **runtime/web security**: Selected (light). Review direct deps + lockfiles for critical vulns / abandoned / license issues; confirm no networking in the GDExtension path (no HTTP, no telemetry, no external services); product must run fully offline. gdext MPL-2.0 notices must ship. No credentials or debug endpoints in release builds.

## Clean-Room Buyer Task List (must pass with archive + docs only)

Tester receives: `gamestruments-<ver>-godot4.zip`, itch listing copy, and the `kit/docs/` contents.

1. Identify prerequisites from README (Godot 4.x, supported platform).
2. Extract and copy the addon tree (or equivalent) into a brand-new Godot 4 project under `res://addons/gamestruments/`.
3. Add a GamestrumentsPlayer node to a scene (or as autoload); set `project_secret` (non-empty), `style` (e.g. "funk" or "fusion"), optional voices and traits (0..1 floats) in inspector.
4. In a script's `_ready`, call `generate("my-level-seed")`.
5. Add AudioStreamPlayer or rely on internal; ensure "Music" bus exists or route appropriately; run the scene and confirm audio plays (mono synth).
6. Change seed/palette (via inspector reload or new generate call) and confirm different musical result.
7. During playback, call `set_race_state("grid", 0.7, 0.5, false)`, then `"race"`, `"attack"`, `final_lap=true` etc. and confirm audible transitions (not instant cuts).
8. Open `kit/docs/` (or equivalent) inside the extracted archive; confirm docs + LICENSE.md + CHANGELOG excerpt are present and match the listing.
9. Attempt one troubleshooting scenario from the docs (e.g. no sound → check Music bus).
10. Compare observed behavior against the claim matrix and limitations; note any mismatch.

Failure at any step → fix docs/product, re-archive, re-test.

## Risk Register

- **GDExtension adoption**: gdext + Godot AudioStreamGeneratorPlayback lifetime quirks (ObjectDB leaks fixed in current code via transient handles + explicit free). Mitigation: the cleanup in exit_tree / on_notification / Drop is part of shipped behavior; document "call generate before use" and bus routing. Re-test on minor Godot 4 patches.
- **Per-platform QA pending GURI-485**: Only linux.x86_64 binary in v1. Windows/macOS source builds possible but unverified at runtime. Buyers on other platforms get source escape hatch. Listed as explicit limitation.
- **Audio quality perception**: Pure mono synth (no samples) may sound "thin" to buyers expecting cinematic stems. Mitigation: honest limitations.md + demo that shows the adaptive strength rather than raw fidelity; style variety (fusion/neon/funk/chip) provides differentiation.
- **itch upload type pitfalls**: Replacing zip can reset type to "Executable". Mitigation: after every upload, re-set to Source code / Documentation (or appropriate), save, reload page, re-verify in editor and public view. Record in release packet.
- **Determinism across gdext/Godot patch**: Semantic + musical identity is the contract; byte-identical across patches is not promised. Documented in contract.
- **Support load from "no samples"**: Buyers cannot "just swap WAVs". Documented; success path is re-generate with different secret/seed/traits.

## Post-Launch Measurement Contract Skeleton

Tied directly to kill/pivot criteria in `docs/kit-opportunity.md`.

- **Metrics** (definitions):
  - itch views
  - demo starts (when web demo ships; for v1 use "downloads of kit" + support signals as proxy)
  - downloads (paid kit)
  - purchases
  - conversion (purchases / views or purchases / downloads)
  - refund rate
  - support tickets per 10 downloads (categorized: integration, no-sound, platform, docs, audio-quality)
- **Source**: itch.io analytics dashboard (primary); GitHub issues / Linear tags for support volume; manual review of public posts / forums. No in-product telemetry in v1.
- **Baseline**: unknown (pre-launch; first data will establish).
- **Attribution limits**: cannot distinguish "via Audio Lab link" vs "direct itch search" or "GitHub" without extra instrumentation (none for v1).
- **Minimum useful sample**: 50–100 downloads or 10–20 purchases (whichever first) before strong conclusions; smaller samples directional only.
- **Owner**: Fran
- **Review cadence**: weekly for first month post-launch, then monthly for first quarter, then quarterly (or on any kill/pivot threshold crossing).
- **Decision thresholds** (directly from opportunity brief kill/pivot; act if met):
  - paid downloads < 20 in first 60 days AND support tickets > 1 per 5 downloads → investigate discovery vs product-market fit; consider pivot or price test.
  - refund rate > 15% in first 100 sales → pause and audit docs + demo scope.
  - >3 distinct reports of "cannot get sound in fresh project after following README" after doc refresh → treat as integration failure; block further spend until fixed.
  - Any license or redistribution blocker → halt binary distribution.
  - (Additional: clean-room integration fails repeatedly → the generate + drive story is not self-service.)

All numbers are hypotheses; adjust after first data. Record actuals + decisions in post-launch notes.

## Next Steps (this branch)

- Complete buyer docs (`kit/docs/*`).
- Update kit-contract.md inventory + price pin.
- Add Unreleased changelog entries.
- Commit, push, open PR (do not merge).
- Demo / clean-room / candidate gates handled in follow-on work (GURI-564 tracked).

Evidence for this plan will be carried forward in the release packet.
