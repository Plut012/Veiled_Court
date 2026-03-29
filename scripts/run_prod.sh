#!/bin/bash
# Run Animal Go in production mode (desktop - with GPU)
# Uses real KataGo with CUDA acceleration

set -e

cd "$(dirname "$0")/.."

echo "=== Animal Go - Production Mode ==="
echo "Using: Real KataGo with NVIDIA GPU"
echo ""

# Check if nets directory exists
if [ ! -d "nets" ] || [ -z "$(ls -A nets/*.bin.gz 2>/dev/null)" ]; then
    echo "ERROR: Neural networks not found!"
    echo ""
    echo "Please run: ./scripts/download_nets.sh"
    echo "Then try again."
    exit 1
fi

echo "Neural networks found:"
ls -lh nets/*.bin.gz | awk '{print "  " $9 " (" $5 ")"}'
echo ""

# Check for NVIDIA GPU
if ! command -v nvidia-smi &> /dev/null; then
    echo "WARNING: nvidia-smi not found. GPU may not be available."
fi

echo "Starting with docker-compose..."
docker-compose up --build -d

echo ""
echo "✓ Container started"
echo ""
echo "View logs: docker-compose logs -f"
echo "Stop: docker-compose down"
echo "Status: docker-compose ps"
echo ""
echo "Server running at: http://localhost:3000"
