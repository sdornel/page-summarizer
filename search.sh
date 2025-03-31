#!/usr/bin/env bash
set -eo pipefail

QUERY="$1"
if [[ -z "$QUERY" ]]; then
  echo "❌ Usage: ./search.sh \"your search query\""
  exit 1
fi

# Run Puppeteer (assumes node_modules are installed already)
node src/scrapers-js/search-engine-scraper.js "$QUERY" # required unless you already have url array inside urls.json

# Build with security flags
# podman build --no-cache --pull -t deep-research .
podman build -t deep-research .

# Ensure output directory exists
mkdir -p "./output"

# Run with strict confinement
podman run -it --rm \
  --read-only \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  --tmpfs /run:rw,noexec,nosuid,size=8m \
  --shm-size=256m \
  -v "$(pwd)/output:/app/output:Z" \
  deep-research "$QUERY"