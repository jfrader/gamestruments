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

## Extracted Demo

1. Extract the verified archive to a clean directory.
2. Open only the extracted `kit/demo/` folder in Godot 4.7.2 or the release-packet version.
3. Confirm `GamestrumentsPlayer` loads without GDExtension errors.
4. Run the scene. A short intro card must state that the music adapts to
   gameplay; dismiss it and confirm audio begins from the generated garage
   section without requiring repository files.

Perform the following on every supported OS family:

- Play Night Circuit: start the countdown, use throttle/brake and grip-assisted
  steering, avoid the rival, spend/recharge boost, and complete three laps. No
  music section buttons may stand in for gameplay.
- Switch through all four circuits in the garage and confirm each style
  (neon, pocket funk, fusion, micro motor) regenerates and plays its own
  garage section before racing.
- Confirm off-road/contact penalties, finish order, pause/resume (including
  audio), focus-loss pause, and restart. Check both 960×620 and 1280×800 windows.
- Observe garage before starting, grid at countdown, cruise away from the
  rival, attack under pressure/boost, final lap on lap three, and the finish
  outro. The HUD is a requested-section indicator, not audible-bar telemetry.
- Change `music_style` in `race_music.gd` and reload for all four styles;
  confirm each is audible and materially distinct. Reload twice with the same
  `level_seed`, then change it and verify stable/different music respectively.
- In a fresh-project API integration, try supported voice overrides such as
  `pluck`, `organ`, `supersaw`, and `chip`.
- Confirm transitions wait for musical boundaries rather than cutting immediately.
- Let at least one section loop for 30 seconds and listen for clicks, silence, or discontinuity.
- Test Music and Master bus gain/mute behavior.
- Regenerate while playback is active in the fresh-project API integration.
- Close and reopen the scene twice; confirm clean shutdown without leak or crash output.

An unsupported voice must fail generation clearly instead of playing an invalid score.

## Claims Audit

Confirm directly from the extracted archive and demo:

- [ ] Product is described as adaptive racing music, not a general adaptive music engine.
- [ ] Godot 4.7.x and the three supported desktop platform families are explicit.
- [ ] `generate(seed) -> bool` and five-argument `set_race_state(...) -> bool` match runtime behavior.
- [ ] The six documented sections are reachable.
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
| Fresh extracted demo | | |
| Playable race, controls, win/loss, pause/restart, resize | | |
| Four styles and seed behavior | | |
| Voice validation | | |
| Six adaptive sections | | |
| Loop and bus behavior | | |
| Regeneration during playback | | |
| Clean repeated shutdown | | |
| Claims and archive contents | | |
| Exact-runtime listening quality | | |

Author QA passes only when every row passes on all advertised OS families. Next, run independent clean-room acceptance. The final itch transaction remains manual and requires single-use operator approval over the exact immutable release packet (`docs/release-packet-template.md`) and storefront listing (`storefront/listing.md`); drafting is not approval.
