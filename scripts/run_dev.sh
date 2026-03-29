#!/bin/bash
# Run Animal Go in development mode (laptop - no GPU)
# Uses mock KataGo for fast iteration without neural networks

set -e

cd "$(dirname "$0")/.."

echo "=== Animal Go - Development Mode ==="
echo "Using: Mock KataGo (no GPU required)"
echo ""

# Build image
echo "Building Docker image..."
docker build -t animal-go:dev \
    --build-arg SKIP_NETS=true \
    -f Dockerfile.dev \
    .

echo ""
echo "Starting container..."
docker run -it --rm \
    -p 3000:3000 \
    -e KATAGO_BINARY=/app/scripts/mock_katago.sh \
    -e ANIMAL_GO_CONFIG_DIR=/app/configs-dev \
    -e RUST_LOG=debug \
    -e ENV=development \
    --name animal-go-dev \
    animal-go:dev

echo ""
echo "Container stopped"
