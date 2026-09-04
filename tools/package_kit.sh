#!/usr/bin/env bash
# tools/package_kit.sh
# Package the Gamestruments buyer release archive (Godot 4 kit).
#
# Usage:
#   tools/package_kit.sh [--version <ver>] [--out-dir <dir>]
#
# - VERSION defaults to 0.1.0-rc1 (or first arg, or git describe)
# - Produces gamestruments-$VERSION-godot4.zip in OUT_DIR (default /tmp/opencode)
# - Builds from the current checkout (pinned via rust-toolchain.toml)
# - Uses explicit allowlist for reproducible buyer artifact.
# - Zip is made deterministic: sorted file list + zip -X (no extra fields) +
#   normalized mtimes in staging. Residual non-determinism: the .so binary
#   may contain linker timestamps / build IDs / UUIDs from cargo/rustc even in
#   release; we do not strip the binary itself (honest). Source files, text,
#   and zip metadata are normalized.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

VERSION="0.1.0-rc1"
OUT_DIR="/tmp/opencode"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version|-v)
      VERSION="$2"; shift 2 ;;
    --out-dir|--out)
      OUT_DIR="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: $0 [--version X.Y.Z-rcN] [--out-dir DIR]"; exit 0 ;;
    *)
      # positional fallback for version
      if [[ "$VERSION" == "0.1.0-rc1" && "$1" != --* ]]; then
        VERSION="$1"; shift
      else
        echo "Unknown arg: $1" >&2; exit 1
      fi
      ;;
  esac
done

echo "==> Packaging Gamestruments kit version: $VERSION"
echo "==> Repo: $REPO_ROOT (commit: $(git rev-parse --short HEAD))"
echo "==> Output dir: $OUT_DIR"

mkdir -p "$OUT_DIR"

# 1. Build the release binary (uses rust-toolchain.toml pin for gdext 0.5.5)
echo "==> cargo build -p gamestruments-godot --release"
cargo build -p gamestruments-godot --release

BIN_SRC="target/release/libgamestruments_godot.so"
if [[ ! -f "$BIN_SRC" ]]; then
  echo "ERROR: release binary not found at $BIN_SRC" >&2
  exit 1
fi

# 2. Assemble staging with explicit allowlist
STAGING="$(mktemp -d -t gamestruments-kit-XXXXXX)"
trap 'rm -rf "$STAGING"' EXIT

echo "==> Staging at $STAGING (allowlist only)"

# crates source (full for rebuild)
mkdir -p "$STAGING/crates"
cp -a crates/engine "$STAGING/crates/engine"
cp -a crates/godot "$STAGING/crates/godot"
cp crates/README.md "$STAGING/crates/README.md"

# addon layout (buyer drop-in; .gdextension paths are already res://addons/gamestruments/...)
mkdir -p "$STAGING/addons/gamestruments/bin"
cp crates/godot/gamestruments.gdextension "$STAGING/addons/gamestruments/gamestruments.gdextension"
cp "$BIN_SRC" "$STAGING/addons/gamestruments/bin/libgamestruments_godot.so"

# kit content
mkdir -p "$STAGING/kit"
cp -a kit/demo "$STAGING/kit/demo"
cp -a kit/docs "$STAGING/kit/docs"

# Self-contained demo: copy the built addon *into* the demo subtree so that
# opening the extracted `kit/demo/` folder directly as a Godot project works
# (its project.godot + res://kit_demo.tscn + res://addons/gamestruments/...).
# Buyers still get the root addons/ for dropping into their own project.
mkdir -p "$STAGING/kit/demo/addons/gamestruments/bin"
cp crates/godot/gamestruments.gdextension "$STAGING/kit/demo/addons/gamestruments/gamestruments.gdextension"
cp "$BIN_SRC" "$STAGING/kit/demo/addons/gamestruments/bin/libgamestruments_godot.so"

# kit/README.md (one-page buyer overview)
cat > "$STAGING/kit/README.md" << 'KITREADME'
# Gamestruments — Godot 4 Kit

Seed-driven, sample-free, runtime-adaptive music scores for Godot 4 games.

A single `project_secret` (per-title) + instrument palette + seed produces a
deterministic adaptive score at level load. Drive bar-quantized state changes
(e.g. race phases) at runtime via `set_race_state`. Pure synthesis; no samples.

**MIT license** on the Rust core. Full source included in the archive.

## What ships (archive root)

- `addons/gamestruments/` — ready-to-use layout for your project:
  - `gamestruments.gdextension`
  - `bin/libgamestruments_godot.so` (linux.x86_64)
- `kit/demo/addons/gamestruments/` — **identical copy** inside the demo so that
  `kit/demo/` can be opened directly as a standalone Godot project (its
  `project.godot` points at `res://kit_demo.tscn`; the addon is at
  `res://addons/gamestruments/...` relative to the demo root). Use this copy
  only for evaluating the demo; for your own game use the root `addons/`.
- `crates/` — full MIT source (`engine/` + `godot/`) + README; rebuild with
  `cargo build -p gamestruments-godot --release`
- `kit/demo/` — minimal exerciser scene + script for the public API
  (self-contained: open the folder in Godot 4 to run it)
- `kit/docs/` — buyer documentation (README, quickstart, api, limitations, troubleshooting)
- `kit/README.md` (this file)
- `LICENSE.md`, `crates/*/LICENSE.md`, `THIRD_PARTY_NOTICES.md`
- (CHANGELOG excerpt available in the publishing repo at the release tag)

