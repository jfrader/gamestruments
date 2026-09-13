#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <gamestruments.zip> [godot-binary]" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARCHIVE="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
CHECKSUM="$ARCHIVE.sha256.txt"
GODOT_BIN="${2:-}"
EXTRACTED="$(mktemp -d -t gamestruments-verify-XXXXXX)"
trap 'rm -rf "$EXTRACTED"' EXIT

test -f "$CHECKSUM" || { echo "ERROR: archive checksum sidecar is missing" >&2; exit 1; }
(cd "$(dirname "$ARCHIVE")" && sha256sum -c "$(basename "$CHECKSUM")")
unzip -q "$ARCHIVE" -d "$EXTRACTED"

required=(
  "README.md"
  "RELEASE-MANIFEST.json"
  "Cargo.toml"
  "Cargo.lock"
  "rust-toolchain.toml"
  "CHANGELOG.md"
  "LICENSE.md"
  "THIRD_PARTY_NOTICES.md"
  "licenses/MPL-2.0.txt"
  "licenses/Apache-2.0.txt"
  "licenses/MIT.txt"
  "licenses/Unicode-3.0.txt"
  "licenses/Unlicense.txt"
  "licenses/cargo-dependencies.json"
  "licenses/glam-0.32.1-ATTRIBUTION.md"
  "licenses/rust-1.94.0-COPYRIGHT-library.html"
  "catalog/racing/tiny-torque-level-004/score.json"
  "addons/gamestruments/gamestruments.gdextension"
  "addons/gamestruments/bin/libgamestruments_godot.so"
  "addons/gamestruments/bin/gamestruments_godot.dll"
  "addons/gamestruments/bin/libgamestruments_godot.dylib"
  "kit/examples/project.godot"
  "kit/examples/README.md"
  "kit/examples/01-playback/README.md"
  "kit/examples/01-playback/playback.tscn"
  "kit/examples/01-playback/playback.gd"
  "kit/examples/02-game-signals/README.md"
  "kit/examples/02-game-signals/game_signals.tscn"
  "kit/examples/02-game-signals/game_signals.gd"
  "kit/examples/03-song-form/README.md"
  "kit/examples/03-song-form/song_form.tscn"
  "kit/examples/03-song-form/song_form.gd"
  "kit/examples/tools/generate_example_scenes.gd"
  "kit/examples/tools/runtime_smoke.gd"
  "kit/examples/addons/gamestruments/gamestruments.gdextension"
  "kit/examples/addons/gamestruments/bin/libgamestruments_godot.so"
  "kit/examples/addons/gamestruments/bin/gamestruments_godot.dll"
  "kit/examples/addons/gamestruments/bin/libgamestruments_godot.dylib"
  "kit/docs/README.md"
)
if [[ -e "$EXTRACTED/kit/demo" ]]; then
  echo "ERROR: archive contains the retired kit/demo project" >&2
  exit 1
fi
for file in "${required[@]}"; do
  test -f "$EXTRACTED/$file" || { echo "ERROR: archive is missing $file" >&2; exit 1; }
done

# Examples subtree must exist and no Godot editor cache may ship anywhere.
test -d "$EXTRACTED/kit/examples" || { echo "ERROR: archive is missing kit/examples" >&2; exit 1; }
if find "$EXTRACTED" -type d -name '.godot' -print -quit | grep -q .; then
  echo "ERROR: archive contains a .godot directory" >&2
  exit 1
fi

node "$REPO_ROOT/tools/verify-native-libraries.mjs" --root "$EXTRACTED"
node "$REPO_ROOT/tests/verify-release-manifest.mjs" \
  "$EXTRACTED" \
  "$(basename "$ARCHIVE")"

if find "$EXTRACTED" -type f \( -iname '*.wav' -o -iname '*.ogg' -o -iname '*.mp3' \) -print -quit | grep -q .; then
  echo "ERROR: buyer archive contains an audio asset" >&2
  exit 1
fi
if find "$EXTRACTED" -type f \( -iname '*.ts' -o -iname '*.tsx' -o -iname '*.js' -o -iname '*.mjs' \) -print -quit | grep -q .; then
  echo "ERROR: buyer archive contains authoring code" >&2
  exit 1
fi
if find "$EXTRACTED" -type l -print -quit | grep -q .; then
  echo "ERROR: buyer archive contains a symbolic link" >&2
  exit 1
fi
if grep -R -I -E '/home/|/Users/' "$EXTRACTED" >/dev/null; then
  echo "ERROR: buyer archive contains a private build path" >&2
  exit 1
fi

cargo metadata --manifest-path "$EXTRACTED/Cargo.toml" --locked --format-version 1 > "$EXTRACTED/cargo-metadata.json"
node "$REPO_ROOT/tests/verify-third-party-notices.mjs" \
  "$EXTRACTED/cargo-metadata.json" \
  "$EXTRACTED/THIRD_PARTY_NOTICES.md" \
  "$EXTRACTED/licenses/cargo-dependencies.json" \
  "$EXTRACTED/licenses"
cargo test --manifest-path "$EXTRACTED/Cargo.toml" --workspace --locked
cargo build --manifest-path "$EXTRACTED/Cargo.toml" --workspace --all-targets --release --locked

if [[ -n "$GODOT_BIN" ]]; then
  node "$REPO_ROOT/tests/godot-package-smoke.mjs" \
    --project-root "$EXTRACTED" \
    --godot "$GODOT_BIN" \
    --library "$EXTRACTED/addons/gamestruments/bin/libgamestruments_godot.so"
fi

echo "GAMESTRUMENTS_ARCHIVE_VERIFY_PASS"
