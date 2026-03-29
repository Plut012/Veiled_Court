#!/bin/bash
# Download KataGo neural networks
# Run this ONCE on the desktop before building production Docker image

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
NETS_DIR="${PROJECT_DIR}/nets"

cd "$PROJECT_DIR"

echo "=== Downloading KataGo Neural Networks ==="
echo ""
echo "Target: ${NETS_DIR}"
echo ""

mkdir -p "$NETS_DIR"

# IMPORTANT: The URLs below may be outdated!
# Always check https://katagotraining.org/ for the latest standard network
# The URL changes with each new training run

echo "1/2 Downloading standard network (~400MB)..."
echo "    This may take a few minutes..."
echo ""

# Standard net - CHECK THIS URL AT: https://katagotraining.org/
# Look for "Network File" link at top of page
# Example URL (verify before using):
STANDARD_NET_URL="https://media.katagotraining.org/uploaded/networks/models/kata1/kata1-zhizi-b28c512nbt-muonfd2.bin.gz"

# Try downloading with multiple methods
if command -v curl &> /dev/null; then
    curl -L "$STANDARD_NET_URL" -o "${NETS_DIR}/kata1-b28c512nbt.bin.gz" || {
        echo "ERROR: Failed to download standard network"
        echo "Please download manually from: https://katagotraining.org/"
        echo "Save as: ${NETS_DIR}/kata1-b28c512nbt.bin.gz"
        exit 1
    }
elif command -v wget &> /dev/null; then
    wget "$STANDARD_NET_URL" -O "${NETS_DIR}/kata1-b28c512nbt.bin.gz" || {
        echo "ERROR: Failed to download standard network"
        echo "Please download manually from: https://katagotraining.org/"
        echo "Save as: ${NETS_DIR}/kata1-b28c512nbt.bin.gz"
        exit 1
    }
else
    echo "ERROR: Neither curl nor wget found"
    echo "Please install one and try again, or download manually"
    exit 1
fi

echo "✓ Standard network downloaded"
echo ""

echo "2/2 Downloading human style network (~200MB)..."
echo ""

# Human net - stable GitHub releases URL
HUMAN_NET_URL="https://github.com/lightvector/KataGo/releases/download/v1.15.0/b18c384nbt-humanv0.bin.gz"

if command -v curl &> /dev/null; then
    curl -L "$HUMAN_NET_URL" -o "${NETS_DIR}/b18c384nbt-humanv0.bin.gz" || {
        echo "ERROR: Failed to download human network"
        echo "Please download manually from: https://github.com/lightvector/KataGo/releases/tag/v1.15.0"
        echo "Save as: ${NETS_DIR}/b18c384nbt-humanv0.bin.gz"
        exit 1
    }
elif command -v wget &> /dev/null; then
    wget "$HUMAN_NET_URL" -O "${NETS_DIR}/b18c384nbt-humanv0.bin.gz" || {
        echo "ERROR: Failed to download human network"
        echo "Please download manually from: https://github.com/lightvector/KataGo/releases/tag/v1.15.0"
        echo "Save as: ${NETS_DIR}/b18c384nbt-humanv0.bin.gz"
        exit 1
    }
fi

echo "✓ Human style network downloaded"
echo ""

# Verify downloads
echo "=== Download Summary ==="
ls -lh "${NETS_DIR}"/*.bin.gz | awk '{print $9 ": " $5}'
echo ""

# Check file sizes (rough validation)
STANDARD_SIZE=$(stat -f%z "${NETS_DIR}/kata1-b28c512nbt.bin.gz" 2>/dev/null || stat -c%s "${NETS_DIR}/kata1-b28c512nbt.bin.gz")
HUMAN_SIZE=$(stat -f%z "${NETS_DIR}/b18c384nbt-humanv0.bin.gz" 2>/dev/null || stat -c%s "${NETS_DIR}/b18c384nbt-humanv0.bin.gz")

if [ "$STANDARD_SIZE" -lt 100000000 ]; then
    echo "WARNING: Standard network seems too small ($STANDARD_SIZE bytes)"
    echo "Expected ~400MB. Download may have failed."
fi

if [ "$HUMAN_SIZE" -lt 50000000 ]; then
    echo "WARNING: Human network seems too small ($HUMAN_SIZE bytes)"
    echo "Expected ~200MB. Download may have failed."
fi

echo "✓ Neural networks ready for Docker build"
echo ""
echo "Next steps:"
echo "  1. Verify networks were downloaded correctly"
echo "  2. Run: ./scripts/run_prod.sh"
