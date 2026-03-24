#!/bin/bash
# Download required binaries and frontend dist for local development.
# Run from the repo root: ./scripts/setup-dev.sh

set -euo pipefail

TRIPLE=$(rustc --print host-tuple)

mkdir -p "src-tauri/binaries"

echo "Detected target triple: $TRIPLE"

echo "Downloading altmanager-ws for $TRIPLE..."
curl -fL "https://github.com/altmanager/altmanager-ws/releases/latest/download/altmanager-ws-${TRIPLE}" \
  -o "src-tauri/binaries/altmanager-ws-$TRIPLE"
chmod +x "src-tauri/binaries/altmanager-ws-$TRIPLE"

echo "Downloading altmanager-web dist..."
curl -fL "https://github.com/altmanager/altmanager-web/releases/latest/download/dist.zip" \
  -o dist.zip
unzip -q -o dist.zip
rm dist.zip

echo "Generating icons..."
npx tauri icon src-tauri/icons/icon.svg

echo "Done."
