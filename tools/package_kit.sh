#!/usr/bin/env bash
# tools/package_kit.sh
# Package the Gamestruments buyer release archive (Godot 4 kit).
#
# Usage:
#   tools/package_kit.sh --version <ver> --assets-dir <dir> [--out-dir <dir>]
#
# - VERSION defaults to 0.1.0-rc2 (or first arg)
# - Produces gamestruments-$VERSION-godot4.zip in OUT_DIR (default /tmp/opencode)
# - Requires prebuilt Linux, Windows, and universal macOS libraries in ASSETS_DIR
# - Uses explicit allowlist for reproducible buyer artifact.
# - Zip is made deterministic: sorted file list + zip -X (no extra fields) +
#   normalized mtimes in staging. Residual non-determinism: native binaries
#   may contain linker timestamps / build IDs / UUIDs from cargo/rustc even in
#   release; we do not strip the binary itself (honest). Source files, text,
#   and zip metadata are normalized.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

DEFAULT_VERSION="0.1.0-rc2"
VERSION="$DEFAULT_VERSION"
VERSION_SET=false
OUT_DIR="/tmp/opencode"
ASSETS_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version|-v)
      VERSION="$2"; VERSION_SET=true; shift 2 ;;
    --out-dir|--out)
      OUT_DIR="$2"; shift 2 ;;
    --assets-dir)
      ASSETS_DIR="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: $0 --version X.Y.Z-rcN --assets-dir DIR [--out-dir DIR]"; exit 0 ;;
    *)
      # positional fallback for version
      if [[ "$VERSION_SET" == false && "$1" != --* ]]; then
        VERSION="$1"; VERSION_SET=true; shift
      else
        echo "Unknown arg: $1" >&2; exit 1
      fi
      ;;
  esac
done

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
  echo "ERROR: version must be SemVer-like (for example 0.1.0 or 0.1.0-rc2)" >&2
  exit 1
fi
if [[ -z "$ASSETS_DIR" ]]; then
  echo "ERROR: --assets-dir is required for a cross-platform buyer archive" >&2
  exit 1
fi
ASSETS_DIR="$(cd "$ASSETS_DIR" && pwd)"

NATIVE_LIBS=(
  "libgamestruments_godot.so"
  "gamestruments_godot.dll"
  "libgamestruments_godot.dylib"
)
for library in "${NATIVE_LIBS[@]}"; do
  if [[ ! -f "$ASSETS_DIR/$library" ]]; then
    echo "ERROR: release binary not found at $ASSETS_DIR/$library" >&2
    exit 1
  fi
done

echo "==> Packaging Gamestruments kit version: $VERSION"
echo "==> Repo: $REPO_ROOT (commit: $(git rev-parse --short HEAD))"
echo "==> Output dir: $OUT_DIR"
echo "==> Native assets: $ASSETS_DIR"

mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"

# 1. Assemble staging with explicit allowlist
STAGING="$(mktemp -d -t gamestruments-kit-XXXXXX)"
trap 'rm -rf "$STAGING"' EXIT

echo "==> Staging at $STAGING (allowlist only)"

# crates source (full for rebuild)
mkdir -p "$STAGING/crates"
cp -a crates/engine "$STAGING/crates/engine"
cp -a crates/godot "$STAGING/crates/godot"
cp crates/README.md "$STAGING/crates/README.md"
cp crates/LICENSE.md "$STAGING/crates/LICENSE.md"
cp Cargo.toml Cargo.lock rust-toolchain.toml CHANGELOG.md "$STAGING/"

# addon layout (buyer drop-in; .gdextension paths are already res://addons/gamestruments/...)
mkdir -p "$STAGING/addons/gamestruments/bin"
cp crates/godot/gamestruments.gdextension "$STAGING/addons/gamestruments/gamestruments.gdextension"
for library in "${NATIVE_LIBS[@]}"; do
  cp "$ASSETS_DIR/$library" "$STAGING/addons/gamestruments/bin/$library"
done

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
for library in "${NATIVE_LIBS[@]}"; do
  cp "$ASSETS_DIR/$library" "$STAGING/kit/demo/addons/gamestruments/bin/$library"
done

# kit/README.md (one-page buyer overview)
cat > "$STAGING/kit/README.md" << 'KITREADME'
# Gamestruments — Adaptive Racing Music for Godot 4

Seed-driven, sample-free, runtime-adaptive racing music for Godot 4 games.

