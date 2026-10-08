# Gamestruments — itch.io listing

Repo-owned source for the itch.io page
(<https://gurisitosgames.itch.io/gamestruments-godot>). Change this file first,
then the page. Creator docs: <https://itch.io/docs/creators/>.

Earlier listing copy and devlogs are in git history.

## Metadata

- **Title:** Gamestruments — Adaptive Music for Godot 4
- **Slug:** gamestruments-godot
- **Short description:** Free, open source procedural music that adapts to your game, in Godot and HTML5.
- **Classification:** Game Assets
- **Kind:** Downloadable
- **Pricing:** No payments required. Donations enabled ("Name your own price", $0 minimum, suggested donation optional).
- **Tags:** adaptive-music, audio, dynamic-music, godot, godot-4, music, procedural, open-source, soundtrack, adventure
- **Platforms:** no OS executable flags (the zip holds libraries and source).
- **Community:** comments enabled; bug reports point to GitHub issues.
- **External links:** source <https://github.com/jfrader/gamestruments>,
  browser preview <https://gamestruments.gurisitos.games>, live Lab demo
  <https://gurisitosgames.itch.io/gamestruments-audio-lab-demo>.

## Downloads

- `gamestruments-<version>-godot4.zip`, the kit archive from the GitHub release
  for that tag (built by `.github/workflows/release.yml`), type **Source code**.
- `gamestruments-lab-itch.zip`, the HTML Audio Lab preview, built with
  `npm run build:itch`; browser play, 960x600, mobile friendly, fullscreen.
- Existing walkthrough and sample videos stay as free media.

## Long description

### Gamestruments — Adaptive Music for Godot 4

**Free and open source procedural music that adapts to your game.**
Gamestruments generates deterministic, sample-free music in Godot and in HTML5.
Choose a recipe and seed, generate a score when your scene loads, then tell the
player what's happening; it blends between musical sections on bar boundaries.
No sample library, authoring tool, or cloud service is required.

**Try it first:** [open the interactive Audio Lab](https://gurisitosgames.itch.io/gamestruments-audio-lab-demo)
or [the standalone browser preview](https://gamestruments.gurisitos.games).

### Recipes

- **Racing** — pace, rival pressure, lap state, and the finish.
- **Suspense** — song-form tension for infiltration, hacking, and horror.
- **Adventure** — an eight-section fantasy quest arc.

### What you get

- A prebuilt Godot 4 addon for Linux x86_64, Windows x86_64, and macOS
  arm64/x86_64.
- Mastered output around −14 LUFS with true peak below −1 dBTP, 48 kHz mono.
- The full MIT-licensed Rust source, lockfile, and pinned toolchain.
- Quickstart, API reference, limitations, and troubleshooting docs.
- Three native example scenes (playback, game signals, song form).

### Getting started

1. Copy `addons/gamestruments/` into your project and restart Godot.
2. Add a `GamestrumentsPlayer` node, set the recipe, style, and title
   namespace, then call `generate(seed)` and check the result.
3. Connect your game events to `set_race_state(...)`, `set_trace_state(...)`,
   or `set_adventure_state(...)`.

### Requirements

- Godot 4.7.x on Linux x86_64, Windows x86_64, or macOS arm64/x86_64.
- Native playback is fully offline.
- Everything is MIT (use it in commercial games too); godot-rust is MPL-2.0.

### Free, with optional donations

Gamestruments is free and open source:
<https://github.com/jfrader/gamestruments>. Contributions and bug reports are
welcome there. If it helps your game, a donation here keeps development going.
