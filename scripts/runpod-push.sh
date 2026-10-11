#!/bin/bash
# ──────────────────────────────────────
#  Veiled Court — Build & Push the GPU pod image
#  Pushes to GitHub Container Registry for RunPod to pull.
#  Only needed when src/ or configs/ change (the frontend lives on the coordinator).
# ──────────────────────────────────────

set -euo pipefail
cd "$(dirname "$0")/.."

[ -f .env ] && source .env

IMAGE="${DOCKER_IMAGE:-ghcr.io/plut012/kitsune:latest}"

if command -v podman &>/dev/null; then
    ENGINE=podman
elif command -v docker &>/dev/null; then
    ENGINE=docker
else
    echo "error: need podman or docker" >&2
    exit 1
fi

./scripts/download_nets.sh

# Log into ghcr.io: GITHUB_PAT from .env, else the gh CLI's token
# (needs: gh auth refresh -h github.com -s write:packages), else an existing login
if [ -n "${GITHUB_PAT:-}" ] && [ -n "${GITHUB_USER:-}" ]; then
    echo "── logging into ghcr.io..."
    echo "$GITHUB_PAT" | $ENGINE login ghcr.io -u "$GITHUB_USER" --password-stdin >/dev/null
elif command -v gh &>/dev/null && gh auth status &>/dev/null; then
    echo "── logging into ghcr.io via gh..."
    gh auth token | $ENGINE login ghcr.io -u "$(gh api user -q .login)" --password-stdin >/dev/null
fi

echo "── building $IMAGE..."
$ENGINE build -t "$IMAGE" -f Dockerfile.runpod .

echo "── pushing..."
$ENGINE push "$IMAGE"

echo ""
echo "  pushed: $IMAGE"
echo ""
echo "  A stopped pod may resume with its cached image. To be sure it runs this build,"
echo "  terminate the 'kitsune-game' pod in the RunPod console — the next"
echo "  summon creates a fresh one."
echo ""
