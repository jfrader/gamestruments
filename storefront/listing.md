# Gamestruments - itch.io Storefront Listing

This is the exact, repo-owned source for the Gamestruments itch.io listing.
Any updates to the storefront must be approved here first.
For official itch creator documentation, see: https://itch.io/docs/creators/

The next-update copy below is prepared locally, not published. Before approval,
check its recipe inventory, requirements, screenshots, and download contents
against the final music build. The dated launch devlog is historical.

## Metadata Fields

- **Title:** Gamestruments — Adaptive Music for Godot 4
- **Recommended Slug:** gamestruments-godot
- **Short Description:** Procedural music that adapts to your game, generated inside Godot.
- **Classification:** Game Assets
- **Kind:** Downloadable
- **Pricing:** $12.99 minimum / Pay-what-you-want above (No launch discount)
- **Language:** English
- **Tags:** godot, godot-4, music, adaptive-music, dynamic-music, procedural, racing, suspense, adventure, soundtrack, audio, engine
- **Release Status:** Released
- **Platforms:** No OS executable flags (the zip contains libraries/source, not a standalone OS executable).
- **Community:** Comments enabled for public support.
- **External Links:** Live demo at https://gurisitosgames.itch.io/gamestruments-audio-lab-demo and standalone browser preview at https://gamestruments.gurisitos.games (Audio Lab: same generator, browser audio layer). The source repository is private.

## Visual Thesis and Media Capture Checklist

**Visual Thesis:** Use the existing Audio Lab's visual identity. Show the actual music and controls, then the small Godot integration that drives them. No separate showcase design.

**Content Plan:**
1. Hear the existing lab and understand the product
2. See how game state changes the music
3. See the included examples and integration steps
4. Check requirements and limitations
5. Purchase the kit

**Media Capture Checklist:**
- [ ] Build the itch HTML preview from the same approved lab source, with its actual phases and controls unchanged.
- [ ] Clearly identify the interactive browser experience as **Audio Lab preview**; do not present it as exact native playback or as a bundled Godot interface.
- [ ] Capture native integration screenshots and native-audio evidence from the final extracted Godot kit, not the development checkout.
- [ ] Cover image follows the 315:250 ratio (630x500 recommended).
- [ ] Show one readable integration script and the native example it runs. Keep API calls legible rather than filling the page with code screenshots.
- [ ] Include a short transition video recorded from the final native kit; it is required before promoting this update.
- [ ] Fran has approved the music excerpts and finished video.

## Long Description

### Gamestruments — Adaptive Music for Godot 4

**Procedural music that adapts to your game.** Gamestruments generates
deterministic, sample-free music inside Godot. Choose a recipe and seed,
generate a score when your scene loads, then tell the player what's happening;
it blends between musical sections on bar boundaries. No sample library,
authoring tool, or cloud service is required.

**Try the demo first:** [open the interactive Audio Lab](https://gurisitosgames.itch.io/gamestruments-audio-lab-demo)
or [use the standalone browser preview](https://gamestruments.gurisitos.games).

### What you get

- A prebuilt Godot 4 addon for Linux x86_64, Windows x86_64, and macOS
  arm64/x86_64, with the GDExtension descriptor.
- The full MIT-licensed Rust source, lockfile, and pinned toolchain.
- Complete docs: quickstart, API reference, limitations, and troubleshooting.
- Three native example scenes (playback, game signals, song form) with their own
  addon copy. No browser or network needed.

Use it for title music, rising tension, and action, then connect the music to
your own game events.

### Getting started

1. Copy `addons/gamestruments/` into your project and restart Godot.
2. Add a `GamestrumentsPlayer` child, set the recipe, style, and title
   namespace, then call `generate(seed)` and check the result.
3. Connect your game events to `set_race_state(...)`, `set_trace_state(...)`,
   or `set_adventure_state(...)` for the chosen recipe. Form controls
   (`cue_section` etc.) apply when a form is attached (Suspense, or Racing/Adventure
   with `autoplay`).

The complete public docs and one free example script ship in a separate free
download on this page: `gamestruments-docs-and-example.zip`.

### Requirements

- Godot 4.7.x on Linux x86_64, Windows x86_64, or macOS arm64/x86_64.
- Native playback is fully offline. The optional browser preview needs a
  connection and uses a different Web Audio layer, so its mix can differ from
  the native addon.
- First-party code is MIT; gdext is MPL-2.0.

All future updates to this kit are included with your purchase. Questions or
bugs? Post in the comments.

## Public docs download (prepared, not uploaded)

- A free per-file demo download, `gamestruments-docs-and-example.zip`, is
  prepared for the same paid product page. The paid price is unchanged.
- It contains the complete buyer docs plus one `example.gd` extracted from the
  canonical quickstart. Docs and source only: the example needs the paid addon
  to run, and no native binaries or runnable project ship in it.
- Downloaded Markdown, not a browser-native doc viewer.
- Build with `npm run build:public-docs`.
- The source repository stays private.
- The free docs are readable before purchase, and the free ZIP is the
  low-maintenance primary preparation. Publication still requires approval.
- Configure the per-file demo download with itch.io's demo checkbox in the
  product edit form (official reference:
  <https://itch.io/docs/creators/getting-started>).
- Put download links in the description rather than hiding them in the External
  Links field; the download page is where the file is attached.
- Keep demo-to-kit navigation same-tab (user preference).
- Confirm the platform supports this behavior before publishing.

## Video sequence (draft)

Target 35–45 seconds. Music and captions only; no voiceover or sales copy. Code
excerpts come from the tested canonical public example, using its existing API
and phase names. Native-audio evidence must be captured from the final packaged
native kit. Final recording awaits music approval; the existing Audio Lab stays
as the interactive preview, unchanged.

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

## Launch Devlog (2026-09-11)

**Title:** Now live: music your game writes as it plays

Gamestruments is out! It's an adaptive music engine for Godot 4 — your game generates its own soundtrack at runtime, with no audio files, no authoring tool, and no external services.

The release ships four recipes:

- **Racing** — six sections from the garage to the finish line, driven by speed, rival pressure, and lap state.
- **Suspense** — song-form tension for infiltration, hacking, and horror, with a form your gameplay can hold or advance.
- **Adventure** — an eight-section fantasy quest arc (explore, town, dungeon, combat, boss, sanctuary, victory), four of them longer arrangements, driven by discovery and threat.

It runs on Linux, Windows, and macOS, includes the full MIT Rust source, and comes with a playable four-circuit demo so you can hear it before writing any code.

Try it in your browser: https://gamestruments.gurisitos.games
Get the kit: https://gurisitosgames.itch.io/gamestruments-godot

All future updates are included. Building something with it? I'd love to see it — questions and bug reports are welcome in the comments.
