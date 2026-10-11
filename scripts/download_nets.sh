#!/bin/bash
# Download KataGo neural networks into nets/ (skips files already present).
# Used by run-local.sh and runpod-push.sh.
#
# The standard net URL rotates with each training run, so it is scraped from
# katagotraining.org. Override with STANDARD_NET_URL=... if that breaks.

set -euo pipefail
cd "$(dirname "$0")/.."

STANDARD_NET="nets/kata1-b28c512nbt.bin.gz"
HUMAN_NET="nets/b18c384nbt-humanv0.bin.gz"
HUMAN_NET_URL="https://github.com/lightvector/KataGo/releases/download/v1.15.0/b18c384nbt-humanv0.bin.gz"

is_gzip() { [ -s "$1" ] && [ "$(head -c2 "$1" | od -An -tx1 | tr -d ' ')" = "1f8b" ]; }

mkdir -p nets

if ! is_gzip "$STANDARD_NET"; then
    echo "── downloading standard net (~500MB, one time)..."
    URL="${STANDARD_NET_URL:-$(curl -s https://katagotraining.org/ \
        | grep -oE 'https://media\.katagotraining\.org/uploaded/networks/models/kata1/kata1-b28c512nbt[^"]+\.bin\.gz' \
        | head -1)}"
    [ -n "$URL" ] || { echo "error: could not find net URL — set STANDARD_NET_URL or download manually from katagotraining.org to $STANDARD_NET" >&2; exit 1; }
    curl -L -o "$STANDARD_NET" "$URL"
    is_gzip "$STANDARD_NET" || { rm -f "$STANDARD_NET"; echo "error: download failed (not a gzip file)" >&2; exit 1; }
fi

if ! is_gzip "$HUMAN_NET"; then
    echo "── downloading human net (~100MB, one time)..."
    curl -L -o "$HUMAN_NET" "$HUMAN_NET_URL"
    is_gzip "$HUMAN_NET" || { rm -f "$HUMAN_NET"; echo "error: download failed (not a gzip file)" >&2; exit 1; }
fi
