# Gamestruments Kit Opportunity Brief (2026-09-04)

## Target Buyer and Job to be Done

Solo and small-team Godot 4 developers who want unique, adaptive, instrumental music scores that react to gameplay state at runtime.

The job: obtain a musically coherent adaptive score from a single seed + per-title secret + instrument palette, generate the score once at level load inside the game binary, and drive bar-quantized state changes (e.g. race phase, intensity, pressure, final-lap) without shipping samples, Strudel, or a full middleware authoring tool.

## Evidence-Backed Pain and Existing Alternatives

Market research (2026-09-04) found no seed-driven, sample-free, runtime-adaptive music engine shipping as a ready-to-use Godot 4 component.

- Pre-authored stem packs + simple players: $5–30 on itch and asset stores. Static or lightly switched; buyers complain they still sound "pre-baked" and require manual arrangement work.
- Loopsmith (authoring tool): $49. Powerful but requires the buyer to author inside the tool and export; not a drop-in runtime generator.
- ChipForge (Unity-only): $19.99. Demonstrates demand for procedural/chiptune generation but unavailable to Godot users.
- FMOD / Wwise free tiers: powerful, zero direct cost, but steep learning curve, authoring outside the engine, and integration overhead disproportionate for solo/small teams.
- Dead Godot 3 GDScript experiments: several abandoned repos; none reached a maintained, sample-free, seed + runtime contract for Godot 4.

Buyers who want "unique per level/playthrough without composing or sampling" currently either hand-author loops, pay for limited stem packs, or adopt heavy middleware.

## Product Promise and One-Sentence Differentiator

Promise: a MIT-licensed Rust core + Godot 4 GDExtension that lets a game call `generate(secret, seed, style, traits)` at level load and then `set_race_state(...)` (or equivalent) at runtime; the engine produces and renders a deterministic adaptive score using only synthesized voices, with bar-aware transitions.

One-sentence differentiator: the first seed-driven, sample-free, runtime-adaptive music generator purpose-built for Godot 4 that ships as a small GDExtension with full MIT Rust source and zero authoring UI or sample dependencies.

## Smallest Sellable Scope and Explicit Exclusions

Smallest sellable: Linux x86_64 GDExtension binary + .gdextension file + the MIT Rust crates source (engine + godot glue) + a minimal demo scene that exercises project_secret + palette + seed + generate + state changes + docs + licenses + changelog.

Explicit exclusions (non-goals for v1 kit):
- No authoring UI or Strudel Lab.
- No sample import or redistribution.
- No Godot < 4 support.
- No FMOD/Wwise interop.
- No Windows/macOS binaries (pending GURI-485 platform QA).
- No web export, mobile, or console targets in first release.
- No pre-rendered WAV catalog shipping with the kit (catalog takes are for validation only).

## Compatibility and Distribution Assumptions

- Engine: Godot 4.x (minimum 4.2 per .gdextension, tested on 4.7+).
- Binding: GDExtension (gdext 0.4.x series at time of build).
- Platforms shipped in v1: linux.x86_64 only.
- Source: buyers receive the Rust crates under MIT; they rebuild or use the provided binary.
- File formats: no external score JSON at runtime for the kit path (generation is inside the extension); demo may include example data.
- Distribution: itch.io as a zip; buyers drop `addons/gamestruments/` (or equivalent) into their project.

## Expected Acquisition Path and Proof Needed Before Launch

Primary path: itch.io "Godot" + "audio" + "procedural" / "music" tags, linked from the free Audio Lab demo and GitHub.

Proof needed before launch (per game-kit-release gates):
- Opportunity + contract docs approved.
- Clean-room buyer can integrate the archive into a fresh Godot 4 project, set secret/palette/seed, call generate at _ready, hear music, and drive state changes from game logic using only the shipped docs.
- Reproducibility of reserved takes (e.g. empty secret + level-004 + funk) matches catalog parity test.
- Linux binary + source build both produce working player.
- No Strudel or samples leak into the runtime path.

## Price Range Hypothesis and Comparable Products (as of 2026-09-04)

Standard price hypothesis: $19–25.

Comparables (dated research):
- Generic adaptive stem packs + players on itch/asset stores: $5–30 (common $10–20).
- Loopsmith (full authoring + export tool): $49.
- ChipForge (Unity procedural chiptune kit): $19.99.
- FMOD/Wwise: free for small use but high integration cost.
- Abandoned Godot 3 procedural experiments: $0 (but dead).

Positioning: priced like a solid Unity procedural kit, above basic stem packs, below a full authoring tool, justified by runtime generation inside the game, MIT source, zero samples, and Godot-4-native GDExtension.

## Largest Legal, Technical, Production, and Support Risks

- Legal: gdext dependency is MPL-2.0 (our crates declare MIT); must ship correct combined notices and confirm binary redistribution obligations. Strudel remains strictly outside the runtime (AGPL authoring lab only).
- Technical: Godot 4.x AudioStreamGeneratorPlayback lifetime / ObjectDB leaks observed in current gdext 0.4.5 + 4.7; documented but must not block buyer use. Cross-platform audio parity and determinism on Windows/macOS unproven (GURI-485 gate).
- Production: release workflow must produce reproducible per-OS .so/.dll/.dylib + sha256 from tags; no pre-built Godot addons yet.
- Support: buyers will expect "works in Godot 4.3–4.x on the three desktop OSes"; any platform gap or version skew becomes support load. No sample library means buyers cannot fall back to "just swap the audio files."

## Kill or Pivot Criteria

Kill or pivot if:
- After first 30–60 days on itch, < X paid downloads or conversion below Y% with no evidence of discovery problem (see measurement).
- Clean-room integration fails repeatedly even after doc fixes (i.e. the "generate at load + drive state" story is not self-service).
- A critical blocker (e.g. persistent audio leaks affecting shipped games, non-reproducible output across minor Godot patch, or license incompatibility) cannot be resolved without changing the core promise.
- A better open alternative appears that removes the differentiation.

## Measurement Contract Skeleton

- Metrics (definitions):
  - itch views / downloads / purchases (direct storefront numbers).
  - "Successful first generate" proxy: not directly measurable; use support volume + refund rate + "works in Godot X" forum mentions as inverse signals.
  - Support tickets per 10 downloads (categorized: integration, audio glitch, platform, docs).
- Sources: itch.io dashboard (views, sales, refunds), GitHub issues, Linear support tags, manual review of public posts.
- Baseline: unknown (pre-launch).
- Minimum useful sample: 50–100 downloads or 10–20 purchases (whichever first) before drawing strong conclusions; smaller samples treated as directional only.
- Attribution limits: cannot distinguish "discovered via Audio Lab demo" vs "direct itch search" without extra instrumentation (none planned for v1).
- Review owner/cadence: Fran; first review 14 days post-launch, then monthly for first quarter, then quarterly.
- Decision thresholds (tied to kill/pivot):
  - If paid downloads < 20 in first 60 days AND support tickets > 1 per 5 downloads → investigate discovery vs product-market fit; consider pivot or price test.
  - Refund rate > 15% in first 100 sales → pause and audit docs + demo scope.
  - >3 distinct reports of "cannot get sound in fresh project after following README" after doc refresh → treat as integration failure, block further spend until fixed.
  - Any license or redistribution blocker discovered pre-shipment of binaries → halt binary distribution until resolved.

(Owner: Fran. All numbers and thresholds are hypotheses to be validated or adjusted after first data.)
