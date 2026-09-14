# Gamestruments — Documentation

Gamestruments generates deterministic, sample-free adaptive music inside a
Godot 4 game. Install the addon, add one `GamestrumentsPlayer`, call
`generate(seed)` at level load, then drive changes through your recipe's
sections. The archive root `README.md` has the shortest first-play setup. The
player ships three recipes — Racing, Suspense, and Adventure — and each generated score
belongs to one recipe.

- [Quickstart](quickstart.md) — complete, copy-paste Racing and Suspense scripts.
- [API reference](api.md) — supported properties, methods, selection rules, and sections.
- [Limitations](limitations.md) — scope, platforms, audio, and determinism.
- [Troubleshooting](troubleshooting.md) — install, silence, and state issues.
- [Examples](../examples/README.md) — the three native reference scenes and how to run them.

Prefer to hear it first? The browser Audio Lab at
<https://gamestruments.gurisitos.games> is the same engine as the native addon
(parity-tested), including HTML5.

The Rust core is MIT licensed and its complete rebuildable source is included.
