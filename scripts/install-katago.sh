#!/bin/bash
# Automated KataGo installation with model downloads
# Detects GPU and downloads appropriate version

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "=== KataGo Installation ==="
echo ""

# Detect system
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

echo "System: ${OS} ${ARCH}"

# Detect CUDA
if command -v nvidia-smi &> /dev/null; then
    CUDA_VERSION=$(nvidia-smi | grep -oP 'CUDA Version: \K[0-9]+\.[0-9]+' | head -1)
    GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
    echo "GPU: ${GPU_NAME}"
    echo "CUDA: ${CUDA_VERSION}"
    USE_GPU=true

    # Map CUDA version to KataGo release
    if [[ "$CUDA_VERSION" =~ ^13\. ]]; then
        CUDA_TAG="cuda12.5"  # KataGo doesn't have CUDA 13, use 12.5
    elif [[ "$CUDA_VERSION" =~ ^12\. ]]; then
        CUDA_TAG="cuda12.5"
    elif [[ "$CUDA_VERSION" =~ ^11\. ]]; then
        CUDA_TAG="cuda11.8"
    else
        echo "Warning: Unusual CUDA version, using cuda11.8"
        CUDA_TAG="cuda11.8"
    fi
else
    echo "No CUDA detected - will use CPU version (much slower)"
    USE_GPU=false
    CUDA_TAG="cpu"
fi

echo ""
echo "Installing to: ${PROJECT_DIR}/assets/katago"
echo ""

# Create directory
mkdir -p assets/katago
cd assets/katago

# KataGo version to download
KATAGO_VERSION="v1.16.4"

# Determine download URL based on system
if [ "$USE_GPU" = true ]; then
    if [ "$OS" = "linux" ] && [ "$ARCH" = "x86_64" ]; then
        # CUDA 13.0 -> use CUDA 12.5 build (compatible)
        if [[ "$CUDA_TAG" == "cuda12.5" ]]; then
            BINARY_URL="https://github.com/lightvector/KataGo/releases/download/${KATAGO_VERSION}/katago-${KATAGO_VERSION}-cuda12.5-cudnn8.9.7-linux-x64.zip"
        elif [[ "$CUDA_TAG" == "cuda11.8" ]]; then
            # Try CUDA 12.1 as fallback for older CUDA
            BINARY_URL="https://github.com/lightvector/KataGo/releases/download/${KATAGO_VERSION}/katago-${KATAGO_VERSION}-cuda12.1-cudnn8.9.7-linux-x64.zip"
        else
            BINARY_URL="https://github.com/lightvector/KataGo/releases/download/${KATAGO_VERSION}/katago-${KATAGO_VERSION}-cuda12.5-cudnn8.9.7-linux-x64.zip"
        fi
    else
        echo "Error: Unsupported system for GPU: ${OS} ${ARCH}"
        exit 1
    fi
else
    if [ "$OS" = "linux" ] && [ "$ARCH" = "x86_64" ]; then
        BINARY_URL="https://github.com/lightvector/KataGo/releases/download/${KATAGO_VERSION}/katago-${KATAGO_VERSION}-opencl-linux-x64.zip"
    else
        echo "Error: Unsupported system: ${OS} ${ARCH}"
        exit 1
    fi
fi

# Download binary
echo "1/3 Downloading KataGo binary..."
echo "URL: ${BINARY_URL}"
if ! wget -q --show-progress "$BINARY_URL"; then
    echo "Error: Failed to download KataGo binary"
    exit 1
fi

echo "Extracting..."
unzip -q katago-*.zip
chmod +x katago
rm katago-*.zip

# Test binary
echo "Testing binary..."
./katago version
echo ""

# Download standard network
echo "2/3 Downloading standard network (~400MB)..."
STANDARD_NET="kata1-b28c512nbt-s8965856768-d4917590699.bin.gz"
if ! wget -q --show-progress "https://media.katagotraining.org/uploaded/networks/models/kata1/${STANDARD_NET}"; then
    echo "Error: Failed to download standard network"
    exit 1
fi

echo "✓ Standard network downloaded"
echo ""

# Download human network
echo "3/3 Downloading human style network (~200MB)..."
HUMAN_NET="b18c384nbt-humanv0.bin.gz"
if ! wget -q --show-progress "https://media.katagotraining.org/uploaded/networks/models/human/${HUMAN_NET}"; then
    echo "Error: Failed to download human network"
    exit 1
fi

echo "✓ Human network downloaded"
echo ""

# Create symlinks for easier config references
ln -sf "$STANDARD_NET" kata1-b28c512nbt.gz
ln -sf "$HUMAN_NET" b18c384nbt-humanv0.bin.gz

cd "$PROJECT_DIR"

echo "=== Installation Complete! ==="
echo ""
echo "Installed files:"
echo "  • Binary: assets/katago/katago"
echo "  • Standard net: assets/katago/${STANDARD_NET}"
echo "  • Human net: assets/katago/${HUMAN_NET}"
echo ""
echo "Total size: ~$(du -sh assets/katago | cut -f1)"
echo ""

# Now update config paths
echo "Updating config file paths..."
./scripts/update-config-paths.sh

echo ""
echo "✅ KataGo is ready!"
echo ""
echo "Next steps:"
echo "  ./scripts/run-dev.sh       # Start dev server"
echo "  ./scripts/test-spirit.sh dragon  # Test a spirit"
