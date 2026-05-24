#!/bin/bash
# Run Veiled Court in development mode (laptop - no GPU)
# Uses mock KataGo for fast iteration without neural networks

set -e

cd "$(dirname "$0")/.."

echo "=== Veiled Court — Development Mode ==="
echo "Using: Mock KataGo (no GPU required)"
echo ""

# Build image
echo "Building Docker image..."
docker build -t kitsune:dev \
    --build-arg SKIP_NETS=true \
    -f Dockerfile.dev \
    .

echo ""
echo "Starting container..."
docker run -it --rm \
    -p 3000:3000 \
    -e KATAGO_BINARY=/app/scripts/mock_katago.sh \
    -e KITSUNE_CONFIG_DIR=/app/configs-dev \
    -e RUST_LOG=debug \
    -e ENV=development \
    --name kitsune-dev \
    kitsune:dev

echo ""
echo "Container stopped"
