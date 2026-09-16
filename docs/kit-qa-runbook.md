# Gamestruments Kit — Author QA Runbook

Use this only with an immutable candidate produced by the release workflow. Do not reuse the retired `0.1.0-rc1` archive or its checksum.

Independent clean-room acceptance in `docs/kit-plan.md` remains a separate required gate.

## Release Packet Header

Record before testing:

```text
Version:
Source commit and tag:
Workflow run URL:
Archive filename:
Archive byte size:
Archive SHA-256:
Godot version:
Tester:
Date:
Operating system and architecture:
```

Verify the downloaded digest before extraction using `sha256sum` on Linux, `shasum -a 256` on macOS, or `Get-FileHash -Algorithm SHA256` on Windows.

## Automated Evidence

The release workflow must be green and show all of these before author QA:

- Linux x86_64 build and native Godot runtime smoke.
- Windows x86_64 build and native Godot runtime smoke.
- macOS arm64/x86_64 universal build, architecture check, and native Godot runtime smoke.
- Separate native Godot runtime smoke on macOS arm64 and macOS Intel x86_64 runners.
- Repeated generate/play/transition/free cycles without error, crash, or leaked-object output.
- One cross-platform archive containing all native libraries.
- Extracted-source rebuild with Rust 1.94.0 and the committed lockfile.
- Archive contents and zero-audio-assets checks.
- Rust, WASM, TypeScript, and browser control tests.

Any missing, skipped, or failing job blocks the candidate.

## Extracted Examples

1. Extract the verified archive to a clean directory.
2. Open only the extracted `kit/examples/project.godot` in Godot 4.7.2 or the release-packet version.
3. Confirm `GamestrumentsPlayer` loads without GDExtension errors.
4. Press F5. The default `01-playback` scene must generate and play without repository files.

Run each scene fresh (close and reopen it) and check:

- **01 Playback:** a non-empty `project_secret`, `recipe = "suspense"`,
  `style = "terminal"`, `arrangement = "seeded"`, and a successful `generate()`.
  Music plays; restart replays from the first section; the section readout updates.
- **02 Game signals:** the racing scene generates, then the controls produce
  `set_race_state` requests for grid, cruise, attack, final lap, and victory.
  Confirm requested vs currently playing are reported separately and that changes
  wait for bar boundaries.
- **03 Song form:** the suspense scene generates with `arrangement = "all-phases"`,
  `set_trace_state` drives sections, `cue_section("chorus")` is accepted, and
  `set_form_hold`, `advance_form`, and `is_form_held` behave as documented.
  Holding the form must not stop sound.

Then, on every supported OS family:

- Confirm each scene stops and frees cleanly with no leak, crash, or error output when closed, and reopen it twice.
- Remove or rename the example addon copy, reopen a scene, and confirm the
  in-scene error explains the missing addon instead of crashing the editor.
  Restore the addon afterward.
- Confirm the browser links in the docs and listing open the Audio Lab showcase,
  and that the examples themselves need no browser or network.
- Confirm the archive contains the addon, the self-contained examples project,
  source, docs, licenses, and manifest, with no audio assets or authoring packages.
- In a fresh-project API integration, try supported voice overrides such as
  `pluck`, `organ`, `supersaw`, and `chip`.
- In the fresh-project API integration, generate the Suspense recipe
  (`recipe = "suspense"`, styles terminal/cipher/noir, arrangements all-phases and
  seeded), drive `set_trace_state` through boot/scan/exploit/alert/extract/
  complete, and confirm sections change on bar boundaries without errors.
- Confirm Suspense form controls: `set_form_hold`, `advance_form`,
  `is_form_held`, `cue_section`, and `get_current_section` behave as documented;
  Racing/Adventure return `false` for form controls unless `autoplay=true`.
- Confirm an empty Suspense style defaults to terminal and an unknown style
  fails with a clear error.
