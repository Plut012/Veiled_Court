# Development Mode Guide

Quick setup for fast iteration on laptop hardware.

## TL;DR

```bash
./scripts/setup-dev-mode.sh  # One time
./scripts/run-dev.sh         # Every time
# Open http://localhost:3000
```

## What is Dev Mode?

Dev mode creates lightweight configs optimized for:
- ⚡ **Fast responses** (~1-2 seconds instead of 10-30 seconds)
- 💻 **Low VRAM usage** (~1-2GB instead of 3-4GB)
- 🔄 **Quick iteration** on UI/frontend changes

## Hardware Targets

### Laptop (Current)
- **GPU:** Quadro P2000 (4GB VRAM)
- **CUDA:** 13.0
- **Status:** ✅ Supported with dev configs

### Desktop (Production)
- **GPU:** RTX 2070 (8GB VRAM)
- **CUDA:** 11.x
- **Status:** Use full production configs

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
unset ANIMAL_GO_CONFIG_DIR
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

## Next Steps

1. ✅ Dev mode set up
2. ⏳ Install KataGo (see SETUP.md)
3. ⏳ Test with Dragon spirit (has human net)
4. ⏳ Test with Mantis Shrimp (standard only)
5. ⏳ Verify all 9 spirits load correctly
6. ⏳ Full desktop setup for production testing
