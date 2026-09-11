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

Your game generates its own soundtrack. Gamestruments writes deterministic, sample-free adaptive music right inside Godot — no audio files to ship, no authoring tool, no external services. Add one node, generate a score at level load, and tell it what's happening; it moves between musical sections on bar boundaries.

**Try it in your browser:** https://gamestruments.gurisitos.games

### Two recipes, one player

- **Racing** — six sections from garage to victory, driven by speed, rival pressure, and lap state.
- **Suspense** — song-form tension for infiltration, hacking, and horror, with a form gameplay can hold or advance.

### Up and running in a minute

1. Copy `addons/gamestruments/` into your project.
2. Add a `GamestrumentsPlayer` node.
3. Set `project_secret` and a style, then call `generate(seed)`.
4. Drive it with `set_race_state(...)` for Racing or `set_trace_state(...)` for Suspense.

Copy-paste GDScript is in the archive `README.md` and `kit/docs/quickstart.md`.

### What you get

- Linux, Windows, and universal macOS libraries with the GDExtension descriptor.
- Full MIT Rust source, lockfile, and pinned toolchain.
- A playable four-circuit Godot demo (Night Circuit, Harbor Sprint, Canyon Run, Micro Mile), one style each.
- Quickstart, API, limitations, troubleshooting, changelog, licenses, and a release manifest with hashes.

### Good to know

- Godot 4.7.x on Linux x86_64, Windows x86_64, and macOS arm64/x86_64.
- Fully offline: no network, no samples, no telemetry. 22050 Hz mono synth routed through Godot's audio buses.
- Not a general-purpose music graph or DAW. Voices are synthesized; no sample import or MIDI export.
- First-party code is MIT (closed-source commercial games welcome); gdext is MPL-2.0.

### Updates and support

All future updates to this kit are included with your purchase. Questions or bugs? Post in the comments with your OS, Godot version, and Output text. Refunds follow itch.io's standard terms.
