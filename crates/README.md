# In-game engine

Godot games set a per-title deterministic namespace and an instrument palette
on `GamestrumentsPlayer`, then call `generate(seed)` at level load.

Score identity uses generator version + namespace + seed + style + palette +
traits. The public property remains named `project_secret`, but it is embedded
in the game and is not a security credential.

TypeScript Studio is not used at runtime.

```bash
cargo test -p gamestruments-engine
cargo build -p gamestruments-godot
```

`godot` targets gdext 0.5.5 and Godot 4.7 with repository-scoped
`rust-toolchain.toml` pinning Rust 1.94.0. Release builds produce `.so`, `.dll`,
or `.dylib` files for the platform-specific paths in
`gamestruments.gdextension`.

`GamestrumentsPlayer` routes to a `Music` audio bus when one exists and falls
back to `Master` in a fresh project.

### Toolchain

The workspace root pins Rust 1.94.0 with rustfmt and clippy. Cargo commands
inside this checkout select it through rustup. The buyer archive includes the
workspace manifests, lockfile, toolchain pin, and both crates needed to rebuild.
