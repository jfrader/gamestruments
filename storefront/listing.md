# Gamestruments - itch.io Storefront Listing

This is the exact, repo-owned source for the Gamestruments itch.io listing.
Any updates to the storefront must be approved here first.
For official itch creator documentation, see: https://itch.io/docs/creators/

The 0.1.3 copy below became current on 2026-09-15 (see `docs/releases/v0.1.3.md`
for exact artifacts). It supersedes the 0.1.2 copy published on 2026-09-13
(`docs/releases/v0.1.2.md`). The dated devlogs at the end are historical and are
not edited.

## Metadata Fields

- **Title:** Gamestruments — Adaptive Music for Godot 4
- **Recommended Slug:** gamestruments-godot
- **Short Description:** Procedural music that adapts to your game, in Godot and HTML5.
- **Classification:** Game Assets
- **Kind:** Downloadable
- **Pricing:** $12.99 minimum / Pay-what-you-want above (No launch discount)
- **Language:** English
- **Tags:** adaptive-music, audio, dynamic-music, godot, godot-4, music, procedural, racing, soundtrack, adventure
- **Release Status:** Released
- **Released version:** 0.1.3 (2026-09-15)
- **Platforms:** No OS executable flags (the zip contains libraries/source, not a standalone OS executable).
- **Community:** Comments enabled for public support.
- **External Links:** Live demo at https://gurisitosgames.itch.io/gamestruments-audio-lab-demo and standalone browser preview at https://gamestruments.gurisitos.games (Godot and HTML5, one engine). The source repository is private.

## Visual Thesis and Media Capture Checklist

**Visual Thesis:** Use the existing Audio Lab's visual identity. Show the actual music and controls, then the small Godot integration that drives them. No separate showcase design.

**Content Plan:**
1. Hear the existing lab and understand the product
2. See how game state changes the music
3. See the included examples and integration steps
4. Check requirements and limitations
5. Purchase the kit

**Media Capture Checklist:**
- [x] Build the itch HTML preview from the same approved lab source, with its actual phases and controls unchanged.
- [x] Clearly identify the interactive browser experience as **Audio Lab preview**; do not present it as exact native playback or as a bundled Godot interface.
- [ ] Capture native integration screenshots and native-audio evidence from the final extracted Godot kit, not the development checkout.
- [x] Cover image follows the 315:250 ratio (630x500 recommended).
- [ ] Show one readable integration script and the native example it runs. Keep API calls legible rather than filling the page with code screenshots.
- [ ] Include a short transition video recorded from the final native kit; it is required before promoting this update.
- [ ] Fran has approved the music excerpts and finished video.

## Long Description

### Gamestruments — Adaptive Music for Godot 4

**Procedural music that adapts to your game.** Gamestruments generates deterministic, sample-free music in Godot and in HTML5. Choose a recipe and seed, generate a score when your scene loads, then tell the player what's happening; it blends between musical sections on bar boundaries. No sample library, authoring tool, or cloud service is required.

