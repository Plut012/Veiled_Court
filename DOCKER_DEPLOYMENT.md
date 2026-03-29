# Docker Deployment Guide

Complete guide for containerized deployment of Spirit Animals Go.

## Overview

Two deployment modes:
- **Development** (laptop): Mock KataGo, no GPU required
- **Production** (desktop): Real KataGo with NVIDIA GPU

## Quick Start

### Development Mode (Laptop)

```bash
# No setup needed - mock KataGo works out of the box
./scripts/run_dev.sh

# Server runs on http://localhost:3000
```

### Production Mode (Desktop)

```bash
# One-time setup: Download neural networks (~600MB)
./scripts/download_nets.sh

# Start server
./scripts/run_prod.sh

# Server runs on http://localhost:3000
```

## Detailed Setup

### Prerequisites

**Development:**
- Docker installed
- No GPU required

**Production:**
- Docker + Docker Compose installed
- NVIDIA GPU (tested with RTX 2070, Quadro P2000)
- NVIDIA Container Toolkit installed
- Neural networks downloaded (~600MB)

### Installing NVIDIA Container Toolkit

```bash
# Ubuntu/Debian
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | \
    sudo tee /etc/apt/sources.list.d/nvidia-docker.list

sudo apt-get update
sudo apt-get install -y nvidia-container-toolkit
sudo systemctl restart docker
```

Verify:
```bash
docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi
```

### Downloading Neural Networks

**Important:** Networks must be downloaded BEFORE building the production image.

```bash
cd /data/dev/animal_go

# Download both networks (~600MB total)
./scripts/download_nets.sh

# Verify
ls -lh nets/
# Should show:
#   kata1-b28c512nbt.bin.gz (~400MB)
#   b18c384nbt-humanv0.bin.gz (~200MB)
```

**If download fails:**
1. Open browser and go to: https://katagotraining.org/
2. Download latest `kata1-b28c512nbt-*.bin.gz`
3. Save to: `nets/kata1-b28c512nbt.bin.gz`
4. Download from: https://github.com/lightvector/KataGo/releases/tag/v1.15.0
5. Get `b18c384nbt-humanv0.bin.gz`
6. Save to: `nets/b18c384nbt-humanv0.bin.gz`

## Running

### Development Mode

```bash
./scripts/run_dev.sh
```

**What it does:**
- Builds lightweight Docker image (no GPU support)
- Uses `mock_katago.sh` for instant responses
- Uses `configs-dev/` for reduced visit counts
- Runs on port 3000

**Characteristics:**
- Build time: ~5-10 minutes (first time)
- Response time: ~instant (mock moves)
- VRAM usage: 0 (no GPU)
- Good for: UI development, testing game flow

### Production Mode

```bash
./scripts/run_prod.sh
```

**What it does:**
- Builds full Docker image with CUDA support
- Copies neural networks into image
- Runs real KataGo with GPU acceleration
- Uses full `configs/` with production visit counts
- Auto-restarts on desktop reboot

**Characteristics:**
- Build time: ~10-15 minutes (first time)
- Response time: 0.5-30 seconds (depends on spirit)
- VRAM usage: 2-4GB
- Good for: Real gameplay, final testing

## Managing Containers

### Development

```bash
# Stop (Ctrl+C in terminal)

# View logs
docker logs animal-go-dev

# Remove container
docker rm animal-go-dev
```

### Production

```bash
# Start
./scripts/run_prod.sh

# Stop
docker-compose down

# View logs
docker-compose logs -f

# Restart
docker-compose restart

# Status
docker-compose ps

# Rebuild after code changes
docker-compose up --build -d
```

## Architecture

### Development Container

```
Dockerfile.dev
├── Base: rust:1.75-slim
├── No GPU support
├── Includes: Rust app + configs-dev + mock_katago.sh
└── Excludes: Neural networks
```

### Production Container

```
Dockerfile
├── Base: nvidia/cuda:12.1.0-runtime-ubuntu22.04
├── GPU: NVIDIA with CUDA 12.1
├── Includes: Rust app + configs + real KataGo + neural nets
└── Auto-restart: unless-stopped
```

