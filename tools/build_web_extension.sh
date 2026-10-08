#!/usr/bin/env bash
# Build the GDExtension's web side module for Godot's single-threaded dlink
# web templates (Thread Support off: no SharedArrayBuffer or cross-origin
# isolation needed).
#
#   tools/build_web_extension.sh [--out-dir DIR]
#
# Writes DIR/gamestruments_godot.wasm. Needs emcc from the Emscripten version
# that built Godot's templates and the pinned nightly Rust (gdext's web support
# builds std with -Zbuild-std), both in tools/web-toolchain.env.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=web-toolchain.env
source "$root/tools/web-toolchain.env"
out_dir="target/web-extension"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir) out_dir="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

cd "$root"

if ! command -v emcc >/dev/null 2>&1; then
  echo "ERROR: emcc not found; activate emsdk $EMSCRIPTEN_VERSION first" >&2
  exit 1
fi
actual_em="$(emcc --version | head -n1 | sed -E 's/.* ([0-9]+\.[0-9]+\.[0-9]+).*/\1/')"
if [[ "$actual_em" != "$EMSCRIPTEN_VERSION" ]]; then
  echo "ERROR: emcc $actual_em found; Godot's web templates are built with $EMSCRIPTEN_VERSION" >&2
  exit 1
fi

# panic=abort: the Godot web templates are built without exception support,
# and nightly Rust no longer offers JS exceptions for Emscripten (wasm-eh would
# import a __cpp_exception tag the templates do not provide).
flags=(
  "-C panic=abort"
  "-C link-args=-sSIDE_MODULE=2"
  "-C llvm-args=-enable-emscripten-cxx-exceptions=0"
  "-Z default-visibility=hidden"
  "-Z link-native-libraries=no"
  "--remap-path-prefix=$root=."
  "--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=cargo"
  "--remap-path-prefix=${RUSTUP_HOME:-$HOME/.rustup}=rustup"
)

target_dir="target/web-nothreads"
RUSTFLAGS="${flags[*]}" CARGO_INCREMENTAL=0 \
  cargo "+$RUST_WEB_TOOLCHAIN" build -Zbuild-std=std,panic_abort \
    -p gamestruments-godot --target wasm32-unknown-emscripten --release --locked \
    --target-dir "$target_dir" --features nothreads
mkdir -p "$out_dir"
cp "$target_dir/wasm32-unknown-emscripten/release/gamestruments_godot.wasm" "$out_dir/"
node "$root/tools/verify_web_extension.mjs" "$out_dir/gamestruments_godot.wasm"
(cd "$out_dir" && sha256sum gamestruments_godot.wasm)
