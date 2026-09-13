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
# - Uses an explicit allowlist for the buyer artifact.
# - Zip metadata is deterministic: sorted file list + zip -X (no extra fields) +
#   normalized mtimes and permissions in staging. Residual non-determinism: native binaries
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
GODOT_VERSION="${GODOT_VERSION:-4.7.2-stable}"

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
node tools/verify-native-libraries.mjs --assets-dir "$ASSETS_DIR"

SOURCE_COMMIT="$(git rev-parse HEAD)"
SOURCE_REF="${GAMESTRUMENTS_SOURCE_REF:-${GITHUB_HEAD_REF:-${GITHUB_REF_NAME:-}}}"
if [[ -z "$SOURCE_REF" ]]; then
  SOURCE_REF="$(git symbolic-ref --quiet --short HEAD 2>/dev/null || git rev-parse --short HEAD)"
fi

PROVENANCE="${GAMESTRUMENTS_PROVENANCE:-}"
if [[ -z "$PROVENANCE" ]]; then
  case "${GITHUB_EVENT_NAME:-}" in
    pull_request) PROVENANCE="pull_request" ;;
    workflow_dispatch) PROVENANCE="workflow_dispatch" ;;
    push)
      if [[ "${GITHUB_REF_TYPE:-}" == "tag" ]]; then PROVENANCE="release"; else PROVENANCE="local"; fi
      ;;
    *) PROVENANCE="local" ;;
  esac
fi

DIRTY=false
if [[ -n "$(git status --porcelain)" ]]; then
  DIRTY=true
fi

if [[ "$PROVENANCE" == "release" ]]; then
  if [[ "$DIRTY" == true ]]; then
    echo "ERROR: release provenance requires a clean source checkout" >&2
    exit 1
  fi
  if [[ "$SOURCE_REF" != "v$VERSION" ]]; then
    echo "ERROR: release source ref must be v$VERSION, got $SOURCE_REF" >&2
    exit 1
  fi
  TAG_COMMIT="$(git rev-parse -q --verify "refs/tags/$SOURCE_REF^{commit}" 2>/dev/null || true)"
  if [[ "$TAG_COMMIT" != "$SOURCE_COMMIT" ]]; then
    echo "ERROR: release tag $SOURCE_REF does not resolve to $SOURCE_COMMIT" >&2
    exit 1
  fi
fi

WORKFLOW_URL=""
if [[ -n "${GITHUB_SERVER_URL:-}" && -n "${GITHUB_REPOSITORY:-}" && -n "${GITHUB_RUN_ID:-}" ]]; then
  WORKFLOW_URL="$GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID"
fi
if [[ "$PROVENANCE" == "release" && -z "$WORKFLOW_URL" ]]; then
  echo "ERROR: release provenance requires a GitHub Actions workflow URL" >&2
  exit 1
fi

echo "==> Packaging Gamestruments kit version: $VERSION"
echo "==> Repo: $REPO_ROOT (commit: ${SOURCE_COMMIT:0:12}, ref: $SOURCE_REF, provenance: $PROVENANCE, dirty: $DIRTY)"
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
mkdir -p "$STAGING/catalog/racing/tiny-torque-level-004"
cp catalog/racing/tiny-torque-level-004/score.json \
  "$STAGING/catalog/racing/tiny-torque-level-004/score.json"

# addon layout (buyer drop-in; .gdextension paths are already res://addons/gamestruments/...)
mkdir -p "$STAGING/addons/gamestruments/bin"
cp crates/godot/gamestruments.gdextension "$STAGING/addons/gamestruments/gamestruments.gdextension"
for library in "${NATIVE_LIBS[@]}"; do
  cp "$ASSETS_DIR/$library" "$STAGING/addons/gamestruments/bin/$library"
done

# kit content (explicit allowlist; never copy .godot, addons, or *.uid caches)
mkdir -p "$STAGING/kit" \
  "$STAGING/kit/examples/01-playback" \
  "$STAGING/kit/examples/02-game-signals" \
  "$STAGING/kit/examples/03-song-form" \
  "$STAGING/kit/examples/tools"
cp kit/examples/project.godot "$STAGING/kit/examples/project.godot"
cp kit/examples/README.md "$STAGING/kit/examples/README.md"
cp kit/examples/01-playback/README.md kit/examples/01-playback/playback.gd kit/examples/01-playback/playback.tscn \
  "$STAGING/kit/examples/01-playback/"
cp kit/examples/02-game-signals/README.md kit/examples/02-game-signals/game_signals.gd kit/examples/02-game-signals/game_signals.tscn \
  "$STAGING/kit/examples/02-game-signals/"
cp kit/examples/03-song-form/README.md kit/examples/03-song-form/song_form.gd kit/examples/03-song-form/song_form.tscn \
  "$STAGING/kit/examples/03-song-form/"
cp kit/examples/tools/generate_example_scenes.gd kit/examples/tools/runtime_smoke.gd \
  "$STAGING/kit/examples/tools/"
cp -a kit/docs "$STAGING/kit/docs"
cp kit/README.md "$STAGING/README.md"
# Same document inside kit/ with links rewritten for its deeper location.
sed -e 's#kit/docs/#docs/#g' -e 's#kit/examples/#examples/#g' kit/README.md > "$STAGING/kit/README.md"

# Self-contained examples: copy the built addon *into* the examples subtree so that
# opening the extracted `kit/examples/` folder directly as a Godot project works
# (its project.godot + res://01-playback/... + res://addons/gamestruments/...).
# Buyers still get the root addons/ for dropping into their own project.
mkdir -p "$STAGING/kit/examples/addons/gamestruments/bin"
cp crates/godot/gamestruments.gdextension "$STAGING/kit/examples/addons/gamestruments/gamestruments.gdextension"
for library in "${NATIVE_LIBS[@]}"; do
  cp "$ASSETS_DIR/$library" "$STAGING/kit/examples/addons/gamestruments/bin/$library"
done

# Project, crate, and dependency licenses
cp LICENSE.md "$STAGING/LICENSE.md"
cp crates/LICENSE.md "$STAGING/crates/engine/LICENSE.md"
cp crates/LICENSE.md "$STAGING/crates/godot/LICENSE.md"
cp THIRD_PARTY_NOTICES.md "$STAGING/THIRD_PARTY_NOTICES.md"
cp -a licenses "$STAGING/licenses"

# Build provenance and native-library digests
node tools/create-release-manifest.mjs \
  --out "$STAGING/RELEASE-MANIFEST.json" \
  --repo-root "$REPO_ROOT" \
  --assets-dir "$ASSETS_DIR" \
  --version "$VERSION" \
  --commit "$SOURCE_COMMIT" \
  --ref "$SOURCE_REF" \
  --provenance "$PROVENANCE" \
  --dirty "$DIRTY" \
  --godot-version "$GODOT_VERSION" \
  --workflow-url "$WORKFLOW_URL"

# 2. Normalize for determinism (mtimes + permissions)
echo "==> Normalizing staging for deterministic zip (fixed mtime 2024-01-01, sorted, -X)"
find "$STAGING" -type f -exec touch -t 202401010000.00 {} +
find "$STAGING" -type d -exec touch -t 202401010000.00 {} +
find "$STAGING" -type f -exec chmod 0644 {} +
find "$STAGING" -type d -exec chmod 0755 {} +

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
