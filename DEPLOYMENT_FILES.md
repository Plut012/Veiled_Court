# Deployment Files Reference

All files related to deploying Spirit Animals Go to your Ubuntu server.

## Quick Reference

| File | Purpose | When to Use |
|------|---------|-------------|
| `QUICKSTART_DEPLOY.md` | 3-step quick start | First deployment |
| `DEPLOYMENT_CHECKLIST.md` | Step-by-step checklist | During deployment |
| `DEPLOY.md` | Complete manual guide | Detailed reference |
| `DOCKER_DEPLOYMENT.md` | Docker-specific docs | Docker troubleshooting |
| `scripts/deploy_server.sh` | Automated installer | Main deployment script |

## File Details

### Documentation

**QUICKSTART_DEPLOY.md** - Start here
- 3 simple commands to deploy
- Quick troubleshooting
- Expected performance on RTX 2070

**DEPLOYMENT_CHECKLIST.md** - Use during deployment
- Pre-deployment checks
- Step-by-step verification
- Success criteria
- Troubleshooting common issues

**DEPLOY.md** - Complete reference
- Manual installation steps
- Firewall configuration
- Remote access setup (Nginx, HTTPS)
- Detailed troubleshooting
- Performance expectations

**DOCKER_DEPLOYMENT.md** - Docker deep dive
- Container architecture
- Development vs Production modes
- Docker management commands
- Environment variables
- Performance metrics

### Scripts

**scripts/deploy_server.sh** - Main deployment script
```bash
sudo ./scripts/deploy_server.sh
```
Automated installer that:
- Checks for NVIDIA GPU
- Installs Docker
- Installs NVIDIA Container Toolkit
- Downloads neural networks
- Builds and starts container
- Configures auto-start

**scripts/download_nets.sh** - Download neural networks
```bash
./scripts/download_nets.sh
```
Downloads KataGo networks (~600MB):
- Standard network: kata1-b28c512nbt.bin.gz
- Human style network: b18c384nbt-humanv0.bin.gz

**scripts/run_prod.sh** - Start production server
```bash
./scripts/run_prod.sh
```
Starts server with:
- Real KataGo with GPU
- Full visit counts
- Auto-restart enabled

**scripts/run_dev.sh** - Start development server
```bash
./scripts/run_dev.sh
```
Starts server with:
- Mock KataGo (no GPU needed)
- Reduced visit counts
- Fast iteration

### Configuration Files

**Dockerfile** - Production container
- Base: nvidia/cuda:12.1.0-runtime-ubuntu22.04
- Includes: KataGo v1.16.4, Rust, neural networks
- GPU support: NVIDIA CUDA 12.1

**docker-compose.yml** - Container orchestration
- Port: 3000
- GPU allocation: 1 GPU with CUDA support
- Restart policy: unless-stopped
- Volume: ./logs (optional)

**.gitignore** - Git exclusions
- Excludes: target/, nets/*.bin.gz, logs/
- Neural networks must be downloaded separately

## Deployment Workflow

### First-Time Deployment

1. Read: `QUICKSTART_DEPLOY.md`
2. Use: `DEPLOYMENT_CHECKLIST.md`
3. Run: `sudo ./scripts/deploy_server.sh`
4. Reference: `DEPLOY.md` if issues

### Subsequent Deployments

1. Update code: `git pull`
2. Rebuild: `docker-compose up --build -d`
3. Verify: `docker-compose logs -f`

### Development Testing

1. Use: `./scripts/run_dev.sh` (no GPU needed)
2. Test: Make changes, see results quickly
3. Deploy: When ready, use `./scripts/run_prod.sh`

## File Locations

```
animal_go/
├── Documentation
│   ├── QUICKSTART_DEPLOY.md      ← Start here
│   ├── DEPLOYMENT_CHECKLIST.md   ← During deployment
│   ├── DEPLOY.md                 ← Complete guide
│   ├── DOCKER_DEPLOYMENT.md      ← Docker details
│   └── DEPLOYMENT_FILES.md       ← This file
│
├── Scripts
│   ├── scripts/deploy_server.sh  ← Main installer
│   ├── scripts/download_nets.sh  ← Download networks
│   ├── scripts/run_prod.sh       ← Start production
│   └── scripts/run_dev.sh        ← Start development
│
├── Docker
│   ├── Dockerfile                ← Production image
│   ├── Dockerfile.dev            ← Development image
│   └── docker-compose.yml        ← Orchestration
│
└── Config
    ├── configs/                  ← Production configs (9 spirits)
    ├── configs-dev/              ← Dev configs (reduced visits)
    └── nets/                     ← Neural networks (download separately)
```

## Common Commands Quick Reference

```bash
# Deploy for first time
sudo ./scripts/deploy_server.sh

# Start production server
./scripts/run_prod.sh

# Start development server
./scripts/run_dev.sh

# View logs
docker-compose logs -f

# Stop server
docker-compose down

# Restart server
docker-compose restart

# Check status
docker-compose ps

# Monitor GPU
nvidia-smi
watch -n 1 nvidia-smi

# Update code
git pull
docker-compose up --build -d

# Rebuild from scratch
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

## Getting Help

**For deployment issues:**
1. Check `DEPLOYMENT_CHECKLIST.md` troubleshooting section
2. See `DEPLOY.md` manual installation steps
3. Review logs: `docker-compose logs -f`

**For Docker issues:**
1. See `DOCKER_DEPLOYMENT.md`
2. Test GPU: `docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi`
3. Check runtime: `docker info | grep -i runtime`

**For performance issues:**
1. Check GPU usage: `nvidia-smi`
2. See performance expectations in `DEPLOY.md`
3. Consider reducing visit counts in configs/

## Next Steps

Ready to deploy? Start with [QUICKSTART_DEPLOY.md](QUICKSTART_DEPLOY.md)!
