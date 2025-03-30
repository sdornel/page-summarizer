#!/usr/bin/env bash
set -e

if [ "$#" -eq 0 ]; then
  echo "❌ Please provide a search query."
  echo "Usage: ./search.sh \"your search query here\""
  exit 1
fi

OUTPUT_DIR="./output"
TARGET_DIR="./target"

# Ensure output and target directories exist on the host
mkdir -p "$OUTPUT_DIR"
mkdir -p "$TARGET_DIR"

echo "▶️ Building Docker image..."
podman build --progress=plain -t deep-research .

echo "▶️ Running container with hardened security options..."
podman run --rm -it \
  --read-only \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  --tmpfs /tmp:rw,exec,nosuid,size=100m \
  -v "$(pwd)/output":/home/appuser/app/output:Z \
  -v "$(pwd)/target":/home/appuser/app/target:rw,Z \
  deep-research "$1"
