#!/bin/bash
# ──────────────────────────────────────
#  Veiled Court — Build & Push Docker Image
#  Pushes to GitHub Container Registry for RunPod to pull.
# ──────────────────────────────────────

set -euo pipefail
cd "$(dirname "$0")/.."

source .env

IMAGE="${DOCKER_IMAGE:-ghcr.io/plut012/kitsune:latest}"

echo ""
echo "═══ Building Veiled Court Docker Image ═══"
echo ""
echo "  Image: $IMAGE"
echo ""

# Log into GHCR
echo "  → logging into ghcr.io..."
echo "$GITHUB_PAT" | docker login ghcr.io -u "$GITHUB_USER" --password-stdin 2>/dev/null
echo "  ✓ authenticated"

# Build
echo "  → building image..."
docker build -t "$IMAGE" -f Dockerfile .

echo "  ✓ image built"
echo ""
echo "  → pushing to registry..."
docker push "$IMAGE"

echo "  ✓ pushed: $IMAGE"
echo ""
echo "  Next: ./scripts/runpod-pod.sh start"
echo ""
