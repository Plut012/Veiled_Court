# Deployment Checklist

Use this checklist when deploying to your Ubuntu server with RTX 2070.

## Pre-Deployment (On Your Local Machine)

- [ ] Code is committed and pushed to git repository
- [ ] All scripts have execute permissions (already set)
- [ ] Docker files are present (Dockerfile, docker-compose.yml)
- [ ] Config files are present (configs/ directory with 9 .cfg files)

## Server Requirements

- [ ] Ubuntu server (tested on 20.04, 22.04)
- [ ] NVIDIA RTX 2070 GPU installed
- [ ] NVIDIA drivers installed (test with `nvidia-smi`)
- [ ] Internet connection (for downloading dependencies)
- [ ] Sudo/root access
- [ ] At least 10GB free disk space

## Deployment Steps

### 1. Initial Setup

```bash
# SSH into your server
ssh user@your-server-ip

# Update system (recommended)
sudo apt-get update
sudo apt-get upgrade -y

# Install git if not present
sudo apt-get install -y git

# Check NVIDIA drivers
nvidia-smi  # Should show your RTX 2070
```

- [ ] Connected to server
- [ ] System updated
- [ ] Git installed
- [ ] NVIDIA drivers working

### 2. Clone Repository

```bash
# Clone to /opt or your preferred location
cd /opt
sudo git clone <your-repo-url> animal_go
sudo chown -R $USER:$USER animal_go
cd animal_go
```

- [ ] Repository cloned
- [ ] Permissions set correctly

### 3. Run Automated Deployment

```bash
# This is the main step - does everything
sudo ./scripts/deploy_server.sh
```

**This script will:**
- [ ] Install Docker and Docker Compose
- [ ] Install NVIDIA Container Toolkit
- [ ] Download neural networks (~600MB, ~5-10 min)
- [ ] Build Docker image (~10-15 min)
- [ ] Start the server
- [ ] Configure auto-start on boot
- [ ] Configure firewall (if UFW is active)

**Total time: 15-30 minutes** (mostly download and compile time)

### 4. Verify Deployment

```bash
# Check container is running
docker-compose ps

# Check logs
docker-compose logs --tail=50

# Test local access
curl http://localhost:3000

# Check GPU usage
nvidia-smi
```

- [ ] Container status shows "Up"
- [ ] Logs show "Server listening on 0.0.0.0:3000"
- [ ] Local curl returns HTML
- [ ] GPU shows VRAM usage (2-4GB when playing)

### 5. Test from Browser

**From your local machine:**
```bash
# Open in browser
http://<server-ip>:3000
```

**Test steps:**
- [ ] Selection screen loads with all 9 spirits
- [ ] Click Dragon spirit
- [ ] Choose 9×9 board
- [ ] Start game
- [ ] Place a stone
- [ ] Bot responds (wait ~5-10 seconds)
- [ ] Place another stone
- [ ] Verify board updates correctly

**If you can't access from outside:**
```bash
# On server, check firewall
sudo ufw status
sudo ufw allow 3000/tcp
sudo ufw reload
```

- [ ] Game accessible from browser
- [ ] Can place stones
- [ ] Bot responds with moves
- [ ] All spirits work

## Post-Deployment

### Optional: Set up Remote Access

**With domain name and HTTPS:**
```bash
# Install Nginx
sudo apt-get install -y nginx certbot python3-certbot-nginx

# Configure reverse proxy (see DEPLOY.md for details)
# Get SSL certificate
sudo certbot --nginx -d your-domain.com
```

- [ ] Domain configured (optional)
- [ ] HTTPS enabled (optional)

### Optional: Monitoring

**Set up monitoring scripts:**
```bash
# Add to crontab for health checks
crontab -e

# Add line:
# */5 * * * * curl -sf http://localhost:3000 > /dev/null || docker-compose restart
```

- [ ] Health monitoring set up (optional)

## Troubleshooting

### Container won't start

```bash
# Check logs for errors
docker-compose logs

# Common issues:
# 1. Neural networks not downloaded
ls -lh nets/  # Should show 2 .bin.gz files (~600MB total)

# 2. GPU not accessible
docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi

# 3. Port already in use
sudo lsof -i :3000
```

### Slow performance

```bash
# Check GPU is being used
nvidia-smi  # Should show python/KataGo process

# Check VRAM usage
watch -n 1 nvidia-smi

# If GPU not being used, check runtime:
docker info | grep -i runtime  # Should show "nvidia" in runtimes
```

### Can't access from browser

```bash
# Check firewall
sudo ufw status
sudo ufw allow 3000/tcp

# Check server is listening
sudo netstat -tlnp | grep 3000

# Check from server itself
curl http://localhost:3000
```

## Maintenance

### View logs
```bash
docker-compose logs -f
```

### Restart server
```bash
docker-compose restart
```

### Stop server
```bash
docker-compose down
```

### Update code
```bash
git pull
docker-compose up --build -d
```

### Check resource usage
```bash
docker stats animal-go-prod
nvidia-smi
```

## Success Criteria

✅ **Deployment is successful when:**
- [ ] Container is running (docker-compose ps shows "Up")
- [ ] Server responds on port 3000
- [ ] All 9 spirits are accessible
- [ ] Bots respond to moves in 5-30 seconds
- [ ] GPU shows 2-4GB VRAM usage during games
- [ ] Server auto-starts after reboot
- [ ] Game is playable from browsers on the network

## Reference Documentation

- [QUICKSTART_DEPLOY.md](QUICKSTART_DEPLOY.md) - Quick 3-step guide
- [DEPLOY.md](DEPLOY.md) - Complete deployment guide
- [DOCKER_DEPLOYMENT.md](DOCKER_DEPLOYMENT.md) - Docker details
- [README.md](README.md) - Project overview

## Support

If deployment fails, see [DEPLOY.md](DEPLOY.md) for manual installation steps, or check logs:
```bash
docker-compose logs -f
```
