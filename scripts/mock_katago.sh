#!/bin/bash
# Mock KataGo - GTP-speaking fake for laptop development
# Responds to basic GTP commands without needing GPU or neural networks

set -e

# Track board state
BOARD_SIZE=19
declare -A OCCUPIED_POINTS

# GTP coordinates (columns: A-T skipping I)
COLS="ABCDEFGHJKLMNOPQRST"

# Helper: Check if a point is occupied
is_occupied() {
    local coord="$1"
    [[ -n "${OCCUPIED_POINTS[$coord]}" ]]
}

# Helper: Generate random legal move
generate_random_move() {
    local attempts=0
    local max_attempts=100

    while [ $attempts -lt $max_attempts ]; do
        # Random column (0-18, skip I)
        local col_idx=$((RANDOM % BOARD_SIZE))
        local col=${COLS:$col_idx:1}

        # Random row (1-19)
        local row=$((RANDOM % BOARD_SIZE + 1))

        local coord="${col}${row}"

        # Check if unoccupied
        if ! is_occupied "$coord"; then
            echo "$coord"
            return 0
        fi

        attempts=$((attempts + 1))
    done

    # If all attempts failed, pass
    echo "pass"
}

# Helper: Send GTP response
gtp_response() {
    local status="$1"
    local message="$2"

    if [ -z "$message" ]; then
        echo "$status"
        echo ""
    else
        echo "$status $message"
        echo ""
    fi
}

# Log to stderr (so it doesn't interfere with GTP protocol)
log() {
    echo "[MOCK_KATAGO] $*" >&2
}

log "Mock KataGo started (no GPU, random moves)"
log "Board size: ${BOARD_SIZE}x${BOARD_SIZE}"

# Main GTP command loop
while IFS= read -r line; do
    # Trim whitespace
    line=$(echo "$line" | xargs)

    # Skip empty lines
    [ -z "$line" ] && continue

    # Parse command
    cmd=$(echo "$line" | awk '{print $1}')
    args=$(echo "$line" | cut -d' ' -f2-)

    log "Received: $line"

    case "$cmd" in
        version)
            gtp_response "=" "Mock KataGo v1.0 (dev mode)"
            ;;

        name)
            gtp_response "=" "MockKataGo"
            ;;

        protocol_version)
            gtp_response "=" "2"
            ;;

        boardsize)
            BOARD_SIZE=$(echo "$args" | awk '{print $1}')
            log "Board size set to ${BOARD_SIZE}"
            gtp_response "="
            ;;

        clear_board)
            OCCUPIED_POINTS=()
            log "Board cleared"
            gtp_response "="
            ;;

        komi)
            # Ignore komi, just acknowledge
            gtp_response "="
            ;;

        play)
            # Args: COLOR COORD (e.g., "black D4")
            color=$(echo "$args" | awk '{print $1}')
            coord=$(echo "$args" | awk '{print $2}')

            if [ "$coord" != "pass" ]; then
                OCCUPIED_POINTS[$coord]="$color"
                log "Play: $color $coord"
            fi
            gtp_response "="
            ;;

        genmove)
            # Generate a random legal move
            move=$(generate_random_move)

            if [ "$move" != "pass" ]; then
                OCCUPIED_POINTS[$move]="black"
            fi

            log "Generated move: $move"
            gtp_response "=" "$move"
            ;;

        showboard)
            # Not required but helpful for debugging
            gtp_response "=" ""
            ;;

        quit)
            log "Quit command received"
            gtp_response "="
            exit 0
            ;;

        kata-*)
            # KataGo-specific commands - just acknowledge
            gtp_response "="
            ;;

        *)
            log "Unknown command: $cmd"
            gtp_response "?" "unknown command"
            ;;
    esac
done

log "Mock KataGo exiting"
