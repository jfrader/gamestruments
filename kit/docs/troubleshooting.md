# Troubleshooting

## Extension Not Loaded / "GamestrumentsPlayer" Does Not Appear

Symptoms: Node search does not find `GamestrumentsPlayer`; `class` errors at runtime; `.gdextension` shows red in FileSystem.

Checks (in order):

1. The `.gdextension` file must be directly under `res://addons/gamestruments/gamestruments.gdextension`.
2. The binary must be at the path declared inside it:
   ```
   res://addons/gamestruments/bin/libgamestruments_godot.so
   ```
   (Only the Linux entry is present in v1.)
3. Restart the Godot editor after copying the files (or use "Project → Reload Current Project").
4. Check the Output panel for gdextension load errors (symbol, architecture mismatch, missing dependencies).
5. Confirm you are on a supported platform (Linux x86_64 for the binary). On other OSes you must rebuild from the included source.

If the paths are correct but it still fails, delete the `.godot` import cache folder and re-open the project.

## Silence / No Sound After generate()

1. Confirm you called `player.generate("some-seed")` and that `project_secret` was non-empty at the time of the call (the node errors via `godot_error` if empty).
2. The player creates an `AudioStreamPlayer` child that routes to the **"Music"** bus. Verify the bus exists and is not muted (Project Settings → Audio → Buses). If you have no "Music" bus, create one or temporarily route via Master for testing.
3. Run the scene from the editor (not just "play current scene" in some contexts) so the `_ready` path executes.
4. Check the Audio bus layout volume and that no other bus is soloed.
5. After `generate`, the transport starts in the "garage" section. If your first `set_race_state` happens before any audio frames, you may hear a brief delay until the next bar.
6. Use the debugger's Audio tab or add a temporary `AudioStreamPlayer` playing a test tone on the same bus to isolate whether the problem is the extension or Godot audio output.

## Volume Too Low or "Thin" Sound

- This is a mono 22050 Hz synth by design (see `limitations.md`). It will sound different from sample-based or high-rate music.
- Raise the Music bus gain in the editor for testing.
- The traits (`energy`, `brightness`, etc.) and style choice affect perceived density and timbre. Try `style = "fusion"` + higher `energy` for a fuller starting point.
- The internal velocity and gating are intentional; they are not bugs.

## State Changes Do Not Seem to Do Anything

- `set_race_state` only requests a change. The actual crossover happens on the next bar boundary.
- You must call `generate(...)` successfully first; otherwise the internal transport is `None` and calls are no-ops (see source).
- Phase strings are matched against the rule set. Common working values: `"garage"`, `"grid"`, `"race"`, `"attack"`, `"final-lap"`, `"victory"`.
- Listen over a few bars; crossfades are musical (two-bar default) rather than instant.

## Different Results Than Expected / "Not the Same as Catalog"

- Determinism requires the exact same `(secret, seed, style, palette, traits)` tuple and the generator version that produced the reference.
- The reserved catalog take uses an empty secret + specific traits (see engine tests). Using a non-empty secret or different floats will produce a different id and events.
- Rebuilds from source must use the exact toolchain pinned at tag time for bit parity with the distributed binary.

## Godot Errors About Leaks or "2 instances" at Quit

- The shipped implementation already contains mitigations (transient `Gd` handles only, explicit stop + null stream + child free in `exit_tree` + `PREDELETE`).
- Any remaining ObjectDB counts at quit that are unrelated to `GamestrumentsPlayer` are outside this kit's control. The player itself should not contribute after the fixes.

## Rebuilding from Source (Advanced)

- The full `crates/engine` and `crates/godot` trees + `Cargo.toml` files are in the archive.
- Use the Rust toolchain and gdext version that matched the release build for binary compatibility.
- After a successful `cargo build --release -p gamestruments-godot`, place the resulting cdylib at the path declared in `gamestruments.gdextension`.
- Platform-specific notes and the release workflow live in the repository at the release tag (not reproduced here).

If none of the above resolve the issue, open a GitHub issue with:
- Godot version + OS
- Exact steps from a fresh project (or a minimal reproduction scene)
- The exact `project_secret` / `style` / seed / trait values used
- Any error messages from the Output or Debugger tabs

See `api.md` and `kit-contract.md` for the supported surface before reporting.
