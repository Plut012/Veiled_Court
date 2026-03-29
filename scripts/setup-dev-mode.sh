#!/bin/bash
# Create lightweight configs for laptop development
# Optimized for Quadro P2000 (4GB VRAM)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Setting up Dev Mode for Spirit Animals Go ==="
echo "Target: Quadro P2000 (4GB VRAM)"
echo ""

# Create dev configs directory
mkdir -p "${PROJECT_DIR}/configs-dev"

echo "Creating lightweight configs..."

# Dev mode settings:
# - maxVisits: 50-200 (vs 2000-50000 in prod)
# - numSearchThreads: 2 (vs 4-8 in prod)
# - Skip human net for most spirits (save VRAM)
# - Smaller board default

for spirit_file in "${PROJECT_DIR}/configs"/*.cfg; do
    spirit_name=$(basename "$spirit_file" .cfg)
    dev_file="${PROJECT_DIR}/configs-dev/${spirit_name}.cfg"

    echo "  Creating ${spirit_name}.cfg..."

    # Copy base config
    cp "$spirit_file" "$dev_file"

    # Reduce visits based on spirit
    case "$spirit_name" in
        mantis_shrimp)
            # Mantis Shrimp: 50000+ → 200 (still highest)
            sed -i 's/maxVisits = [0-9]*/maxVisits = 200/' "$dev_file"
            ;;
        jaguar)
            # Jaguar: Keep low for opening, but cap endgame at 500
            sed -i 's/maxVisits = [0-9]*/maxVisits = 100/' "$dev_file"
            ;;
        *)
            # All others: 2000-5000 → 50
            sed -i 's/maxVisits = [0-9]*/maxVisits = 50/' "$dev_file"
            ;;
    esac

    # Reduce threads
    sed -i 's/numSearchThreads = [0-9]*/numSearchThreads = 2/' "$dev_file"

    # Comment out human model to save VRAM (except for one test spirit)
    if [ "$spirit_name" != "dragon" ]; then
        sed -i 's/^humanModelFile = /#humanModelFile = /' "$dev_file"
        sed -i 's/^humanSLProfile = /#humanSLProfile = /' "$dev_file"
    fi
done

echo ""
echo "✓ Dev configs created in configs-dev/"
echo ""
echo "Dev mode settings:"
echo "  • Visit counts: 50-200 (vs 2000-50000)"
echo "  • Threads: 2 (vs 4-8)"
echo "  • Human net: Dragon only (saves ~1GB VRAM)"
echo "  • Response time: ~0.5-2 seconds"
echo ""
echo "To run in dev mode:"
echo "  ./scripts/run-dev.sh"
echo ""
echo "To run specific spirit test:"
echo "  SPIRIT=dragon ./scripts/run-dev.sh"
