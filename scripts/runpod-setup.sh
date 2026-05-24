#!/bin/bash
# ──────────────────────────────────────
#  Veiled Court — RunPod Setup
#  Creates network volume, uploads models,
#  and validates a test pod launch.
# ──────────────────────────────────────

set -euo pipefail
cd "$(dirname "$0")/.."

# ── config ──
VOLUME_NAME="kitsune-models"
VOLUME_SIZE_GB=2
DATA_CENTER="US-TX-3"
GPU_TYPE="NVIDIA RTX A5000"
DOCKER_IMAGE="${DOCKER_IMAGE:-ghcr.io/plut012/kitsune:latest}"
VOLUME_MOUNT="/models"

# ── load env ──
if [ ! -f .env ]; then
    echo "ERROR: .env file not found. Add RUNPOD_API_KEY to .env"
    exit 1
fi
source .env

if [ -z "${RUNPOD_API_KEY:-}" ]; then
    echo "ERROR: RUNPOD_API_KEY not set in .env"
    exit 1
fi

API="https://api.runpod.io/graphql"
AUTH="Authorization: Bearer ${RUNPOD_API_KEY}"

# ── helpers ──
gql() {
    local query="$1"
    curl -s "$API" \
        -H "Content-Type: application/json" \
        -H "$AUTH" \
        -d "{\"query\": \"$query\"}"
}

gql_post() {
    curl -s "$API" \
        -H "Content-Type: application/json" \
        -H "$AUTH" \
        -d @-
}

log() { echo "  → $1"; }
ok()  { echo "  ✓ $1"; }
die() { echo "  ✗ $1" >&2; exit 1; }

# ──────────────────────────────────────
#  Step 1: Verify account
# ──────────────────────────────────────
echo ""
echo "═══ Veiled Court — RunPod Setup ═══"
echo ""

log "verifying API key..."
ACCT=$(gql "{ myself { id currentSpendPerHr } }")
USER_ID=$(echo "$ACCT" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['myself']['id'])" 2>/dev/null) \
    || die "API key invalid"
ok "authenticated as $USER_ID"

# ──────────────────────────────────────
#  Step 2: Create Network Volume
# ──────────────────────────────────────
echo ""
log "checking for existing volume '$VOLUME_NAME'..."

