# Gamestruments - itch.io Storefront Listing

This is the exact, repo-owned source for the Gamestruments itch.io listing.
Any updates to the storefront must be approved here first.
For official itch creator documentation, see: https://itch.io/docs/creators/

## Metadata Fields

- **Title:** Gamestruments — Adaptive Music for Godot 4
- **Recommended Slug:** gamestruments-godot
- **Short Description:** A seed-driven, sample-free adaptive music engine for Godot 4.
- **Classification:** Game Assets
- **Kind:** Downloadable
- **Pricing:** $12.99 minimum / Pay-what-you-want above (No launch discount)
- **Language:** English
- **Tags:** godot, godot-4, music, adaptive-music, dynamic-music, procedural, racing, soundtrack, audio, engine
- **Release Status:** Released
- **Platforms:** No OS executable flags (the zip contains libraries/source, not a standalone OS executable).
- **Community:** Comments enabled for public support.
- **External Links:** Browser preview at https://gamestruments.gurisitos.games (Audio Lab: same generator, browser audio layer). The source repository is private.

## Visual Thesis and Media Capture Checklist

**Visual Thesis:** Precision music sequencer fused with neon racing telemetry—dark, exact, energetic, uncluttered.

**Content Plan:**
1. Product/claim
2. Exact-runtime proof
3. Adaptive workflow
4. Requirements/limitations
5. Purchase CTA

**Media Capture Checklist:**
- [ ] All screenshots must come from the immutable packaged Godot demo.
- [ ] All audio and video evidence must be captured directly from the immutable packaged Godot runtime.
- [ ] Do NOT use browser Audio Lab captures as kit media. The Audio Lab link is allowed as a preview; kit media must be captured from the packaged Godot demo.
- [ ] Cover image follows the 315:250 ratio (630x500 recommended).
- [ ] Include 3–5 screenshots of playable racing, rival pressure, final lap, and finish. Generator panels or manual section selection are not gameplay evidence.
- [ ] Include an optional YouTube/Vimeo video demonstrating the runtime audio transitions.

## Long Description

### Gamestruments — Adaptive Music for Godot 4

Gamestruments is a seed-driven, sample-free adaptive music engine built specifically for Godot 4. Generate a deterministic score at level load from a per-title namespace, instrument palette, and seed, then drive bar-quantized state changes at runtime. The shipped Racing recipe scores races from garage through victory; no samples, no web authoring UI, and no external runtime services.

**Try it in your browser:** https://gamestruments.gurisitos.games — the Audio Lab preview uses the same generator with a browser audio layer; the included Godot demo is the exact in-game sound.

### Use It in Your Game
1. Copy the `addons/gamestruments/` directory into your Godot 4 project at `res://addons/gamestruments/`.
2. Add a `GamestrumentsPlayer` node.
3. Set a stable non-empty `project_secret` namespace, choose a style, call `generate(seed)` at level load, and check its result.
4. Drive states via `set_race_state(...)`; changes commit on musical bar boundaries.

The archive root `README.md` and `kit/docs/quickstart.md` contain copy-paste GDScript examples. The player uses a `Music` bus when present and otherwise routes to `Master`.

### Included Files
The buyer archive contains:
- `addons/gamestruments/` — Linux x86_64, Windows x86_64, and universal macOS arm64/x86_64 libraries plus the GDExtension descriptor. Ships two recipes: Racing and Suspense (song-form, original and extended arrangements).
- `kit/demo/` — four circuits (Night Circuit, Harbor Sprint, Canyon Run, Micro Mile), a playable three-lap race with grip-assisted steering, boost, one rival, gameplay-driven music, and a readable integration adapter; each circuit regenerates a different shipped style (neon, pocket funk, fusion, micro motor) with its own seed. Includes an identical addon copy.
- `kit/docs/` — quickstart, API, limitations, and troubleshooting documentation.
- Source crates, the required catalog fixture, `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml` — complete pinned Rust rebuild inputs.
- First-party and third-party license terms.
- A machine-readable release manifest with source identity, workflow provenance, tool versions, and native-library hashes.

### Literal Claims
- Generates a deterministic score based on seed, style, palette, traits, and version.
- Two shipped recipes: Racing and Suspense (song-form), the latter with original and extended arrangements and gameplay-holdable form.
- Four Racing styles (neon, pocket funk, fusion, micro motor), demonstrated by four demo circuits that regenerate live in the garage.
- Six adaptive racing sections (garage, grid, cruise, attack, final-lap, victory) with bar-quantized crossovers.
- Fully offline runtime generation; no network requests or external services.
- Synthesized in-process audio; zero samples or pre-baked tracks required.

### Requirements
- **Godot Version:** Godot 4.7.x via GDExtension. Future Godot minor releases are not implied. Tested exactly against CI version 4.7.2.
- **Operating Systems:** Linux x86_64 (built on Ubuntu 24.04), Windows x86_64, and macOS arm64/x86_64. No mobile or web targets.
- **Playback:** Godot `AudioStreamGenerator` routed to a 22050 Hz mono internal synth.

### Limitations
- Racing and song-form state models; this is not a general-purpose adaptive music graph.
- Voices are synthesized; there is no sample import or MIDI export.
- Mono internal synth designed for lean in-game playback.
- No broad compatibility or certification guarantees.
- The macOS library is ad-hoc signed but not Developer ID-signed or notarized. Linux and Windows libraries are not publisher-signed.

### License and Disclosures
- Gamestruments first-party Rust crates, addon descriptor, buyer documentation, and demo integration are provided under the MIT license, permitting use in closed-source commercial games.
- gdext and related binding crates are MPL-2.0. Complete attribution and dependency disclosures are provided in `THIRD_PARTY_NOTICES.md` inside the archive.

### Updates
All future updates to this kit are included with your purchase. itch.io automatically delivers new files uploaded to this page to existing owners, and major releases are announced through the page's devlog.

### Support
Support is provided best-effort for reproducible defects within the advertised environment. Use the public comments section on this page and include the kit version, OS, architecture, exact Godot version, reproduction steps, and complete Godot Output text. Use itch.io's purchase-support flow for purchase-specific or private matters.
Refunds follow the standard itch.io terms presented at purchase time.
