# Gamestruments — Adaptive Racing Music for Godot 4

This archive contains the Gamestruments Godot addon, a standalone demo, complete
first-party Rust source, and the locked inputs needed to rebuild and test it.

## Start Here

1. Open `kit/demo/` as a Godot project to evaluate the packaged runtime.
2. For your game, copy `addons/gamestruments/` into
   `res://addons/gamestruments/`.
3. Follow `kit/docs/quickstart.md`, then use `kit/docs/api.md` as the supported
   API reference.

The runtime prefers an audio bus named `Music`. When that bus does not exist it
uses `Master`, so a fresh project can produce audio before custom bus setup.

## Archive Contents

- `addons/gamestruments/` — Linux x86_64, Windows x86_64, and universal macOS
  arm64/x86_64 libraries plus the GDExtension descriptor.
- `kit/demo/` — self-contained Godot demo with an identical addon copy.
- `kit/docs/` — quickstart, API, limitations, and troubleshooting documentation.
- `crates/`, `catalog/`, `Cargo.toml`, `Cargo.lock`, and
  `rust-toolchain.toml` — source, fixture, and pinned Rust rebuild inputs.
- `LICENSE.md`, `THIRD_PARTY_NOTICES.md`, and `licenses/` — first- and
  third-party license terms and attribution.
- `RELEASE-MANIFEST.json` — source identity, tool versions, workflow provenance,
  and SHA-256 hashes for each native library.
- `CHANGELOG.md` — release history.

## Rebuild and Test

From the archive root:

```sh
cargo test --workspace --locked
cargo build --workspace --all-targets --release --locked
```

Cargo downloads the exact checksummed dependencies in `Cargo.lock` unless they
are already cached or separately vendored.

## Runtime Use

Set a non-empty `project_secret`, select a style and optional voice overrides,
call `generate(seed)`, and check its boolean result. Drive the adaptive score
with `set_race_state(phase, intensity, pressure, final_lap, finish_result)`.
The runtime is sample-free and does not include the browser Audio Lab or its
authoring dependencies.

## License and Provenance

The Rust runtime, addon descriptor, buyer documentation, and demo integration
files are MIT licensed; see `LICENSE.md`. Third-party terms and attribution are
in `THIRD_PARTY_NOTICES.md` and `licenses/`.

Inspect `RELEASE-MANIFEST.json` before treating an archive as a release. Its
`provenance` and `source.dirty` fields distinguish tagged release builds from
pull-request, workflow-dispatch, and local candidates.

## Support

Report reproducible defects in the public comments section on the itch.io
product page. Include the kit version, exact Godot version, operating system
and architecture, minimal reproduction steps, and complete Godot Output text.
Use itch.io's purchase-support flow for purchase-specific or private matters.