EXISTING_VOL=$(gql "{ myself { networkVolumes { id name dataCenterId } } }" \
    | python3 -c "
import json, sys
vols = json.load(sys.stdin)['data']['myself']['networkVolumes']
for v in vols:
    if v['name'] == '$VOLUME_NAME':
        print(v['id'])
        break
" 2>/dev/null || true)

if [ -n "$EXISTING_VOL" ]; then
    ok "volume exists: $EXISTING_VOL"
    VOLUME_ID="$EXISTING_VOL"
else
    log "creating network volume ($VOLUME_SIZE_GB GB in $DATA_CENTER)..."

    VOL_RESULT=$(curl -s "https://rest.runpod.io/v1/networkvolumes" \
        -H "$AUTH" \
        -H "Content-Type: application/json" \
        -d "{
            \"name\": \"$VOLUME_NAME\",
            \"size\": $VOLUME_SIZE_GB,
            \"dataCenterId\": \"$DATA_CENTER\"
        }")

    VOLUME_ID=$(echo "$VOL_RESULT" | python3 -c "
import json, sys
data = json.load(sys.stdin)
print(data.get('id', data.get('networkVolumeId', '')))" 2>/dev/null) \
        || die "failed to create volume: $VOL_RESULT"

    if [ -z "$VOLUME_ID" ]; then
        die "failed to create volume: $VOL_RESULT"
    fi
    ok "volume created: $VOLUME_ID"
fi

# ──────────────────────────────────────
#  Step 3: Launch upload pod
#  (small GPU, just to mount volume and
#   upload the KataGo models)
# ──────────────────────────────────────
echo ""
log "launching upload pod to populate volume..."

UPLOAD_RESULT=$(cat <<QUERY | gql_post
{
    "query": "mutation { podFindAndDeployOnDemand(input: { cloudType: SECURE, gpuCount: 1, gpuTypeId: \"$GPU_TYPE\", volumeInGb: 0, containerDiskInGb: 20, minVcpuCount: 2, minMemoryInGb: 8, name: \"kitsune-upload\", imageName: \"runpod/pytorch:2.1.0-py3.10-cuda11.8.0-devel-ubuntu22.04\", networkVolumeId: \"$VOLUME_ID\", volumeMountPath: \"$VOLUME_MOUNT\", ports: \"22/tcp\", dockerArgs: \"sleep infinity\" }) { id desiredStatus machine { podHostId } } }"
}
QUERY
)

UPLOAD_POD_ID=$(echo "$UPLOAD_RESULT" | python3 -c "
import json, sys
data = json.load(sys.stdin)
if 'errors' in data:
    print('ERROR:' + data['errors'][0]['message'], file=sys.stderr)
    sys.exit(1)
print(data['data']['podFindAndDeployOnDemand']['id'])" 2>/dev/null) \
    || die "failed to launch upload pod: $UPLOAD_RESULT"

ok "upload pod launching: $UPLOAD_POD_ID"

# ── wait for pod to be ready ──
log "waiting for pod to be ready..."
for i in $(seq 1 60); do
    POD_STATUS=$(gql "{ pod(input: { podId: \\\"$UPLOAD_POD_ID\\\" }) { runtime { uptimeInSeconds ports { ip isIpPublic privatePort publicPort type } } } }")

    PUBLIC_IP=$(echo "$POD_STATUS" | python3 -c "
import json, sys
data = json.load(sys.stdin)['data']['pod']
if data.get('runtime') and data['runtime'].get('ports'):
    for port in data['runtime']['ports']:
        if port['privatePort'] == 22 and port['isIpPublic']:
            print(f\"{port['ip']}:{port['publicPort']}\")
            break
" 2>/dev/null || true)

    if [ -n "$PUBLIC_IP" ]; then
        break
    fi
    sleep 5
done

if [ -z "$PUBLIC_IP" ]; then
    die "pod didn't become ready in 5 minutes. Check RunPod console."
fi
ok "pod ready at $PUBLIC_IP"

# ──────────────────────────────────────
#  Step 4: Upload KataGo models
# ──────────────────────────────────────
echo ""
log "uploading KataGo models to network volume..."

SSH_HOST=$(echo "$PUBLIC_IP" | cut -d: -f1)
SSH_PORT=$(echo "$PUBLIC_IP" | cut -d: -f2)

# Check if models exist locally
STANDARD_NET="assets/katago/kata1-b28c512nbt.bin.gz"
HUMAN_NET="assets/katago/b18c384nbt-humanv0.bin.gz"
KATAGO_BIN="assets/katago/katago"

MISSING=""
[ ! -f "$STANDARD_NET" ] && MISSING="$MISSING $STANDARD_NET"
[ ! -f "$HUMAN_NET" ] && MISSING="$MISSING $HUMAN_NET"
[ ! -f "$KATAGO_BIN" ] && MISSING="$MISSING $KATAGO_BIN"

if [ -n "$MISSING" ]; then
    echo ""
    echo "  ⚠ Missing files (upload manually once pod is ready):"
    for f in $MISSING; do
        echo "    - $f"
    done
    echo ""
    echo "  Upload command:"
    echo "    scp -P $SSH_PORT <file> root@$SSH_HOST:$VOLUME_MOUNT/"
    echo ""
    echo "  Then terminate the upload pod:"
    echo "    ./scripts/runpod-pod.sh terminate $UPLOAD_POD_ID"
    echo ""
else
    log "uploading standard net (~400MB)..."
    scp -P "$SSH_PORT" -o StrictHostKeyChecking=no "$STANDARD_NET" "root@$SSH_HOST:$VOLUME_MOUNT/"
    ok "standard net uploaded"

    log "uploading human net (~200MB)..."
    scp -P "$SSH_PORT" -o StrictHostKeyChecking=no "$HUMAN_NET" "root@$SSH_HOST:$VOLUME_MOUNT/"
    ok "human net uploaded"

    log "uploading katago binary..."
    scp -P "$SSH_PORT" -o StrictHostKeyChecking=no "$KATAGO_BIN" "root@$SSH_HOST:$VOLUME_MOUNT/"
    ssh -p "$SSH_PORT" -o StrictHostKeyChecking=no "root@$SSH_HOST" "chmod +x $VOLUME_MOUNT/katago"
    ok "katago binary uploaded"

    # Verify
    log "verifying uploads..."
    ssh -p "$SSH_PORT" -o StrictHostKeyChecking=no "root@$SSH_HOST" "ls -lh $VOLUME_MOUNT/"
    ok "all models on volume"

    # Terminate upload pod
    echo ""
    log "terminating upload pod..."
    gql "mutation { podTerminate(input: { podId: \\\"$UPLOAD_POD_ID\\\" }) }" > /dev/null
    ok "upload pod terminated"
fi

# ──────────────────────────────────────
#  Step 5: Save config
# ──────────────────────────────────────
echo ""
log "saving RunPod config..."

cat > .runpod.env <<EOF
# RunPod deployment config (gitignored)
RUNPOD_VOLUME_ID=$VOLUME_ID
RUNPOD_DATA_CENTER=$DATA_CENTER
RUNPOD_GPU_TYPE=$GPU_TYPE
RUNPOD_DOCKER_IMAGE=$DOCKER_IMAGE
RUNPOD_VOLUME_MOUNT=$VOLUME_MOUNT
EOF

ok "config saved to .runpod.env"

echo ""
echo "═══ Setup Complete ═══"
echo ""
echo "  Volume:  $VOLUME_NAME ($VOLUME_ID)"
echo "  Region:  $DATA_CENTER"
echo "  GPU:     $GPU_TYPE (\$0.27/hr secure)"
echo "  Image:   $DOCKER_IMAGE"
echo ""
echo "  Next steps:"
echo "    1. Build & push Docker image:  ./scripts/runpod-push.sh"
echo "    2. Test pod launch:            ./scripts/runpod-pod.sh start"
echo "    3. Check status:               ./scripts/runpod-pod.sh status"
echo ""
