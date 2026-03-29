#!/bin/bash
# Run Spirit Animals Go in dev mode
# Uses lightweight configs for fast iteration

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Check if dev configs exist
if [ ! -d "configs-dev" ]; then
    echo "Dev configs not found. Running setup..."
    ./scripts/setup-dev-mode.sh
fi

# Set environment variable to use dev configs
export ANIMAL_GO_CONFIG_DIR="${PROJECT_DIR}/configs-dev"

echo "=== Running Spirit Animals Go - Dev Mode ==="
echo ""
echo "Config dir: ${ANIMAL_GO_CONFIG_DIR}"
echo "GPU: Quadro P2000 (4GB VRAM)"
echo "Visit counts: 50-200 (fast responses)"
echo ""
echo "Server will start on: http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Run with cargo (debug build for faster compilation during dev)
RUST_LOG=info cargo run
