#!/usr/bin/env bash

# ===== PROD =====
# set -eo pipefail

# QUERY="$1"
# if [[ -z "$QUERY" ]]; then
#   echo "❌ Usage: ./search.sh \"your search query\""
#   exit 1
# fi

# export QUERY

# export RUST_LOG="${RUST_LOG:-info}"
# export RUST_BACKTRACE="${RUST_BACKTRACE:-full}"

# node src/scrapers-js/search-engine-scraper.js "$QUERY" # required unless you already have url array inside urls.json

# # Ensure volumes exist
# mkdir -p ./output && touch ./output/urls.json ./output/summaries.json

# docker compose up --build




QUERY="$1"
if [[ -z "$QUERY" ]]; then
  echo "❌ Please provide a search query."
  exit 1
fi

export QUERY
export RUST_LOG="${RUST_LOG:-info}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-full}"

mkdir -p ./output && touch ./output/urls.json ./output/summaries.json

node src/scrapers-js/search-engine-scraper.js "$QUERY" # required unless you already have url array inside urls.json

# 👇 No need to inline QUERY=... docker compose, just rely on export above
docker compose up --build