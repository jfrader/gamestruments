# Limitations

## Product Scope

- This release is an adaptive **racing** music generator. Its fixed state model is garage, grid, cruise, attack, final lap, and victory.
- It is not a general-purpose music graph, editor plugin, DAW, pattern editor, or complete game.
- Four sound styles ship: fusion, neon, funk, and chip. Voice overrides select from the built-in synthesizer vocabulary; arbitrary samples and plugins are unsupported.

## Audio

- The runtime synth renders mono at 22050 Hz and sends identical left/right frames through Godot. This favors a small, deterministic runtime over sample-library fidelity.
- The browser Audio Lab uses a separate stereo Web Audio presentation layer. It can demonstrate composition and controls, but not the exact buyer sound.
- No WAV, OGG, MP3, MIDI, stem export, sample import, mastering, effects rack, spatial audio, or middleware bridge ships.

## Runtime and Platforms

- Supported: Godot 4.7+, Linux x86_64, Windows x86_64, and macOS arm64/x86_64.
- Unsupported: Godot 3, Godot versions older than 4.7, web export, mobile, and consoles.
- Native libraries are unsigned. Operating-system quarantine or application-signing rules may apply when you redistribute them as part of your own game.
- The node requires an audible `Music` bus and successful `generate(seed)` before state requests.
- State transitions wait for bar boundaries; instant cuts are not supported by the public API.

## Determinism and Persistence

- Determinism is scoped to the exact generator version and input tuple. Updating the kit may intentionally change output.
- `project_secret` is a deterministic per-title namespace, not a hidden credential or security boundary.
- Runtime score serialization, save-game migration between generator versions, and a public event-editing API are not supported.
- Exact floating-point audio bytes are not promised across operating systems or Godot patch releases; score identity and event data under one generator version are the stable contract.

## Integration and Support

- Only `GamestrumentsPlayer` and the documented exported properties and methods are supported. Internal Rust modules and generated child nodes may change.
- Source is included under MIT, but custom builds, modified APIs, and unadvertised targets are outside standard support.
- Support is best-effort through reproducible GitHub issues; no response-time or long-term update SLA is promised.
