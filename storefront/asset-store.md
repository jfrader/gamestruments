# Gamestruments — Godot Asset Store listing

This is the exact, repo-owned source for the Godot Asset Store submission.
Update this file before changing anything on https://store.godotengine.org.
Official docs:
https://docs.godotengine.org/en/stable/community/asset_store/submitting_to_asset_store.html

Gamestruments is free and open source (MIT). The store listing is free too.

## Publisher (fill at account creation)

Fran creates the publisher account. Recommended values — confirm availability
at registration, do not invent a second publisher:

- **Publisher Name:** Gurisitos Games
- **Publisher URL Slug:** gurisitosgames

## Asset metadata

- **Asset Name:** Gamestruments — Adaptive Music for Godot 4
- **Asset URL slug:** gamestruments
- **Asset type:** Addon
- **License:** MIT (must match the plugin-folder `LICENSE.md`)
- **Minimum Godot version:** 4.7
- **Maximum Godot version:** leave open / 4.x
- **Pricing:** Free. Optional donation link:
  https://gurisitosgames.itch.io/gamestruments-godot
- **Source code link:** https://github.com/jfrader/gamestruments
- **AI usage disclosure:** no AI-generated content in the addon binaries,
  descriptor, or plugin-folder docs.

## Tags

godot, godot-4, music, adaptive-music, dynamic-music, procedural, audio,
engine, racing, adventure

## Short summary

Procedural music that adapts to your game, generated inside Godot.

## Long description

### Gamestruments — Adaptive Music for Godot 4

Procedural music that adapts to your game. Gamestruments generates
deterministic, sample-free music in Godot and in HTML5. Choose a recipe and seed,
generate a score when your scene loads, then tell the player what's happening;
it blends between musical sections on bar boundaries. No sample library,
authoring tool, or cloud service is required.

**Try it in the browser:** https://gamestruments.gurisitos.games
(Godot and HTML5, one engine.)

**Source, docs and native examples** (free, MIT):
https://github.com/jfrader/gamestruments

### Recipes

- **Racing** — pace, rival pressure, lap state, and the finish.
- **Suspense** — song-form tension for infiltration, hacking, and horror.
- **Adventure** — an eight-section fantasy quest arc.

### Install

1. Copy `addons/gamestruments/` into your Godot 4.7.x project and restart.
2. Add a `GamestrumentsPlayer` child, set the recipe, style, and title
   namespace, then call `generate(seed)` and check the result.
3. Connect your game events to `set_race_state(...)`,
   `set_trace_state(...)`, or `set_adventure_state(...)`.

Copy-paste GDScript is in this addon's `README.md`.

### What this Asset Store download is

The Linux x86_64, Windows x86_64, and universal macOS arm64/x86_64 libraries
plus the GDExtension descriptor, with a README and MIT license inside the
plugin folder. It is the drop-in addon.

The full kit archive on GitHub releases and itch.io adds the MIT Rust source,
lockfile, pinned toolchain, docs, and three native example scenes.

### Requirements

- Godot 4.7.x on Linux x86_64, Windows x86_64, or macOS arm64/x86_64.
- Also runs in HTML5. Native playback is fully offline. Hear it in the
  browser before you install.
- First-party code is MIT; gdext is MPL-2.0.

## Media plan

Captured 2026-09-14 from the extracted v0.1.2 kit (`kit/examples/`, Godot
4.7.2, native addon). Never browser Audio Lab captures. Upload these files
through the Store Media tab.

- **Icon / thumbnail:** `addons/gamestruments/icon.png` (256×256, square pad
  of the 01 Playback example).
- **Screenshots:**
  - `storefront/asset-store-media/01-playback.png` — generate / play / restart
  - `storefront/asset-store-media/02-game-signals.png` — `set_race_state`
  - `storefront/asset-store-media/03-song-form.png` — hold / cue / advance
- **Featured image:** `01-playback.png` unless a later native video still exists.
- **YouTube:** GURI-727 demonstration video when it exists. Do not promote
  without Fran-approved native audio footage.

Visual thesis (itch listing): show the actual music and the small Godot
integration that drives it. No separate showcase design.

## Packaging

Build the store ZIP from this repo (does not restructure `tools/package_kit.sh`):

```sh
tools/package-asset-store.sh --version 0.1.2 \
  --kit-zip dist/gamestruments-0.1.2-godot4.zip
```

Or pass unpacked native libraries:

```sh
tools/package-asset-store.sh --version 0.1.2 --assets-dir /path/to/libs
```

Output (default `dist/`):

- `gamestruments-0.1.2-godot4-asset-store.zip`
- `gamestruments-0.1.2-godot4-asset-store.zip.sha256.txt`

ZIP layout (AssetLib drop-in):

```text
addons/gamestruments/README.md
addons/gamestruments/LICENSE.md
addons/gamestruments/icon.png
addons/gamestruments/gamestruments.gdextension
addons/gamestruments/bin/libgamestruments_godot.so
addons/gamestruments/bin/gamestruments_godot.dll
addons/gamestruments/bin/libgamestruments_godot.dylib
```

Version upload on the store: name `0.1.2`, changelog “Initial Asset Store
listing of the v0.1.2 addon.”, min Godot 4.7, file under 1 GB.

## Submission checklist

- [ ] Fran created the publisher account (name + slug above).
- [ ] Source link set to the public GitHub repository.
- [ ] Asset type Addon, license MIT, Godot 4.7 minimum.
- [ ] Pricing left Free; itch page linked for optional donations.
- [x] Icon and screenshots captured from packaged `kit/examples/` (files in repo).
- [ ] Icon and screenshots uploaded via the Store Media tab.
- [ ] Store ZIP built with `tools/package-asset-store.sh` and checksum recorded.
- [ ] Submitted from Overview → submit for review.

## Related

- itch copy: `storefront/listing.md`
- Plugin-folder docs: `addons/gamestruments/README.md`, `LICENSE.md`
- Packager: `tools/package-asset-store.sh`
- Linear: GURI-746 (parent GURI-564)
