#!/usr/bin/env bash
# Builds the Audio Lab with relative asset paths and packages it for an
# itch.io HTML5 upload (index.html must sit at the zip root).
set -euo pipefail

cd "$(dirname "$0")/.."

npm run typecheck
npm run build:packages
npx vite build --base ./

rm -f dist/gamestruments-lab-itch.zip
(
  cd dist/demo
  zip -q -r ../gamestruments-lab-itch.zip . -x '.*' -x '*/.*'
)

echo "==> itch demo zip: dist/gamestruments-lab-itch.zip"
ls -la dist/gamestruments-lab-itch.zip
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum dist/gamestruments-lab-itch.zip
else
  shasum -a 256 dist/gamestruments-lab-itch.zip
fi