See `kit/docs/README.md` for requirements, quickstart, scope, and claims.

## Quickstart pointer

1. Extract archive.
2. For your game: copy the root `addons/gamestruments/` into your Godot project's `res://addons/`.
   For quick evaluation of the demo: just open the extracted `kit/demo/` folder
   directly as a Godot project (the addon is already inside it at the correct
   relative location).
3. Add `GamestrumentsPlayer` node, set `project_secret` + `style`, call
   `generate("level-seed")` then `set_race_state(...)` as needed.
4. Route its AudioStreamPlayer child (or the node) to a "Music" bus.

Full steps and inspector fields: `kit/docs/quickstart.md` and `kit/docs/api.md`.

## Licenses

- Gamestruments Rust crates (`crates/engine`, `crates/godot`): MIT.
  See `LICENSE.md` (root) and the copies under `crates/*/LICENSE.md`.
- gdext (the godot-rust binding used to build the GDExtension): MPL-2.0.
  See `THIRD_PARTY_NOTICES.md` for attribution and coverage.
- The binary is produced from the MIT sources + MPL-2.0 build dependency.
  Rebuilding from the shipped source pulls gdext via Cargo (subject to MPL-2.0).

Root `LICENSE.md` clarifies the overall project split (AGPL parts for the
authoring Lab are **not** included in this runtime kit).

## Verification

This archive was produced by `tools/package_kit.sh` from a clean pinned
checkout at the release commit. See `docs/kit-qa-runbook.md` (repo) for author
QA steps and the clean-room buyer test contract in `docs/kit-plan.md`.

For the exact shipped files and SHA-256, see the release notes / PR that
landed the tag.
KITREADME

# LICENSE files for crates (MIT from root) + root copy
cp LICENSE.md "$STAGING/LICENSE.md"
cp LICENSE.md "$STAGING/crates/engine/LICENSE.md"
cp LICENSE.md "$STAGING/crates/godot/LICENSE.md"

# THIRD_PARTY_NOTICES.md (gdext MPL-2.0 obligations)
# Shipping the binding *source* is not required for MPL-2.0 when we ship a
# binary produced from it; we include clear attribution + pointer for rebuilders.
cat > "$STAGING/THIRD_PARTY_NOTICES.md" << 'THIRD'
Third-Party Notices
===================

Gamestruments Godot 4 kit

This archive contains a pre-built GDExtension binary (libgamestruments_godot.so)
and the full source of the MIT-licensed Gamestruments crates.

Components under third-party licenses:

- godot (gdext) 0.5.5 and its sub-crates:
    gdextension-api, godot-bindings, godot-cell, godot-core, etc.
  License: Mozilla Public License 2.0 (MPL-2.0)
  Origin: https://github.com/godot-rust/gdext
  What it covers: The Rust bindings and generated glue that let the Rust
    `gamestruments-godot` cdylib register as a Godot 4 GDExtension (api-4-7).
    The MPL-2.0 code is linked into the shipped .so at build time.
  Source of the binding is NOT shipped in this archive. Buyers who rebuild
    from the included `crates/engine` + `crates/godot` sources will have
    `cargo` fetch the exact gdext 0.5.5 tree; at that point the MPL-2.0 terms
    apply to the binding portion.
  Full license text: https://www.mozilla.org/en-US/MPL/2.0/
  (The MPL-2.0 requires that modifications to MPL-covered files be made
   available under MPL; our own crates remain MIT and do not modify the
   gdext sources.)

Gamestruments crates (engine + godot glue) are under the MIT License.
See LICENSE.md and the per-crate copies for the full text.

No other third-party notices are required for the contents of this archive.
If you rebuild or redistribute, satisfy the obligations of any dependencies
you pull (e.g. via cargo tree).
THIRD

# 3. Normalize for determinism (mtimes + permissions)
echo "==> Normalizing staging for deterministic zip (fixed mtime 2024-01-01, sorted, -X)"
find "$STAGING" -type f -exec touch -t 202401010000.00 {} +
find "$STAGING" -type d -exec touch -t 202401010000.00 {} +
# ensure readable; zip will use current umask but -X helps
chmod -R a+rX "$STAGING"

# 4. Build deterministic zip
ZIPNAME="gamestruments-$VERSION-godot4.zip"
ZIPPATH="$OUT_DIR/$ZIPNAME"

( cd "$STAGING" && \
  find . -type f | LC_ALL=C sort | \
  zip -X -9 -@ "$ZIPPATH" )

echo "==> Zip created: $ZIPPATH"

# 5. Report
BYTES=$(stat -c%s "$ZIPPATH" 2>/dev/null || stat -f%z "$ZIPPATH")
SHA=$(sha256sum "$ZIPPATH" | awk '{print $1}')

echo "==> Size: $BYTES bytes"
echo "==> SHA-256: $SHA"

ls -l "$ZIPPATH"
echo "gamestruments-$VERSION-godot4.zip $BYTES $SHA" > "$OUT_DIR/gamestruments-$VERSION-godot4.zip.sha256.txt"

echo "==> Done. Artifact at $ZIPPATH"
echo "    (residual non-determinism note: only the compiled .so may embed"
echo "     rustc/linker build IDs or timestamps; zip container, text, and"
echo "     file order are normalized.)"
