# Quick Deployment Guide

Deploy Spirit Animals Go to your Ubuntu server with RTX 2070 in 3 steps.

## Prerequisites

- Ubuntu server with RTX 2070
- NVIDIA drivers installed (`nvidia-smi` should work)
- Git installed
- Sudo access

## Installation (3 Commands)

```bash
# 1. Clone repository
git clone <your-repo-url> animal_go
cd animal_go

# 2. Run automated deployment (installs everything)
sudo ./scripts/deploy_server.sh

# 3. Access the game
# Open browser: http://<server-ip>:3000
```

The deployment script will:
- ✅ Install Docker and Docker Compose
- ✅ Install NVIDIA Container Toolkit
- ✅ Download neural networks (~600MB)
- ✅ Build Docker image (~10-15 min)
- ✅ Start the server
- ✅ Configure auto-start on boot

## Management

```bash
# View logs
docker-compose logs -f

# Stop server
docker-compose down

# Restart server
docker-compose restart

# Check status
docker-compose ps

# Monitor GPU usage
watch nvidia-smi
```

## Troubleshooting

**Server not accessible from other machines?**
```bash
sudo ufw allow 3000/tcp
```

**Need to rebuild after code changes?**
```bash
git pull
docker-compose up --build -d
```

**Want to see detailed logs?**
```bash
docker-compose logs --tail=100 -f
```

## Performance on RTX 2070

- **Dragon**: ~5-8 seconds per move
- **Mantis Shrimp**: ~25-30 seconds (50k visits)
- **Other spirits**: ~4-6 seconds
- **VRAM usage**: 2-4GB

## Full Documentation

- Complete guide: [DEPLOY.md](DEPLOY.md)
- Docker details: [DOCKER_DEPLOYMENT.md](DOCKER_DEPLOYMENT.md)
- General setup: [SETUP.md](SETUP.md)

## What Gets Deployed

```
Production Container:
├── Rust server (release build)
├── KataGo v1.16.4 (CUDA 12.1)
├── Neural networks (2 models, ~600MB)
├── All 9 spirit configurations
├── Frontend (HTML/CSS/JS)
└── Auto-restart on reboot
```

## Next Steps After Deployment

1. Test all 9 spirits work
2. Set up remote access (optional - see DEPLOY.md)
3. Configure HTTPS with Let's Encrypt (optional)
4. Share with players!

## Support

If the automated script fails, see [DEPLOY.md](DEPLOY.md) for manual installation steps.
