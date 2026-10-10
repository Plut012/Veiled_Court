#!/bin/bash
# Run the coordinator locally
# Serves the frontend on http://localhost:8000 and manages RunPod GPU pods.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VENV="$PROJECT_DIR/coordinator/.venv"

cd "$PROJECT_DIR"

# The coordinator reads RUNPOD_API_KEY from .env (or the environment)
if [ -z "${RUNPOD_API_KEY:-}" ] && ! grep -q "^RUNPOD_API_KEY=" .env 2>/dev/null; then
    echo "RUNPOD_API_KEY not set."
    echo "Add it to $PROJECT_DIR/.env:"
    echo ""
    echo "    RUNPOD_API_KEY=your-key-here"
    echo ""
    exit 1
fi

# Set up venv on first run
if [ ! -d "$VENV" ]; then
    echo "Creating venv..."
    python3 -m venv "$VENV"
    "$VENV/bin/pip" install --quiet -r coordinator/requirements.txt
fi

echo "=== Veiled Court — Coordinator ==="
echo ""
echo "Frontend + pod management: http://localhost:${PORT:-8000}"
echo ""
echo "Press Ctrl+C to stop (stops any running GPU pod on exit)"
echo ""

exec "$VENV/bin/uvicorn" coordinator.main:app --host 0.0.0.0 --port "${PORT:-8000}" "$@"