- Confirm Adventure: 3 styles (folk/dark/orchestral default folk), `set_adventure_state`
  with 8 phases, discovery/threat/quest_complete resolve correctly (e.g. high discovery -> sanctuary, quest_complete -> victory); form controls false for Adventure without autoplay.
- Confirm transitions wait for musical boundaries rather than cutting immediately.
- Let at least one section loop for 30 seconds and listen for clicks, silence, or discontinuity.
- Test Music and Master bus gain/mute behavior.
- Regenerate while playback is active in the fresh-project API integration.
- Close and reopen the scene twice; confirm clean shutdown without leak or crash output.

An unsupported voice must fail generation clearly instead of playing an invalid score.

## Automated Examples Checks

The default `tests/godot-package-smoke.mjs` run checks native playback headlessly
with an explicit extension list. It also extracts the actual README and
quickstart GDScript blocks into a separate bare project, then runs their setup
and gameplay callbacks against the native addon. This is automated integration
coverage, not independent human buyer acceptance.

Use `--screenshots <directory>` on a machine
with a display to additionally require a clean rendered editor import and
capture all three examples, including narrow layouts and missing-addon states.
Godot 4.7.2 headless **editor** startup crashes with the extension in this test
environment; headless runtime playback is a separate, passing check.

On NVIDIA 610.57.04, repeated OpenGL viewport screenshot readbacks can stall in
the driver even in a plain UI project without the addon. On Linux with Mesa
installed, use `__GLX_VENDOR_LIBRARY_NAME=mesa LIBGL_ALWAYS_SOFTWARE=1` for this
rendered QA command and record that the captures use software rendering.
Mesa's unsupported V-Sync warning is expected; engine errors and resource leaks
still fail the harness. This does not replace native-platform listening checks.

## Claims Audit


Confirm directly from the extracted archive and examples:

- [ ] Product is described as Gamestruments with named recipes (Racing, Suspense, Adventure), not as a racing-only engine.
- [ ] `kit/examples/` is described as three independent integration references (PlaybackSeededPool, GameSignalsOriginalRacing, SongFormAllPhases), not a playable game or a four-circuit race series.
- [ ] Godot 4.7.x and the three supported desktop platform families are explicit.
- [ ] `generate(seed) -> bool`, `set_race_state(...) -> bool`, `set_trace_state(...) -> bool`, `set_adventure_state(...) -> bool`, and the form methods match runtime behavior.
- [ ] The six Racing sections (original) and the 27-phase Suspense pool are reachable via `all-phases` and `seeded`. Adventure's eight sections are reachable via `set_adventure_state`.
- [ ] No WAV, OGG, MP3, Strudel, browser Lab, or TypeScript authoring package is present.
- [ ] The exact-runtime sound is accurately represented by proposed storefront media.
- [ ] Complete Rust rebuild inputs, changelog, licenses, and third-party notices are present.
- [ ] `project_secret` is described as a deterministic namespace, not protected secrecy.
- [ ] Limitations do not contradict observed behavior.

## Result

| Check | Pass/fail | Evidence or observation |
|---|---|---|
| Digest and source identity | | |
| Native workflow jobs | | |
| Fresh extracted examples | | |
| 01 playback generation and restart | | |
| 02 game-signal requests | | |
| 03 form holds and cues | | |
| Missing-addon error | | |
| Four styles and seed behavior | | |
| Voice validation | | |
| Racing/Adventure/Suspense adaptive sections | | |
| Adventure recipe and set_adventure_state (8 phases, 3 styles) | | |
| Suspense recipe, trace states, and form controls | | |
| Loop and bus behavior | | |
| Regeneration during playback | | |
| Clean repeated shutdown | | |
| Browser links and offline examples | | |
| Archive completeness | | |
| Claims and archive contents | | |
| Exact-runtime listening quality | | |

Author QA passes only when every row passes on all advertised OS families. Next, run independent clean-room acceptance. The final itch transaction remains manual and requires single-use operator approval over the exact immutable release packet (`docs/release-packet-template.md`) and storefront listing (`storefront/listing.md`); drafting is not approval.
