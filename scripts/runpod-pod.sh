#!/bin/bash
# ──────────────────────────────────────
#  Veiled Court — RunPod Pod Management
#  Usage: ./scripts/runpod-pod.sh [start|status|stop|terminate]
# ──────────────────────────────────────

set -euo pipefail
cd "$(dirname "$0")/.."

source .env

API="https://api.runpod.io/graphql"
AUTH="Authorization: Bearer ${RUNPOD_API_KEY}"
POD_ID_FILE=".runpod-pod-id"
IMAGE="${DOCKER_IMAGE:-ghcr.io/plut012/kitsune:latest}"
GPU_TYPE="${RUNPOD_GPU_TYPE:-NVIDIA GeForce RTX 3090}"

gql() {
    curl -s "$API" \
        -H "Content-Type: application/json" \
        -H "$AUTH" \
        -d "{\"query\": \"$1\"}"
}

get_pod_id() {
    if [ -n "${2:-}" ]; then
        echo "$2"
    elif [ -f "$POD_ID_FILE" ]; then
        cat "$POD_ID_FILE"
    else
        echo "No pod ID. Start a pod first or pass ID as argument." >&2
        exit 1
    fi
}

case "${1:-help}" in

start)
    echo "  → launching game pod ($GPU_TYPE)..."
    echo "  → image: $IMAGE"

    RESULT=$(curl -s "$API" \
        -H "Content-Type: application/json" \
        -H "$AUTH" \
        -d "{\"query\": \"mutation { podFindAndDeployOnDemand(input: { cloudType: COMMUNITY, gpuCount: 1, gpuTypeId: \\\"$GPU_TYPE\\\", volumeInGb: 0, containerDiskInGb: 15, minVcpuCount: 2, minMemoryInGb: 8, name: \\\"kitsune-game\\\", imageName: \\\"$IMAGE\\\", ports: \\\"3000/http\\\", env: [{ key: \\\"RUST_LOG\\\", value: \\\"info\\\" }] }) { id desiredStatus } }\"}")

    POD_ID=$(echo "$RESULT" | python3 -c "
import json, sys
data = json.load(sys.stdin)
if 'errors' in data:
    print('ERROR: ' + data['errors'][0]['message'], file=sys.stderr)
    sys.exit(1)
print(data['data']['podFindAndDeployOnDemand']['id'])")

    if [ $? -ne 0 ]; then
        echo "  ✗ failed to start pod"
        echo "$RESULT" | python3 -m json.tool 2>/dev/null
        exit 1
    fi

    echo "$POD_ID" > "$POD_ID_FILE"
    echo "  ✓ pod queued: $POD_ID"
    echo "  → waiting for pod to be ready (may take 60-90s)..."

    for i in $(seq 1 40); do
        STATUS=$(gql "{ pod(input: { podId: \\\"$POD_ID\\\" }) { runtime { uptimeInSeconds ports { ip isIpPublic privatePort publicPort type } } } }")
        URL=$(echo "$STATUS" | python3 -c "
import json, sys
data = json.load(sys.stdin)['data']['pod']
if data.get('runtime') and data['runtime'].get('ports'):
    for port in data['runtime']['ports']:
        if port['privatePort'] == 3000 and port['isIpPublic']:
            print(f\"https://{port['ip']}:{port['publicPort']}\")
            break
" 2>/dev/null)

        if [ -n "$URL" ]; then
            echo "  ✓ pod running: $URL"
            echo ""
            echo "  Waiting for game server to be healthy..."
            # Poll /health until the server is ready
            for j in $(seq 1 30); do
                if curl -sf "$URL/health" > /dev/null 2>&1; then
                    echo "  ✓ game server ready!"
                    echo ""
                    echo "  Play: $URL"
                    exit 0
                fi
                sleep 3
            done
            echo "  ⚠ pod running but server not healthy yet"
            echo "  URL: $URL"
            exit 0
        fi
        sleep 5
    done
    echo "  ⚠ pod queued but not ready yet (low GPU stock)."
    echo "  Check: ./scripts/runpod-pod.sh status"
    ;;

status)
    POD_ID=$(get_pod_id "$@")
    echo "  → checking pod $POD_ID..."

    STATUS=$(gql "{ pod(input: { podId: \\\"$POD_ID\\\" }) { id name desiredStatus runtime { uptimeInSeconds ports { ip isIpPublic privatePort publicPort type } } } }")

    echo "$STATUS" | python3 -c "
import json, sys
data = json.load(sys.stdin)['data']['pod']
if not data:
    print('  ✗ pod not found')
    sys.exit(1)
print(f\"  Pod:    {data['name']} ({data['id']})\")
print(f\"  Status: {data['desiredStatus']}\")
if data.get('runtime'):
    uptime = data['runtime'].get('uptimeInSeconds', 0)
    mins = uptime // 60
    print(f\"  Uptime: {mins}m {uptime % 60}s\")
    if data['runtime'].get('ports'):
        for port in data['runtime']['ports']:
            if port['isIpPublic'] and port['privatePort'] == 3000:
                print(f\"  URL:    https://{port['ip']}:{port['publicPort']}\")
else:
    print('  Runtime: not yet assigned (queued)')
"
    ;;

stop)
    POD_ID=$(get_pod_id "$@")
    echo "  → stopping pod $POD_ID..."
    gql "mutation { podStop(input: { podId: \\\"$POD_ID\\\" }) { id desiredStatus } }" > /dev/null
    echo "  ✓ pod stopped"
    ;;

terminate)
    POD_ID=$(get_pod_id "$@")
    echo "  → terminating pod $POD_ID..."
    gql "mutation { podTerminate(input: { podId: \\\"$POD_ID\\\" }) }" > /dev/null
    rm -f "$POD_ID_FILE"
    echo "  ✓ pod terminated"
    ;;

help|*)
    echo "Usage: ./scripts/runpod-pod.sh [command]"
    echo ""
    echo "Commands:"
    echo "  start       Launch a new game pod"
    echo "  status      Check current pod status"
    echo "  stop        Stop pod"
    echo "  terminate   Destroy pod completely"
    echo ""
    echo "Config (via .env):"
    echo "  RUNPOD_GPU_TYPE  GPU to use (default: NVIDIA GeForce RTX 3090)"
    echo "  DOCKER_IMAGE     Image to deploy (default: ghcr.io/plut012/kitsune:latest)"
    echo ""
    ;;
esac
