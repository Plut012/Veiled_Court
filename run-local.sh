#!/bin/bash
# ──────────────────────────────────────
#  Veiled Court — local CPU run (podman/docker)
#
#  One command: ./run-local.sh
#  Builds the CPU image, downloads nets if missing, runs on :3000.
#
#  Options (env vars):
#    MOCK=1        use mock engine (instant random moves, no nets needed)
#    CONFIGS=prod  use full-strength configs (slow on CPU; default: dev)
# ──────────────────────────────────────

set -euo pipefail
cd "$(dirname "$0")"

IMAGE="veiled-court-cpu"
NAME="veiled-court"

# ── container engine ──
if command -v podman &>/dev/null; then
    ENGINE=podman
elif command -v docker &>/dev/null; then
    ENGINE=docker
else
    echo "error: need podman or docker" >&2
    exit 1
fi

# ── nets ──
STANDARD_NET="nets/kata1-b28c512nbt.bin.gz"
HUMAN_NET="nets/b18c384nbt-humanv0.bin.gz"

if [ "${MOCK:-0}" != "1" ]; then
    mkdir -p nets

    if [ ! -s "$STANDARD_NET" ] || file "$STANDARD_NET" | grep -qv gzip; then
        echo "── downloading standard net (~500MB, one time)..."
        # URL rotates with each training run — scrape the current one
        URL=$(curl -s https://katagotraining.org/ \
            | grep -oE 'https://media\.katagotraining\.org/uploaded/networks/models/kata1/kata1-b28c512nbt[^"]+\.bin\.gz' \
            | head -1)
        [ -n "$URL" ] || { echo "error: could not find net URL — download manually from katagotraining.org to $STANDARD_NET" >&2; exit 1; }
        curl -L -o "$STANDARD_NET" "$URL"
        file "$STANDARD_NET" | grep -q gzip || { echo "error: download failed (not a gzip file)" >&2; exit 1; }
    fi

    if [ ! -s "$HUMAN_NET" ]; then
        echo "── downloading human net (~100MB, one time)..."
        curl -L -o "$HUMAN_NET" \
            "https://github.com/lightvector/KataGo/releases/download/v1.15.0/b18c384nbt-humanv0.bin.gz"
    fi
fi

# ── build ──
echo "── building image..."
$ENGINE build -q -f Dockerfile.cpu -t "$IMAGE" .

# ── run ──
$ENGINE rm -f "$NAME" &>/dev/null || true

RUN_ARGS=(-d --name "$NAME" -p 3000:3000)

if [ "${MOCK:-0}" = "1" ]; then
    RUN_ARGS+=(-e KATAGO_BINARY=/app/scripts/mock_katago.sh -e KATAGO_MODEL=/app/kitsune)
else
    RUN_ARGS+=(-v "$(pwd)/nets:/app/models:ro,Z")
fi

if [ "${CONFIGS:-dev}" = "prod" ]; then
    RUN_ARGS+=(-e KITSUNE_CONFIG_DIR=/app/configs)
fi

$ENGINE run "${RUN_ARGS[@]}" "$IMAGE" >/dev/null

echo ""
echo "  veiled court → http://localhost:3000"
echo ""
echo "  logs:  $ENGINE logs -f $NAME"
echo "  stop:  $ENGINE rm -f $NAME"
echo ""