### File Locations (in container)

```
/app/
├── target/release/animal_go         # Rust binary
├── configs/                          # Production spirit configs
├── configs-dev/                      # Dev spirit configs (fast)
├── frontend/                         # Static HTML/CSS/JS
├── nets/                             # Neural networks (prod only)
│   ├── kata1-b28c512nbt.bin.gz
│   └── b18c384nbt-humanv0.bin.gz
├── scripts/
│   └── mock_katago.sh                # Mock for dev mode
└── /usr/local/bin/katago             # Real KataGo (prod only)
```

## Environment Variables

### Development
```bash
KATAGO_BINARY=/app/scripts/mock_katago.sh
ANIMAL_GO_CONFIG_DIR=/app/configs-dev
RUST_LOG=debug
ENV=development
```

### Production
```bash
KATAGO_BINARY=/usr/local/bin/katago
KATAGO_NETS_PATH=/app/nets
ANIMAL_GO_CONFIG_DIR=/app/configs
RUST_LOG=info
ENV=production
```

## Troubleshooting

### "Neural networks not found"

```bash
# Download them first
./scripts/download_nets.sh

# Or download manually and place in nets/
```

### "NVIDIA runtime not found"

```bash
# Install NVIDIA Container Toolkit
sudo apt-get install -y nvidia-container-toolkit
sudo systemctl restart docker
```

### "Could not initialize CUDA"

```bash
# Check GPU is available
nvidia-smi

# Verify Docker can access GPU
docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi
```

### Build is slow

- First build downloads Rust dependencies (~5-10 min)
- Subsequent builds use cached layers (much faster)
- To force rebuild: `docker-compose build --no-cache`

### Port 3000 already in use

```bash
# Find what's using it
sudo lsof -i :3000

# Or change port in docker-compose.yml:
ports:
  - "3001:3000"
```

### Mock KataGo not responding

```bash
# Check if script is executable
chmod +x scripts/mock_katago.sh

# Test directly
echo "version" | ./scripts/mock_katago.sh
```

## Performance

### Dev Mode (Mock)
- Response time: <100ms
- VRAM: 0
- CPU: minimal
- Disk: ~2GB (image)

### Prod Mode (Real)
- Response time: 0.5-30s (spirit dependent)
- VRAM: 2-4GB
- CPU: medium (MCTS search)
- Disk: ~4GB (image + nets)

## Updating

### Code Changes

```bash
# Development
# Just rebuild and rerun
./scripts/run_dev.sh

# Production
docker-compose up --build -d
```

### New Neural Networks

```bash
# Download new nets
rm nets/*.bin.gz
./scripts/download_nets.sh

# Rebuild image
docker-compose build
docker-compose up -d
```

### KataGo Version

Edit `Dockerfile` line 14:
```dockerfile
RUN wget -q https://github.com/lightvector/KataGo/releases/download/v1.XX.X/katago-...
```

Then rebuild.

## Production Deployment

### Auto-start on Boot

The `docker-compose.yml` includes `restart: unless-stopped`, which means:
- Container starts automatically after desktop reboot
- Stays running until explicitly stopped
- Restarts on crash

### Monitoring

```bash
# Check status
docker-compose ps

# Watch logs
docker-compose logs -f

# Resource usage
docker stats animal-go-prod
```

### Backup

Important files to backup:
- `configs/` - Spirit configurations
- `frontend/` - UI files
- `src/` - Source code
- `nets/` - Neural networks (large, re-downloadable)

## Next Steps

1. **Test development mode:**
   ```bash
   ./scripts/run_dev.sh
   # Open http://localhost:3000
   # Play a few moves to verify mock works
   ```

2. **Setup production:**
   ```bash
   ./scripts/download_nets.sh
   ./scripts/run_prod.sh
   # Test all 9 spirits with real KataGo
   ```

3. **Deploy to desktop:**
   - Copy project to desktop
   - Run `./scripts/download_nets.sh` once
   - Run `./scripts/run_prod.sh`
   - Container will auto-restart on reboot
