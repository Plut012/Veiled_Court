#!/bin/bash
# Update all config files with correct absolute paths to models

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Find the actual model filenames
STANDARD_NET=$(ls assets/katago/kata1-b28c512nbt*.bin.gz 2>/dev/null | head -1)
HUMAN_NET=$(ls assets/katago/b18c384nbt-humanv0*.bin.gz 2>/dev/null | head -1)

if [ -z "$STANDARD_NET" ]; then
    echo "Error: Standard network not found in assets/katago/"
    echo "Run ./scripts/install-katago.sh first"
    exit 1
fi

echo "Updating config paths..."
echo "Standard net: $STANDARD_NET"
echo "Human net: $HUMAN_NET"
echo ""

# Update production configs
for cfg in configs/*.cfg; do
    if [ -f "$cfg" ]; then
        # Update standard net path (all configs have this)
        sed -i "s|nnModelFile = .*|nnModelFile = ${STANDARD_NET}|" "$cfg"

        # Update human net path (only some configs have this)
        if grep -q "^humanModelFile" "$cfg" 2>/dev/null; then
            if [ -n "$HUMAN_NET" ]; then
                sed -i "s|humanModelFile = .*|humanModelFile = ${HUMAN_NET}|" "$cfg"
            fi
        fi

        echo "✓ Updated $(basename $cfg)"
    fi
done

# Update dev configs if they exist
if [ -d "configs-dev" ]; then
    echo ""
    echo "Updating dev configs..."
    for cfg in configs-dev/*.cfg; do
        if [ -f "$cfg" ]; then
            # Update standard net path
            sed -i "s|nnModelFile = .*|nnModelFile = ${STANDARD_NET}|" "$cfg"

            # Update human net path if present
            if grep -q "^humanModelFile" "$cfg" 2>/dev/null; then
                if [ -n "$HUMAN_NET" ]; then
                    sed -i "s|humanModelFile = .*|humanModelFile = ${HUMAN_NET}|" "$cfg"
                fi
            fi

            echo "✓ Updated $(basename $cfg)"
        fi
    done
fi

echo ""
echo "✓ All config paths updated!"
