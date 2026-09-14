# Gamestruments — Godot Asset Store listing

This is the exact, repo-owned source for the Godot Asset Store submission.
Update this file before changing anything on https://store.godotengine.org.
Official docs:
https://docs.godotengine.org/en/stable/community/asset_store/submitting_to_asset_store.html

The store cannot take paid assets yet. This listing is a **free** discovery
funnel to the paid itch kit. Do not invent a store price.

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
- **Pricing:** Free. Optional donation / “get the full kit” link:
  https://gurisitosgames.itch.io/gamestruments-godot
- **Source code link:** the GitHub repo is **private**. Do not paste
  `https://github.com/jfrader/gamestruments` until Fran makes it public.
  Until then leave the field empty or point at the itch product page.
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
deterministic, sample-free music inside Godot. Choose a recipe and seed,
generate a score when your scene loads, then tell the player what's happening;
it blends between musical sections on bar boundaries. No sample library,
authoring tool, or cloud service is required.

**Try it in the browser:** https://gamestruments.gurisitos.games
(Audio Lab preview — browser audio layer; native mix can differ.)

**Full kit** (MIT source, docs, native examples):
https://gurisitosgames.itch.io/gamestruments-godot

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

The itch kit additionally includes full MIT Rust source, lockfile, pinned
toolchain, quickstart/API/limitations docs, and three native example scenes.

### Requirements

- Godot 4.7.x on Linux x86_64, Windows x86_64, or macOS arm64/x86_64.
- Native playback is fully offline. The optional browser preview needs a
  connection and uses a different Web Audio layer.
- First-party code is MIT; gdext is MPL-2.0.

## Media plan

Store Media tab uploads (do **not** rely on a `raw.githubusercontent.com` icon
link while the repo is private):

- **Icon / thumbnail:** capture from the packaged v0.1.2 Godot examples, not
  the browser Audio Lab. No icon is committed in git yet — there is no local
  Godot binary in this checkout to capture from.
- **Screenshots (3–5):** native example scenes from the extracted
  `gamestruments-0.1.2-godot4.zip` (`kit/examples/`). Never browser Lab
  captures as kit/store media.
- **Featured image:** same source as screenshots.
- **YouTube:** GURI-727 demonstration video when it exists. Do not submit
  without Fran-approved native audio footage.

Visual thesis (itch listing): show the actual music and the small Godot
integration that drives it. No separate showcase design.

## Packaging

Build the store ZIP from this repo (does not restructure `tools/package_kit.sh`):

```sh
tools/package-asset-store.sh --version 0.1.2 \
  --kit-zip /tmp/opencode/gamestruments-0.1.2-release/gamestruments-0.1.2-godot4.zip
```

Or pass unpacked native libraries:

```sh
tools/package-asset-store.sh --version 0.1.2 --assets-dir /path/to/libs
```

Output (default `/tmp/opencode`):

- `gamestruments-0.1.2-godot4-asset-store.zip`
- `gamestruments-0.1.2-godot4-asset-store.zip.sha256.txt`

ZIP layout (AssetLib drop-in):

```text
addons/gamestruments/README.md
addons/gamestruments/LICENSE.md
addons/gamestruments/gamestruments.gdextension
addons/gamestruments/bin/libgamestruments_godot.so
addons/gamestruments/bin/gamestruments_godot.dll
addons/gamestruments/bin/libgamestruments_godot.dylib
```

Version upload on the store: name `0.1.2`, changelog “Initial Asset Store
listing of the v0.1.2 addon.”, min Godot 4.7, file under 1 GB.

## Submission checklist

- [ ] Fran created the publisher account (name + slug above).
- [ ] Fran decided whether the GitHub repo stays private (source-link / icon).
- [ ] Asset type Addon, license MIT, Godot 4.7 minimum.
- [ ] Pricing left Free; itch kit linked as the paid full archive.
- [ ] Icon and screenshots uploaded via Media tab from packaged demo/examples.
- [ ] Store ZIP built with `tools/package-asset-store.sh` and checksum recorded.
- [ ] Submitted from Overview → submit for review.

## Related

- itch copy: `storefront/listing.md`
- Plugin-folder docs: `addons/gamestruments/README.md`, `LICENSE.md`
- Packager: `tools/package-asset-store.sh`
- Linear: GURI-746 (parent GURI-564)
