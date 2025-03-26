#!/usr/bin/env bash

set -e

QUERY="$*"
OUTPUT_DIR="./output"

if [ -z "$QUERY" ]; then
  echo "❌ Please provide a search query."
  echo "Usage: ./scrape.sh \"your search query here\""
  exit 1
fi

# Make sure output directory exists
mkdir -p "$OUTPUT_DIR"

# Puppeteer is irritating to get running properly inside Docker. I did it before but
# then ran into issues with the file creation (wouldn't create on local machine)
# I got fed up and just created this bash script instead of only using Docker

# Run Puppeteer (assumes node_modules are installed already)
node search-engine-scraper/scraper.js "$QUERY"

echo "▶️ Running deep analysis in Podman..."

podman build --progress=plain -t deep-research .

podman run --rm -it \
  -v "$(pwd)/$OUTPUT_DIR":/app/output:Z \
  -w /home/appuser/app \
  deep-research \
  python3 run.py "$QUERY"