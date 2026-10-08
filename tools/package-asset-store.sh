#!/usr/bin/env bash
# tools/package-asset-store.sh
# Package the Gamestruments Godot Asset Store ZIP (addon drop-in only).
#
# Usage:
#   tools/package-asset-store.sh --version <ver> --assets-dir <dir> [--out-dir <dir>]
#   tools/package-asset-store.sh --version <ver> --kit-zip <kit.zip> [--out-dir <dir>]
#
# - Produces gamestruments-$VERSION-godot4-asset-store.zip in OUT_DIR
#   (default dist)
# - Stages addons/gamestruments/{README.md, LICENSE.md, gamestruments.gdextension, bin/}
# - Native libraries come from --assets-dir or are extracted from --kit-zip
# - Zip metadata is deterministic: sorted file list + zip -X + normalized mtimes
# - Does not replace tools/package_kit.sh (full release archive).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

DEFAULT_VERSION="0.1.2"
VERSION="$DEFAULT_VERSION"
VERSION_SET=false
OUT_DIR="dist"
ASSETS_DIR=""
KIT_ZIP=""
GODOT_VERSION="${GODOT_VERSION:-4.7.2-stable}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version|-v)
      VERSION="$2"; VERSION_SET=true; shift 2 ;;
    --out-dir|--out)
      OUT_DIR="$2"; shift 2 ;;
    --assets-dir)
      ASSETS_DIR="$2"; shift 2 ;;
    --kit-zip)
      KIT_ZIP="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: $0 --version X.Y.Z (--assets-dir DIR | --kit-zip ZIP) [--out-dir DIR]"
      exit 0 ;;
    *)
      if [[ "$VERSION_SET" == false && "$1" != --* ]]; then
        VERSION="$1"; VERSION_SET=true; shift
      else
        echo "Unknown arg: $1" >&2; exit 1
      fi
      ;;
  esac
done

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
  echo "ERROR: version must be SemVer-like (for example 0.1.2)" >&2
  exit 1
fi
if [[ -n "$ASSETS_DIR" && -n "$KIT_ZIP" ]]; then
  echo "ERROR: choose either --assets-dir or --kit-zip" >&2
  exit 1
fi
if [[ -z "$ASSETS_DIR" && -z "$KIT_ZIP" ]]; then
  echo "ERROR: --assets-dir or --kit-zip is required" >&2
  exit 1
fi

for required in \
  addons/gamestruments/README.md \
  addons/gamestruments/LICENSE.md \
  addons/gamestruments/icon.png \
  crates/godot/gamestruments.gdextension
do
  if [[ ! -f "$required" ]]; then
    echo "ERROR: missing $required" >&2
    exit 1
  fi
done

NATIVE_LIBS=(
  "libgamestruments_godot.so"
  "gamestruments_godot.dll"
  "libgamestruments_godot.dylib"
)

KIT_EXTRACT=""
cleanup() {
  if [[ -n "$KIT_EXTRACT" ]]; then
    rm -rf "$KIT_EXTRACT"
  fi
}
trap cleanup EXIT

if [[ -n "$KIT_ZIP" ]]; then
  if [[ ! -f "$KIT_ZIP" ]]; then
    echo "ERROR: kit zip not found at $KIT_ZIP" >&2
    exit 1
  fi
  KIT_ZIP="$(cd "$(dirname "$KIT_ZIP")" && pwd)/$(basename "$KIT_ZIP")"
  KIT_EXTRACT="$(mktemp -d -t gamestruments-store-kit-XXXXXX)"
  unzip -q "$KIT_ZIP" \
    "addons/gamestruments/bin/libgamestruments_godot.so" \
    "addons/gamestruments/bin/gamestruments_godot.dll" \
    "addons/gamestruments/bin/libgamestruments_godot.dylib" \
    -d "$KIT_EXTRACT"
  ASSETS_DIR="$KIT_EXTRACT/addons/gamestruments/bin"
fi

ASSETS_DIR="$(cd "$ASSETS_DIR" && pwd)"
for library in "${NATIVE_LIBS[@]}"; do
  if [[ ! -f "$ASSETS_DIR/$library" ]]; then
    echo "ERROR: release binary not found at $ASSETS_DIR/$library" >&2
    exit 1
  fi
done
node tools/verify-native-libraries.mjs --assets-dir "$ASSETS_DIR"

echo "==> Packaging Gamestruments Asset Store zip version: $VERSION"
echo "==> Repo: $REPO_ROOT"
echo "==> Native assets: $ASSETS_DIR"
echo "==> Godot compatibility: $GODOT_VERSION (descriptor minimum 4.7)"

mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"

STAGING="$(mktemp -d -t gamestruments-store-XXXXXX)"
# Replace the kit-extract-only trap with one that also removes staging.
cleanup() {
  rm -rf "$STAGING"
  if [[ -n "$KIT_EXTRACT" ]]; then
    rm -rf "$KIT_EXTRACT"
  fi
}
trap cleanup EXIT

echo "==> Staging at $STAGING"
mkdir -p "$STAGING/addons/gamestruments/bin"
cp addons/gamestruments/README.md "$STAGING/addons/gamestruments/README.md"
cp addons/gamestruments/LICENSE.md "$STAGING/addons/gamestruments/LICENSE.md"
cp addons/gamestruments/icon.png "$STAGING/addons/gamestruments/icon.png"
cp crates/godot/gamestruments.gdextension \
  "$STAGING/addons/gamestruments/gamestruments.gdextension"
for library in "${NATIVE_LIBS[@]}"; do
  cp "$ASSETS_DIR/$library" "$STAGING/addons/gamestruments/bin/$library"
done

echo "==> Normalizing staging for deterministic zip (fixed mtime 2024-01-01, sorted, -X)"
find "$STAGING" -type f -exec touch -t 202401010000.00 {} +
find "$STAGING" -type d -exec touch -t 202401010000.00 {} +
find "$STAGING" -type f -exec chmod 0644 {} +
find "$STAGING" -type d -exec chmod 0755 {} +

ZIPNAME="gamestruments-$VERSION-godot4-asset-store.zip"
ZIPPATH="$OUT_DIR/$ZIPNAME"
rm -f "$ZIPPATH"

( cd "$STAGING" && \
  find . -type f | LC_ALL=C sort | \
  zip -X -9 -@ "$ZIPPATH" )

BYTES=$(stat -c%s "$ZIPPATH" 2>/dev/null || stat -f%z "$ZIPPATH")
SHA=$(sha256sum "$ZIPPATH" | awk '{print $1}')

echo "==> Zip created: $ZIPPATH"
echo "==> Size: $BYTES bytes"
echo "==> SHA-256: $SHA"
echo "$SHA  $ZIPNAME" > "$OUT_DIR/$ZIPNAME.sha256.txt"
echo "==> Done."
