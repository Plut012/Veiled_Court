# Local Development

```bash
./scripts/setup-dev-mode.sh  # one time
./scripts/run-dev.sh         # every time
# http://localhost:3000
```

Dev configs reduce KataGo visits for fast iteration (~1-2s responses, ~1-2GB VRAM).

## Config Differences

### Mantis Shrimp (The AI Disciple)
```
Production: maxVisits = 50000  (~30 seconds)
Dev Mode:   maxVisits = 200    (~2 seconds)

Purpose: Mantis Shrimp is meant to be "alien" - in prod it's
         terrifyingly strong. In dev it's just competent.
```

### Dragon (Living Go)
```
Production: maxVisits = 3000, humanSLProfile = rank7d
Dev Mode:   maxVisits = 50, humanSLProfile = rank7d

Purpose: Dragon keeps human net in dev mode to test dual-loading.
         All other spirits skip human net to save VRAM.
```

### All Others
```
Production: maxVisits = 2000-5000
Dev Mode:   maxVisits = 50

Purpose: Fast enough to test gameplay, not accurate strength.
```

## VRAM Usage

| Mode | Standard Net | Human Net | Total |
|------|--------------|-----------|-------|
| **Production** (all spirits) | ~1.5GB | ~1.5GB | ~3GB |
| **Dev Mode** (Dragon only) | ~1.5GB | ~1.5GB | ~3GB |
| **Dev Mode** (other spirits) | ~1.5GB | - | ~1.5GB |

**Quadro P2000:** 4GB total, ~3.3GB available → ✅ Dev mode works fine

## When to Use Each Mode

### Use Dev Mode When:
- Developing frontend/UI
- Testing WebSocket flow
- Debugging game logic
- Testing session management
- Quick manual testing
- Running on laptop

### Use Production Mode When:
- Final testing before release
- Playing actual games
- Testing spirit personalities
- Benchmarking performance
- Recording demos
- Running on desktop

## Quick Commands

```bash
# Create dev configs (one time)
./scripts/setup-dev-mode.sh

# Run in dev mode
./scripts/run-dev.sh

# Test specific spirit config
./scripts/test-spirit.sh dragon

# Check GPU usage while running
watch -n 1 nvidia-smi

# Switch back to production mode
unset KITSUNE_CONFIG_DIR
cargo run --release
```

## Response Time Comparison

| Spirit | Production | Dev Mode | Board |
|--------|-----------|----------|-------|
| Mantis Shrimp | 30s | 2s | 19×19 |
| Dragon | 8s | 1s | 19×19 |
| Crane | 7s | 1s | 19×19 |
| All spirits | 2s | 0.5s | 9×9 |

**Tip:** Use 9×9 boards in dev mode for even faster testing!

## Troubleshooting

### "Config not found"
```bash
# Configs missing - run setup
./scripts/setup-dev-mode.sh
```

### "KataGo not found"
```bash
# Need to install KataGo first
# See SETUP.md for instructions
mkdir -p assets/katago
# Download binary from KataGo releases
```

### "Out of memory"
```bash
# Check what's using VRAM
nvidia-smi

# Try closing other apps (Firefox uses ~200MB)
# Or test with only standard net spirits:
#   mantis_shrimp, praying_mantis, jaguar, crow
```

### Slow responses even in dev mode
```bash
# Check config was updated
grep maxVisits configs-dev/dragon.cfg
# Should show: maxVisits = 50

# Verify dev config is being used
RUST_LOG=debug ./scripts/run-dev.sh
# Look for: "Loading config from configs-dev/dragon.cfg"
```

## KataGo Setup

Place KataGo binary and neural nets in `assets/katago/`:

```bash
mkdir -p assets/katago
# Download katago binary from https://github.com/lightvector/KataGo/releases
# Download kata1-b28c512nbt.bin.gz (standard net, ~400MB)
# Download b18c384nbt-humanv0.bin.gz (human net, ~200MB)
chmod +x assets/katago/katago
```
