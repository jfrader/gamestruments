# Third-Party Notices

The Gamestruments Godot native libraries are built from the exact dependency versions and checksums recorded in `Cargo.lock`.

## Mozilla Public License 2.0

The following godot-rust packages are MPL-2.0:

- `godot`, `godot-bindings`, `godot-cell`, `godot-codegen`, `godot-core`, `godot-ffi`, and `godot-macros` 0.5.5
- `gdextension-api` 0.5.1

Origin: <https://github.com/godot-rust/gdext> and the corresponding packages on <https://crates.io/>.

The unmodified source code form for these exact packages is available from crates.io using the package names and versions above, or by running `cargo vendor --locked` from the included workspace. Gamestruments does not modify MPL-covered source files. The full license is included at `licenses/MPL-2.0.txt`.

## MIT or Apache License 2.0

The following packages offer MIT or Apache-2.0 terms:

- `serde`, `serde_core`, and `serde_derive` 1.0.229
- `serde_json` 1.0.151
- `proc-macro2` 1.0.107
- `quote` 1.0.47
- `syn` 3.0.4
- `itoa` 1.0.18
- `glam` 0.32.1
- `libc` 0.2.189
- `heck` 0.5.0
- `nanoserde` 0.2.1

The full common license texts are included at `licenses/MIT.txt` and `licenses/Apache-2.0.txt`. Package authorship and source links are recorded in the packages' crates.io metadata and `Cargo.lock`.

## MIT

- `nanoserde-derive` 0.2.1
- `venial` 0.6.1
- `zmij` 1.0.23

The full license is included at `licenses/MIT.txt`.

## MIT or Unlicense

- `memchr` 2.8.3

The full alternatives are included at `licenses/MIT.txt` and `licenses/Unlicense.txt`.

## Unicode License v3

`unicode-ident` 1.0.24 is offered under MIT or Apache-2.0 and also incorporates Unicode-3.0-covered data. The applicable texts are included at `licenses/MIT.txt`, `licenses/Apache-2.0.txt`, and `licenses/Unicode-3.0.txt`.

No TypeScript, Strudel, or browser authoring dependency is linked into or included with the buyer runtime kit.
