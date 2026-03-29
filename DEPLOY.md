# Deployment Guide - Ubuntu Server with RTX 2070

Complete step-by-step guide to deploy Spirit Animals Go on your Ubuntu server.

## Prerequisites

- Ubuntu server with RTX 2070
- Git installed
- Root or sudo access
- Internet connection

## Quick Start (5 Commands)

```bash
# 1. Clone the repository
git clone <your-repo-url> animal_go
cd animal_go

# 2. Run the deployment script
sudo ./scripts/deploy_server.sh

# 3. The script will handle everything, including:
#    - Docker installation
#    - NVIDIA Container Toolkit setup
#    - Neural network downloads (~600MB)
#    - Container build and launch

# 4. Access the game
# Server will be running at: http://<server-ip>:3000
```

## Manual Deployment Steps

If you prefer manual setup or the automated script fails:

### Step 1: Clone Repository

```bash
cd /opt  # or wherever you want to install
git clone <your-repo-url> animal_go
cd animal_go
```

### Step 2: Install Docker

```bash
# Update package index
sudo apt-get update

# Install dependencies
sudo apt-get install -y \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

# Add Docker's GPG key
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
    sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg

# Set up repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
  https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Add user to docker group (optional, to run without sudo)
sudo usermod -aG docker $USER
```

### Step 3: Install NVIDIA Container Toolkit

```bash
# Add NVIDIA repository
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | \
    sudo tee /etc/apt/sources.list.d/nvidia-docker.list

# Install toolkit
sudo apt-get update
sudo apt-get install -y nvidia-container-toolkit

# Configure Docker to use NVIDIA runtime
sudo nvidia-ctk runtime configure --runtime=docker
sudo systemctl restart docker

# Verify GPU access
docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi
```

### Step 4: Download Neural Networks

```bash
cd /opt/animal_go  # or wherever you cloned the repo
./scripts/download_nets.sh

# This downloads ~600MB total:
#   - kata1-b28c512nbt.bin.gz (~400MB)
#   - b18c384nbt-humanv0.bin.gz (~200MB)

# Verify downloads
ls -lh nets/
```

### Step 5: Build and Run

```bash
# Build and start the container
./scripts/run_prod.sh

# The container will:
# - Build the Rust application (~10-15 minutes first time)
# - Start the server on port 3000
# - Auto-restart on reboot (unless-stopped policy)

# Verify it's running
docker ps
docker-compose logs -f
```

### Step 6: Test

```bash
# From the server
curl http://localhost:3000

# From your local machine
# Open browser: http://<server-ip>:3000
```

## Firewall Configuration

If you can't access the server from other machines:

```bash
# Allow port 3000 through UFW
sudo ufw allow 3000/tcp
sudo ufw reload

# Or for iptables
sudo iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
sudo iptables-save | sudo tee /etc/iptables/rules.v4
```

## Management Commands

```bash
# View logs
docker-compose logs -f

# Stop the server
docker-compose down

# Restart the server
docker-compose restart

# Check status
docker-compose ps

# Rebuild after code changes
git pull
docker-compose up --build -d

# View resource usage
docker stats animal-go-prod
```

## Auto-Start on Boot

The container is configured with `restart: unless-stopped`, which means:
- ✅ Starts automatically after server reboot
- ✅ Restarts automatically on crash
- ✅ Stays stopped if you manually stop it

No additional systemd service needed!

## Monitoring

### Check if server is running
```bash
docker-compose ps
curl http://localhost:3000
```

### View KataGo GPU usage
```bash
nvidia-smi
watch -n 1 nvidia-smi  # Real-time monitoring
```

### View application logs
```bash
docker-compose logs -f
docker-compose logs --tail=100
```

## Troubleshooting

### Container fails to start

```bash
# Check logs
docker-compose logs

# Common issues:
# 1. Neural networks not found
./scripts/download_nets.sh

# 2. GPU not accessible
nvidia-smi  # Check GPU is working
docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi

# 3. Port already in use
sudo lsof -i :3000
# Change port in docker-compose.yml if needed
```

### Slow performance

```bash
# Check GPU is being used
nvidia-smi

# Check VRAM usage (should be 2-4GB during games)
watch nvidia-smi

# If no GPU usage, check NVIDIA runtime
docker info | grep -i runtime
```

### Out of memory

```bash
# Check VRAM
nvidia-smi

# RTX 2070 has 8GB, should be sufficient
# If issues occur, reduce visit counts in configs/
```

### Updates not reflecting

```bash
# Rebuild from scratch
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

## Remote Access (Optional)

### Using Nginx as Reverse Proxy

```bash
# Install Nginx
sudo apt-get install -y nginx

# Create config
sudo tee /etc/nginx/sites-available/animal-go <<EOF
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
    }
}
EOF

# Enable site
sudo ln -s /etc/nginx/sites-available/animal-go /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

### Adding HTTPS with Let's Encrypt

```bash
# Install certbot
sudo apt-get install -y certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d your-domain.com

# Auto-renewal is configured automatically
```

## Performance Expectations (RTX 2070)

| Spirit | Visit Count | Response Time | VRAM Usage |
|--------|-------------|---------------|------------|
| Dragon | 3,000 | ~5-8s | 2-3GB |
| Mantis Shrimp | 50,000 | ~25-30s | 3-4GB |
| Crane | 2,000 | ~4-6s | 2-3GB |
| Spider | 2,000 | ~4-6s | 2-3GB |
| Eagle | 3,000 | ~5-8s | 2-3GB |
| Lion | 3,000 | ~5-8s | 2-3GB |
| Praying Mantis | 2,000 | ~4-6s | 2-3GB |
| Jaguar (opening) | 500 | ~2-3s | 2GB |
| Jaguar (endgame) | 20,000 | ~15-20s | 3-4GB |
| Crow | 2,000 | ~4-6s | 2-3GB |

## Backup

Important files to backup:
```bash
# Config files
tar -czf animal-go-configs.tar.gz configs/

# Full backup (excluding large files)
tar -czf animal-go-backup.tar.gz \
    --exclude='target' \
    --exclude='nets' \
    --exclude='logs' \
    .
```

Neural networks can be re-downloaded if needed (they're 600MB).

## Uninstall

```bash
# Stop and remove containers
docker-compose down

# Remove images
docker rmi animal_go-animal-go

# Remove project directory
cd ..
sudo rm -rf animal_go

# Optional: Remove Docker and NVIDIA toolkit
sudo apt-get remove --purge -y docker-ce docker-ce-cli nvidia-container-toolkit
```

## Next Steps

After deployment:
1. Test all 9 spirits work correctly
2. Set up monitoring (optional)
3. Configure remote access if needed
4. Share the URL with players!

## Support

If issues persist:
1. Check logs: `docker-compose logs -f`
2. Verify GPU: `nvidia-smi`
3. Test manually: `docker exec -it animal-go-prod /bin/bash`