**Try the demo first:** [open the interactive Audio Lab](https://gurisitosgames.itch.io/gamestruments-audio-lab-demo) or [use the standalone browser preview](https://gamestruments.gurisitos.games).

### Recipes

- **Racing** — pace, rival pressure, lap state, and the finish.
- **Suspense** — song-form tension for infiltration, hacking, and horror.
- **Adventure** — an eight-section fantasy quest arc.

### What you get

- A prebuilt Godot 4 addon for Linux x86_64, Windows x86_64, and macOS arm64/x86_64, with the GDExtension descriptor.
- **Mastered output.** Every render passes one shared master stage — highpass, compressor, limiter — and measures around −14 LUFS, with true peak held below −1.7 dBTP so nothing reaches 0 dBFS. 48 kHz mono.
- The full MIT-licensed Rust source, lockfile, and pinned toolchain.
- Complete docs: quickstart, API reference, limitations, and troubleshooting.
- Three native example scenes (playback, game signals, song form) with their own addon copy. No browser or network needed.

### Getting started

1. Copy `addons/gamestruments/` into your project and restart Godot.
2. Add a `GamestrumentsPlayer` child, set the recipe, style, and title namespace, then call `generate(seed)` and check the result.
3. Connect your game events to `set_race_state(...)` or `set_trace_state(...)` for the chosen recipe.

The complete public docs and one free example script ship in a separate free download on this page: `gamestruments-docs-and-example.zip`.

### Requirements

- Godot 4.7.x on Linux x86_64, Windows x86_64, or macOS arm64/x86_64.
- Also runs in HTML5. Native playback is fully offline. Hear it in the browser before you buy.
- First-party code is MIT; gdext is MPL-2.0.

All future updates to this kit are included with your purchase. Questions or bugs? Post in the comments.

## Published downloads (live 2026-09-15)

- Paid kit upload: `gamestruments-0.1.3-godot4.zip`, 9985245 bytes,
  SHA-256 `4e519eeef592eca7e725f13d8075a5943b2f536ce2d8b67aa5005f72670d1fa3`,
  type **Source code**, no OS executable flags. Built by the release workflow
  from tag `v0.1.3` (commit `b43f3f2634385980ba9e8d12fcc0c00d719f4fc3`). This
  replaces `gamestruments-0.1.2-godot4.zip` (9840405 bytes, SHA-256
  `1af99d4c0ef5f4cd6092ce6660acedfc946155bae172c2fc6c491a76ac1a20b2`), which is
  removed from the page so buyers only get the current kit.
- Free per-file demo download on the same page:
  `gamestruments-docs-and-example.zip`, 21983 bytes,
  SHA-256 `a39569b8ca72e376d39679624703783258df6255a08c7dafec3c3edc4732a8c4`,
  type **Documentation or Instructions**, free-demo flag set. Contains the
  complete buyer docs plus one `example.gd` from the canonical quickstart; no
  addon or native binaries, and the example needs the paid addon to run.
  Rebuild with `npm run build:public-docs`.
- HTML Lab preview: `gamestruments-lab-itch.zip`, 250833 bytes,
  SHA-256 `08c72a66c94523932a7d9032edb43de39129a48d1f4c2eecd0e38a83ec83a449`,
  browser-play enabled, embed 960x600, mobile friendly, fullscreen.
  Rebuild with `npm run build:itch`.
- Free sample video: `gamestruments-samples.mp4`, type **Video**, free-demo flag
  set. 24 s of the native engine — Racing neon cruise, Suspense terminal theme,
  Adventure folk explore — rendered at 48 kHz from the v0.1.3 engine and
  mastered to about −14 LUFS. Regenerate it whenever the engine's output level
  or sample rate changes, or it will misrepresent the kit.
- itch.io's per-file demo checkbox is the mechanism for the free download
  (official reference: <https://itch.io/docs/creators/getting-started>).
- Demo-to-kit navigation stays same-tab. The source repository stays private.

## Video sequence (pending)

Target 35–45 seconds. Music and captions only; no voiceover or sales copy. Code
excerpts come from the tested canonical public example, using its existing API
and phase names. Native-audio evidence must be captured from the packaged native
kit. Music is approved; the recording is still outstanding.

| Time | Caption | Shot |
|---|---|---|
| 0:00–0:07 | Procedural music for your game | Let an approved native music excerpt play before showing code. |
| 0:07–0:11 | Generate a soundtrack | Highlight the setup and `generate` call in the public example; keep the music playing. |
| 0:11–0:20 | Music follows the action | Show an actual game-state change and let the transition be heard. |
| 0:20–0:24 | Call it from your game | Highlight the matching event callback in the same tested example. |
| 0:24–0:36 | Find the sound for your game | Contrast another approved style or section; no new controls or invented phases. |
| 0:36–0:45 | Try the demo | End with the actual lab and kit destination. |

Keep native and browser footage clearly identified. Final cuts should follow
the music rather than forcing a transition to fit these draft timings.

## Update Devlog — v0.1.3 (2026-09-15)

Published: <https://gurisitosgames.itch.io/gamestruments-godot/devlog/1665444/v013-turned-it-up-and-it-never-clips>

**Title:** v0.1.3 — Turned it up, and it never clips

Gamestruments 0.1.3 is out. If you already own the kit, grab the latest download — updates are included, and this one is worth grabbing.

**The library was too quiet.** Renders were landing around −32 LUFS: roughly 20 dB below any normal listening reference, with the level swinging 4.7 LU between styles. Next to a commercial track, the music sounded broken.

**One master stage fixes it.** Every render now passes the same chain the Audio Lab uses — highpass, compressor, limiter — instead of hard-clipping the raw voice sum:

- **−14.9 to −14.0 LUFS** across the render pack (was −35.8 to −31.1), a 0.9 LU spread between styles
- True peak **−2.0 to −1.7 dBTP** against a −1.0 dBTP ceiling, with a hard clamp backstop on both the offline and realtime paths
- Nothing reaches 0 dBFS

**48 kHz.** The default render rate moves from 22050 Hz to 48000 Hz across the engine, the WASM build, the parity reference, the examples, and the Godot extension.

**What it means for your project.** Same API and no code changes, but your mix balance will move. If you tuned effects or ambience against the old quiet renders, rebalance against the new level. Pocket Circuit is the first consumer and is regenerating its baked loops.

Full release notes: https://github.com/jfrader/gamestruments/releases/tag/v0.1.3

**Hear it:** [try the browser lab](https://gurisitosgames.itch.io/gamestruments-audio-lab-demo) or the [standalone preview](https://gamestruments.gurisitos.games). Then [get the kit](https://gurisitosgames.itch.io/gamestruments-godot).

## Update Devlog — v0.1.2 (2026-09-13)

Published: <https://gurisitosgames.itch.io/gamestruments-godot/devlog/1662999/v012-adventure-music-more-racing-and-a-simpler-kit>

**Title:** v0.1.2 — Adventure music, more Racing, and a simpler kit

Gamestruments 0.1.2 is out. If you already own the kit, just download the latest version — updates are included.

**New: Adventure music.** An eight-section fantasy arc — camp, explore, town, dungeon, combat, boss, sanctuary, victory — in three styles: folk, dark, and orchestral. Your game drives it with one call: `set_adventure_state(area, discovery, threat, quest_complete)`.

**More Racing.** The original six phases are untouched, and four new ones join them: Ignition, Slipstream, Redline, and Cooldown. In the browser lab, Racing now tours all ten on its own. Existing game code keeps using the original six; the extended arrangement is opt-in.

**Easier to start.** The download now ships three small Godot examples instead of the old racing game:

- play a title theme with `generate()`
- feed gameplay values into the music with `set_race_state()`
- hold, advance, and cue sections with a song form

The full docs and one example script are also a free download on the product page, so you can read the integration before buying.

**Hear it:** [try the browser lab](https://gurisitosgames.itch.io/gamestruments-audio-lab-demo) or the [standalone preview](https://gamestruments.gurisitos.games). Then [get the kit](https://gurisitosgames.itch.io/gamestruments-godot).

## Launch Devlog (2026-09-11)

**Title:** Now live: music your game writes as it plays

Gamestruments is out! It's an adaptive music engine for Godot 4 — your game generates its own soundtrack at runtime, with no audio files, no authoring tool, and no external services.

The first release ships two recipes:

- **Racing** — six sections from the garage to the finish line, driven by speed, rival pressure, and lap state.
- **Suspense** — song-form tension for infiltration, hacking, and horror, with a form your gameplay can hold or advance.

It runs on Linux, Windows, and macOS, includes the full MIT Rust source, and comes with a playable four-circuit demo so you can hear it before writing any code.

Try it in your browser: https://gamestruments.gurisitos.games
Get the kit: https://gurisitosgames.itch.io/gamestruments-godot

All future updates are included. Building something with it? I'd love to see it — questions and bug reports are welcome in the comments.
