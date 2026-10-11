#!/bin/bash
# Run the coordinator locally
# Serves the frontend on http://localhost:8000 and manages RunPod GPU pods.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
VENV="$PROJECT_DIR/coordinator/.venv"

cd "$PROJECT_DIR"

# The coordinator reads RUNPOD_API_KEY and COURT_PASSWORD from .env (or the environment)
for VAR in RUNPOD_API_KEY COURT_PASSWORD; do
    if [ -z "${!VAR:-}" ] && ! grep -q "^$VAR=." .env 2>/dev/null; then
        echo "$VAR not set."
        echo "Add it to $PROJECT_DIR/.env:"
        echo ""
        echo "    RUNPOD_API_KEY=your-key-here"
        echo "    COURT_PASSWORD=the-word-players-speak-to-summon"
        echo ""
        exit 1
    fi
done

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
echo "To share it over HTTPS (free, no account, URL changes per run):"
echo ""
echo "    cloudflared tunnel --url http://localhost:${PORT:-8000}"
echo ""
echo "Press Ctrl+C to stop. The GPU pod stops itself after it has been idle."
echo ""

exec "$VENV/bin/uvicorn" coordinator.main:app --host 0.0.0.0 --port "${PORT:-8000}" "$@"
