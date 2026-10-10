#!/bin/bash
# Deploy the coordinator to an Oracle Cloud (or any Ubuntu) VM.
#
# Usage: ./scripts/deploy-oracle.sh <vm-ip> [ssh-user]
#
# Syncs coordinator + frontend + .env to the VM, sets up a venv,
# installs a systemd service on port 80, and opens the host firewall.
# Idempotent — rerun it to redeploy after changes.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

VM_IP="${1:?usage: ./scripts/deploy-oracle.sh <vm-ip> [ssh-user]}"
VM_USER="${2:-ubuntu}"
REMOTE="$VM_USER@$VM_IP"
APP_DIR="/home/$VM_USER/veiled-court"

cd "$PROJECT_DIR"

if ! grep -q "^RUNPOD_API_KEY=" .env 2>/dev/null; then
    echo "RUNPOD_API_KEY not found in $PROJECT_DIR/.env — create it first:"
    echo ""
    echo "    RUNPOD_API_KEY=your-key-here"
    echo ""
    exit 1
fi

echo "=== Deploying coordinator to $REMOTE ==="
echo ""

echo "--- Syncing files..."
rsync -az --delete \
    --exclude '.venv' --exclude '__pycache__' \
    coordinator frontend "$REMOTE:$APP_DIR/"
rsync -az .env "$REMOTE:$APP_DIR/.env"

echo "--- Setting up server..."
ssh "$REMOTE" APP_DIR="$APP_DIR" VM_USER="$VM_USER" 'bash -s' <<'REMOTE_SCRIPT'
set -e

# Python venv (first run only)
if ! dpkg -s python3-venv >/dev/null 2>&1; then
    sudo apt-get update -qq
    sudo apt-get install -y -qq python3-venv
fi
if [ ! -d "$APP_DIR/coordinator/.venv" ]; then
    python3 -m venv "$APP_DIR/coordinator/.venv"
fi
"$APP_DIR/coordinator/.venv/bin/pip" install --quiet -r "$APP_DIR/coordinator/requirements.txt"

# Oracle's Ubuntu images reject all inbound traffic except SSH — open port 80
if ! sudo iptables -C INPUT -p tcp --dport 80 -j ACCEPT 2>/dev/null; then
    sudo iptables -I INPUT -p tcp --dport 80 -j ACCEPT
    sudo netfilter-persistent save 2>/dev/null || true
fi

# Systemd service
sudo tee /etc/systemd/system/veiled-court.service >/dev/null <<UNIT
[Unit]
Description=Veiled Court coordinator
After=network-online.target
Wants=network-online.target

[Service]
User=$VM_USER
WorkingDirectory=$APP_DIR
ExecStart=$APP_DIR/coordinator/.venv/bin/uvicorn coordinator.main:app --host 0.0.0.0 --port 80
AmbientCapabilities=CAP_NET_BIND_SERVICE
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
UNIT

sudo systemctl daemon-reload
sudo systemctl enable veiled-court >/dev/null 2>&1
sudo systemctl restart veiled-court
REMOTE_SCRIPT

echo "--- Waiting for service..."
sleep 3
STATUS=$(curl -sf -m 10 "http://$VM_IP/status" || true)
if [ -n "$STATUS" ]; then
    echo ""
    echo "=== Deployed ==="
    echo ""
    echo "  Status:   $STATUS"
    echo "  Entrance: http://$VM_IP/"
    echo ""
    echo "  Logs:     ssh $REMOTE journalctl -u veiled-court -f"
else
    echo ""
    echo "Service not reachable from here yet."
    echo "If it works via  ssh $REMOTE curl -s localhost/status  but not externally,"
    echo "add an ingress rule for TCP port 80 in the OCI console (see docs/deploy-oracle.md)."
    exit 1
fi
