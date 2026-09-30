# Limitations

## Product Scope

- This release ships four fixed state models: **Racing** (racing: garage, grid, cruise, attack, final lap, victory; extended adds four more), **Suspense** (song-form: a 27-phase pool, rendered `all-phases` or `seeded`), and **Adventure** (fantasy quest: camp, explore, town, dungeon, combat, boss, sanctuary, victory — 16-bar camp/dungeon/boss/sanctuary and 32-bar explore/town/combat/victory), and **Cozy** (a village day: dawn, morning, market, noon, rain, evening, festival, night). All use the same `GamestrumentsPlayer`.
- It is not a general-purpose music graph, editor plugin, DAW, pattern editor, or complete game. `kit/examples/` contains integration reference scenes, not a game or a game template.
- Racing ships four sound styles: fusion, neon, funk, and chip. Suspense ships terminal, cipher, and noir. Adventure ships folk, dark, and orchestral. Cozy ships acoustic, lofi, and bossa. Voice overrides apply to Racing; arbitrary samples and plugins are unsupported. Adventure's acoustic timbres (harp, recorder, vielle, bell, horn, timpani) are synthesized — acoustic-inspired, not sample recordings.

## Audio

- The runtime synth renders mono at 48000 Hz and sends identical left/right frames through Godot. This favors a small, deterministic runtime over sample-library fidelity.
- HTML5 games use the WASM engine. This addon ships no web build: no web library, no `web.*` entry in the GDExtension. Godot supports GDExtension in web exports, but only with custom export templates built with `dlink_enabled=yes`, which this kit does not provide. Preview: <https://gamestruments.gurisitos.games>.
- No WAV, OGG, MP3, MIDI, stem export, sample import, mastering, effects rack, spatial audio, or middleware bridge ships.

## Runtime and Platforms

- Supported: Godot 4.7.x (tested on 4.7.2), Linux x86_64 (built on Ubuntu 24.04), Windows x86_64, and macOS arm64/x86_64.
- Unsupported: Godot versions other than 4.7.x, this addon's web target, mobile, and consoles.
- The macOS universal library is ad-hoc signed for loading but is not Developer ID-signed or notarized. The Linux and Windows libraries are not publisher-signed. Operating-system quarantine or application-signing rules may apply when you redistribute the libraries as part of your own game.
- The node prefers an audible `Music` bus and otherwise uses `Master`. State requests require a successful `generate(seed)` first.
- State transitions wait for bar boundaries; instant cuts are not supported by the public API.

## Determinism and Persistence

- Determinism is scoped to the exact generator version and input tuple. Updating the kit may intentionally change output.
- `project_secret` is a deterministic per-title namespace, not a hidden credential or security boundary.
- Runtime score serialization, save-game migration between generator versions, and a public event-editing API are not supported.
- Exact floating-point audio bytes are not promised across operating systems or Godot patch releases; score identity and event data under one generator version are the stable contract.

## Integration and Support

- Only `GamestrumentsPlayer` and the documented exported properties and methods are supported. Internal Rust modules and generated child nodes may change.
- This kit ships the Godot 4 binding only. The Rust engine is a separate crate, but no other engine binding or custom-adapter framework is supported yet.
- Source is included under MIT, but custom builds, modified APIs, and unadvertised targets are outside standard support.
- Support is best-effort through reproducible reports in the itch.io product page's public comments; no response-time or long-term update SLA is promised. Purchase-specific or private matters use itch.io's purchase-support flow.
