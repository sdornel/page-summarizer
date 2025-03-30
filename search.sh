#!/usr/bin/env bash
set -eo pipefail

# Build with security flags
podman build --no-cache --pull -t deep-research .

# Ensure output directory exists
mkdir -p "./output"

# Run with strict confinement
podman run -it --rm \
  --read-only \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  --tmpfs /tmp:rw,noexec,nosuid,size=100m \
  -v "$(pwd)/output:/app/output" \
  deep-research "$@"