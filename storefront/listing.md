# Gamestruments - itch.io Storefront Listing

This is the exact, repo-owned source for the Gamestruments itch.io listing.
Any updates to the storefront must be approved here first.
For official itch creator documentation, see: https://itch.io/docs/creators/

The 1.0.1 copy below became current on 2026-09-17 (see `docs/releases/v1.0.1.md`
for exact artifacts). It supersedes the 0.1.3 copy published on 2026-09-15
(`docs/releases/v0.1.3.md`). The dated devlogs at the end are historical and are
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
- **Released version:** 1.0.1 (2026-09-17) — uploaded; the Score Debugger is author-only (`?debug`)
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
- [x] Capture native integration screenshots and native-audio evidence from the final extracted Godot kit, not the development checkout. Re-captured 2026-09-18 with the packaged smoke: `playback-play`, `game_signals-race`, `song_form-form` (+ narrow and missing-addon states).
- [x] Cover image follows the 315:250 ratio (630x500 recommended).
- [ ] Show one readable integration script and the native example it runs. Keep API calls legible rather than filling the page with code screenshots.
- [x] Promotion video published: `gamestruments-demo-final.mp4` (type Video, free demo). It is the approved live Audio Lab walkthrough — Fran rejected the raw native panel capture as a showcase; native evidence lives in the screenshots and the `samples` download.
- [x] Fran has approved the music excerpts and finished video (2026-09-18, the 106 s walkthrough and the 98 s portrait reel).

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

## Published downloads (live 2026-09-17)

- Paid kit upload: `gamestruments-1.0.1-godot4.zip`, 10695434 bytes,
  SHA-256 `f5724e947b66f91e4864e8a916ecd85f2a1203e1e1e76d393307354d89b4cd25`,
  type **Source code**, no OS executable flags. Built by the release workflow
  from tag `v1.0.1` (commit `5e78a4db43895ddc73a44138d425e7a454bf168a`). This
  replaces `gamestruments-1.0.0-godot4.zip` (10679228 bytes, SHA-256
  `2ee9d10ef6a0ea6b0e82479c1dfa817d5798c0eb02ed71202009852c02baaf74`), which is
  removed from the page so buyers only get the current kit.
- Free per-file demo download on the same page:
  `gamestruments-docs-and-example.zip`, 22609 bytes,
  SHA-256 `b447d6c89fd7614cd70240d7ab7d2ac69f0b1ef2b5d649c9eb665fc4d6f8d6d1`,
  type **Documentation or Instructions**, free-demo flag set. Contains the
  complete buyer docs plus one `example.gd` from the canonical quickstart; no
  addon or native binaries, and the example needs the paid addon to run.
  Rebuild with `npm run build:public-docs`.
- HTML Lab preview: `gamestruments-lab-itch.zip`, 250833 bytes,
  SHA-256 `08c72a66c94523932a7d9032edb43de39129a48d1f4c2eecd0e38a83ec83a449`,
  browser-play enabled, embed 960x600, mobile friendly, fullscreen.
  Rebuild with `npm run build:itch`.
- Promotion walkthrough video: `gamestruments-demo-final.mp4`, 18855439 bytes,
  SHA-256 `b09c38402061930863b5414764a002a836ba281e3b0db6077dc98ae7e59ba179`,
  type **Video**, free-demo flag set. 106 s, 1920x1080, about -15 LUFS, the live
  Audio Lab driven at human pace (Suspense/Anomaly, Racing/Position Fight,
  Adventure/Boss; sound worlds noir, neon, orchestral; cursor visible; no
  captions; no stops). The 98 s 1080x1920 portrait reel for Instagram is
  `gamestruments-reel-portrait.mp4` (not on the page).
- Free sample video: `gamestruments-samples.mp4`, type **Video**, free-demo flag
  set. 24 s of the native engine — Racing neon cruise, Suspense terminal theme,
  Adventure folk explore — rendered at 48 kHz from the v0.1.3 engine and
  mastered to about −14 LUFS. Regenerate it whenever the engine's output level
  or sample rate changes, or it will misrepresent the kit.
- itch.io's per-file demo checkbox is the mechanism for the free download
  (official reference: <https://itch.io/docs/creators/getting-started>).
- Demo-to-kit navigation stays same-tab. The source repository stays private.

## Video (published 2026-09-18)

Fran approved a live Lab walkthrough instead of the storyboarded native cut:
the final 106 s video (`gamestruments-demo-final.mp4`) is one continuous Lab
session — play, cue the strong sections (Anomaly, Position Fight, No Retreat),
drag Speed intensity, switch the sound world (noir, neon, orchestral) — with
the Lab's own audio captured in-page, no captions or overlays, a visible
cursor, and zero silence. The portrait Instagram reel (98 s) is the same
session recorded natively at a phone viewport.


 Final cuts should follow
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

## Update Devlog — v1.0.1 (2026-09-17)

**Title:** v1.0.1 — Theme Ride is back, and you can see the score

Gamestruments 1.0.1 is out. Two things landed:

- **Theme Ride** is back in the Suspense pool: the Decrypt cell and the Full
  Breach hook over a full 4/4 rock backbeat, on every bar. It is a pool phase
  like any other, so a `seeded` or `all-phases` take can include it.
- The Audio Lab has a **Score Debugger**: one step grid per bar for every voice,
  with hit counts and velocity, per-voice Solo/Mute, and Play bar. Point at a
  bar instead of describing a feeling.

If you already own the kit, grab the latest download — updates are included.

**Heads-up for Suspense users:** the pool now has 28 phases, so seeded and
all-phases Suspense takes are different songs than 1.0.0 produced. Racing and
Adventure are unchanged.

## Update Devlog — v1.0.1 storefront (2026-09-18)

Published: <https://gurisitosgames.itch.io/gamestruments-godot/devlog/1668875/see-it-in-action-walkthrough-video-and-this-updates-polish>

**Title:** See it in action — walkthrough video and this update's polish

See the Audio Lab in action — the walkthrough video is attached to this post
(watch it with sound, it's a free download). One continuous session: Suspense
into Anomaly, Racing into Position Fight, Adventure into the boss, with the
speed/intensity dial moving and the sound world switching from noir to neon to
orchestral. No cuts, no captions: you hear exactly what the engine does.

What's new and cool in this update:

- **The Score Debugger.** Point at a bar instead of describing a feeling: one
  step grid per bar for every voice, with hit counts and velocity, per-voice
  Solo and Mute, and a Play bar. You can finally see what the engine is doing
  under the hood.
- **Theme Ride is back.** Suspense gets its Decrypt cell and the Full Breach
  hook back, riding a full 4/4 rock backbeat on every bar.
- **Three recipes, ten sound worlds.** Racing, Suspense and Adventure, each
  with its own styles — switch the world and the same seed becomes another
  song: noir, neon, orchestral, chip, dark and more.

Try the Lab, watch the video — and if the music fits your game, the kit is
$12.99 with all future updates included.
