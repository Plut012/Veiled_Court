#!/bin/bash
# Quick test of a single spirit's config
# Usage: ./scripts/test-spirit.sh dragon

set -e

SPIRIT=${1:-dragon}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Check if dev configs exist
if [ ! -d "configs-dev" ]; then
    echo "Dev configs not found. Running setup..."
    ./scripts/setup-dev-mode.sh
fi

CONFIG_FILE="configs-dev/${SPIRIT}.cfg"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "Error: Config not found: $CONFIG_FILE"
    echo "Available spirits:"
    ls -1 configs-dev/*.cfg | xargs -n1 basename | sed 's/.cfg$//'
    exit 1
fi

echo "=== Testing Spirit: ${SPIRIT} ==="
echo "Config: ${CONFIG_FILE}"
echo ""

# Check if KataGo is installed
if [ ! -f "assets/katago/katago" ]; then
    echo "⚠ KataGo not installed yet!"
    echo ""
    echo "You have two options:"
    echo ""
    echo "1. Quick test (CPU-only, no install needed):"
    echo "   Download from: https://github.com/lightvector/KataGo/releases"
    echo "   Place binary in: assets/katago/katago"
    echo ""
    echo "2. Full setup:"
    echo "   ./scripts/install-katago.sh"
    exit 1
fi

# Test KataGo with this config
echo "Starting KataGo with ${SPIRIT} config..."
echo "This will test if the config loads correctly."
echo ""
echo "Type 'quit' to exit or 'help' for GTP commands"
echo ""

assets/katago/katago gtp -config "$CONFIG_FILE"
