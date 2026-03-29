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
echo ""

# Step 3: Install NVIDIA Container Toolkit
echo "=== Step 3: Installing NVIDIA Container Toolkit ==="
if docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi &>/dev/null 2>&1; then
    echo "✓ NVIDIA Container Toolkit already working"
else
    echo "Installing NVIDIA Container Toolkit..."

    # Modern method for adding NVIDIA repository (works with Ubuntu 20.04+)
    distribution=$(. /etc/os-release;echo $ID$VERSION_ID) \
        && curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | \
           gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg \
        && curl -s -L https://nvidia.github.io/libnvidia-container/$distribution/libnvidia-container.list | \
           sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
           tee /etc/apt/sources.list.d/nvidia-container-toolkit.list

    apt-get update
    apt-get install -y nvidia-container-toolkit

    # Configure Docker runtime
    nvidia-ctk runtime configure --runtime=docker
    systemctl restart docker

    echo "✓ NVIDIA Container Toolkit installed"
fi

# Verify GPU access
echo "Testing GPU access in Docker..."
docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi >/dev/null 2>&1
echo "✓ GPU accessible from Docker"
echo ""

# Step 4: Download neural networks
echo "=== Step 4: Downloading Neural Networks (~600MB) ==="
cd "$PROJECT_DIR"

if [ -d "nets" ] && [ -n "$(ls -A nets/*.bin.gz 2>/dev/null)" ]; then
    echo "✓ Neural networks already downloaded"
    ls -lh nets/*.bin.gz | awk '{print "  " $9 " (" $5 ")"}'
else
    echo "Downloading neural networks (this may take a few minutes)..."
    # Run as the actual user, not root
    su - "$ACTUAL_USER" -c "cd '$PROJECT_DIR' && ./scripts/download_nets.sh"
    echo "✓ Neural networks downloaded"
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
echo "This will take 10-15 minutes on first run..."
echo ""

cd "$PROJECT_DIR"

# Run as the actual user
su - "$ACTUAL_USER" -c "cd '$PROJECT_DIR' && docker-compose up --build -d"

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
    echo "⚠ Server not responding yet (may still be starting up)"
    echo "Check logs with: docker-compose logs -f"
fi
echo ""

# Step 8: Display summary
echo "========================================"
echo "✓ Deployment Complete!"
echo "========================================"
echo ""
echo "Server Status:"
docker-compose ps
echo ""
echo "Access the game at:"
echo "  Local: http://localhost:3000"
IP_ADDR=$(hostname -I | awk '{print $1}')
if [ -n "$IP_ADDR" ]; then
    echo "  Network: http://$IP_ADDR:3000"
fi
echo ""
echo "Management Commands:"
echo "  View logs:    docker-compose logs -f"
echo "  Stop server:  docker-compose down"
echo "  Restart:      docker-compose restart"
echo "  Status:       docker-compose ps"
echo "  GPU usage:    nvidia-smi"
echo ""
echo "The server will auto-start on reboot."
echo ""
echo "Note: If you were added to the docker group for the first time,"
echo "you may need to log out and back in to use docker without sudo."
echo ""
