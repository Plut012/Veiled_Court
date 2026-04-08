#!/bin/bash
# Automated deployment script for Ubuntu server with NVIDIA GPU
# Installs Docker, NVIDIA Container Toolkit, downloads models, and starts the server

set -e

echo "========================================"
echo "Spirit Animals Go - Server Deployment"
echo "========================================"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run with sudo: sudo ./scripts/deploy_server.sh"
    exit 1
fi

# Get the actual user (not root)
ACTUAL_USER="${SUDO_USER:-$USER}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Project directory: $PROJECT_DIR"
echo "Running as user: $ACTUAL_USER"
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Step 1: Check for NVIDIA GPU
echo "=== Step 1: Checking for NVIDIA GPU ==="
if ! command_exists nvidia-smi; then
    echo "ERROR: nvidia-smi not found!"
    echo "Please install NVIDIA drivers first:"
    echo "  sudo ubuntu-drivers autoinstall"
    echo "  sudo reboot"
    exit 1
fi

nvidia-smi --query-gpu=name --format=csv,noheader
echo "✓ NVIDIA GPU detected"
echo ""

# Step 2: Install Docker if needed
echo "=== Step 2: Installing Docker ==="
if command_exists docker; then
    echo "✓ Docker already installed"
    docker --version
else
    echo "Installing Docker..."
    apt-get update
    apt-get install -y ca-certificates curl gnupg lsb-release

    mkdir -p /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
        gpg --dearmor -o /etc/apt/keyrings/docker.gpg

    echo \
      "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
      https://download.docker.com/linux/ubuntu \
      $(lsb_release -cs) stable" | \
      tee /etc/apt/sources.list.d/docker.list > /dev/null

    apt-get update
    apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

    # Add user to docker group
    usermod -aG docker "$ACTUAL_USER"
    echo "✓ Docker installed"
fi

# Ensure Docker starts on boot
systemctl enable docker
echo ""

# Step 3: Install NVIDIA Container Toolkit
echo "=== Step 3: Installing NVIDIA Container Toolkit ==="
if grep -q '"nvidia"' /etc/docker/daemon.json 2>/dev/null; then
    echo "✓ NVIDIA Container Toolkit already configured"
else
    echo "Installing NVIDIA Container Toolkit..."

    curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | \
        gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg
    curl -s -L https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list | \
        sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
        tee /etc/apt/sources.list.d/nvidia-container-toolkit.list

    apt-get update
    apt-get install -y nvidia-container-toolkit

    # Configure Docker runtime
    nvidia-ctk runtime configure --runtime=docker
    systemctl restart docker

    echo "✓ NVIDIA Container Toolkit installed"
fi
echo ""

# Step 4: Download neural networks
echo "=== Step 4: Downloading Neural Networks ==="
cd "$PROJECT_DIR"

MIN_STANDARD_SIZE=100000000  # ~100MB minimum
MIN_HUMAN_SIZE=50000000      # ~50MB minimum
NEEDS_DOWNLOAD=false

STANDARD_NET="${PROJECT_DIR}/nets/kata1-b28c512nbt.bin.gz"
HUMAN_NET="${PROJECT_DIR}/nets/b18c384nbt-humanv0.bin.gz"

mkdir -p "${PROJECT_DIR}/nets"

# Validate standard net
if [ -f "$STANDARD_NET" ]; then
    SIZE=$(stat -c%s "$STANDARD_NET" 2>/dev/null || echo 0)
    if [ "$SIZE" -lt "$MIN_STANDARD_SIZE" ]; then
        echo "⚠ Standard network is too small (${SIZE} bytes) — re-downloading"
        NEEDS_DOWNLOAD=true
    fi
else
    NEEDS_DOWNLOAD=true
fi

# Validate human net
if [ -f "$HUMAN_NET" ]; then
    SIZE=$(stat -c%s "$HUMAN_NET" 2>/dev/null || echo 0)
    if [ "$SIZE" -lt "$MIN_HUMAN_SIZE" ]; then
        echo "⚠ Human network is too small (${SIZE} bytes) — re-downloading"
        NEEDS_DOWNLOAD=true
    fi
else
    NEEDS_DOWNLOAD=true
fi

if [ "$NEEDS_DOWNLOAD" = true ]; then
    echo "Downloading neural networks (this may take a few minutes)..."
    su - "$ACTUAL_USER" -c "cd '$PROJECT_DIR' && ./scripts/download_nets.sh"

    # Verify after download
    for NET in "$STANDARD_NET" "$HUMAN_NET"; do
        if [ ! -f "$NET" ]; then
            echo "ERROR: $NET not found after download"
            exit 1
        fi
        SIZE=$(stat -c%s "$NET")
        if [ "$SIZE" -lt "$MIN_HUMAN_SIZE" ]; then
            echo "ERROR: $NET is only ${SIZE} bytes — download likely failed"
            exit 1
        fi
    done
    echo "✓ Neural networks downloaded and verified"
else
    echo "✓ Neural networks already downloaded"
    ls -lh nets/*.bin.gz | awk '{print "  " $9 " (" $5 ")"}'
fi
echo ""

# Step 5: Configure firewall
echo "=== Step 5: Configuring Firewall ==="
if command_exists ufw; then
    if ufw status | grep -q "Status: active"; then
        echo "Allowing port 3000..."
        ufw allow 3000/tcp
        ufw reload
        echo "✓ Firewall configured (port 3000 open)"
    else
        echo "⚠ UFW not active, skipping firewall configuration"
    fi
else
    echo "⚠ UFW not installed, skipping firewall configuration"
fi
echo ""

# Step 6: Build and start the container
echo "=== Step 6: Building and Starting Server ==="
echo "This will take a few minutes on first run..."
echo ""

cd "$PROJECT_DIR"

# Run docker as root — user may not have docker group membership yet
docker compose up --build -d

echo ""
echo "✓ Server started"
echo ""

# Step 7: Wait for server to be ready
echo "=== Step 7: Waiting for Server to Start ==="
echo "Checking server health..."

MAX_RETRIES=30
RETRY_COUNT=0
SERVER_READY=false

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s http://localhost:3000 >/dev/null 2>&1; then
        SERVER_READY=true
        break
    fi
    echo -n "."
    sleep 2
    RETRY_COUNT=$((RETRY_COUNT + 1))
done
echo ""

if [ "$SERVER_READY" = true ]; then
    echo "✓ Server is responding"
else
    echo "⚠ Server not responding yet (may still be loading GPU model)"
    echo "Check logs with: docker compose logs -f"
fi
echo ""

# Step 8: Display summary
echo "========================================"
echo "✓ Deployment Complete!"
echo "========================================"
echo ""
echo "Server Status:"
docker compose ps
echo ""
echo "Access the game at:"
echo "  Local: http://localhost:3000"
IP_ADDR=$(hostname -I | awk '{print $1}')
if [ -n "$IP_ADDR" ]; then
    echo "  Network: http://$IP_ADDR:3000"
fi
echo ""
echo "Management Commands:"
echo "  View logs:    docker compose logs -f"
echo "  Stop server:  docker compose down"
echo "  Restart:      docker compose restart"
echo "  Status:       docker compose ps"
echo "  GPU usage:    nvidia-smi"
echo ""
echo "Note: If you were added to the docker group for the first time,"
echo "you may need to log out and back in to use docker without sudo."
echo ""
