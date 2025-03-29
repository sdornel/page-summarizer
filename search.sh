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
# node scrapers-js/search-engine-scraper.js "$QUERY" # required unless you already have url array inside urls.json

echo "▶️ Running deep analysis in Podman..."

podman build --progress=plain -t deep-research . # if you need to build again (takes a while)

podman run --rm -it \
  -v "$(pwd)/$OUTPUT_DIR":/home/appuser/app/output:Z \
  deep-research \
  python3 src/run.py "$QUERY"