A single `project_secret` (a per-title deterministic namespace, not a security
credential) + instrument palette + seed produces a
deterministic adaptive score at level load. Drive bar-quantized state changes
(e.g. race phases) at runtime via `set_race_state`. Pure synthesis; no samples.

**MIT license** on the Rust core. Full source included in the archive.

## What ships (archive root)

- `addons/gamestruments/` — ready-to-use layout for your project:
  - `gamestruments.gdextension`
  - `bin/libgamestruments_godot.so` (Linux x86_64)
  - `bin/gamestruments_godot.dll` (Windows x86_64)
  - `bin/libgamestruments_godot.dylib` (macOS universal: arm64 + x86_64)
- `kit/demo/addons/gamestruments/` — **identical copy** inside the demo so that
  `kit/demo/` can be opened directly as a standalone Godot project (its
  `project.godot` points at `res://kit_demo.tscn`; the addon is at
  `res://addons/gamestruments/...` relative to the demo root). Use this copy
  only for evaluating the demo; for your own game use the root `addons/`.
- `crates/` plus the root Cargo manifests, lockfile, and pinned toolchain — full
  MIT source (`engine/` + `godot/`) + README; rebuild with
  `cargo build -p gamestruments-godot --release`
- `kit/demo/` — minimal exerciser scene + script for the public API
  (self-contained: open the folder in Godot 4 to run it)
- `kit/docs/` — buyer documentation (README, quickstart, api, limitations, troubleshooting)
- `kit/README.md` (this file)
- `LICENSE.md`, `crates/*/LICENSE.md`, `THIRD_PARTY_NOTICES.md`, and full
  dependency license texts under `licenses/`
- `CHANGELOG.md`

See `kit/docs/README.md` for requirements, quickstart, scope, and claims.

## Quickstart pointer

1. Extract archive.
2. For your game: copy the root `addons/gamestruments/` into your Godot project's `res://addons/`.
   For quick evaluation of the demo: just open the extracted `kit/demo/` folder
   directly as a Godot project (the addon is already inside it at the correct
   relative location).
3. Add a `GamestrumentsPlayer` node, set `project_secret` + `style`, call
   `generate("level-seed")`, check that it returns `true`, then call
   `set_race_state(...)` as needed.
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

This archive was produced by the release workflow from a pinned checkout and
target-native libraries. See the release packet for automated and human QA.

For the exact shipped files and SHA-256, see the release notes / PR that
landed the tag.
KITREADME

# Project, crate, and dependency licenses
cp LICENSE.md "$STAGING/LICENSE.md"
cp crates/LICENSE.md "$STAGING/crates/engine/LICENSE.md"
cp crates/LICENSE.md "$STAGING/crates/godot/LICENSE.md"
cp THIRD_PARTY_NOTICES.md "$STAGING/THIRD_PARTY_NOTICES.md"
cp -a licenses "$STAGING/licenses"

# 2. Normalize for determinism (mtimes + permissions)
echo "==> Normalizing staging for deterministic zip (fixed mtime 2024-01-01, sorted, -X)"
find "$STAGING" -type f -exec touch -t 202401010000.00 {} +
find "$STAGING" -type d -exec touch -t 202401010000.00 {} +
# ensure readable; zip will use current umask but -X helps
chmod -R a+rX "$STAGING"

# 3. Build deterministic zip
ZIPNAME="gamestruments-$VERSION-godot4.zip"
ZIPPATH="$OUT_DIR/$ZIPNAME"
rm -f "$ZIPPATH"

( cd "$STAGING" && \
  find . -type f | LC_ALL=C sort | \
  zip -X -9 -@ "$ZIPPATH" )

echo "==> Zip created: $ZIPPATH"

# 4. Report
BYTES=$(stat -c%s "$ZIPPATH" 2>/dev/null || stat -f%z "$ZIPPATH")
SHA=$(sha256sum "$ZIPPATH" | awk '{print $1}')

echo "==> Size: $BYTES bytes"
echo "==> SHA-256: $SHA"

ls -l "$ZIPPATH"
echo "$SHA  $ZIPNAME" > "$OUT_DIR/$ZIPNAME.sha256.txt"

echo "==> Done. Artifact at $ZIPPATH"
echo "    (residual non-determinism note: compiled native libraries may embed"
echo "     rustc/linker build IDs or timestamps; zip container, text, and"
echo "     file order are normalized.)"
