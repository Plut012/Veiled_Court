# Development Scripts

Quick reference for development workflows.

## Hardware Info

**Laptop:** Quadro P2000 (4GB VRAM, CUDA 13.0)
**Home Desktop:** RTX 2070 (8GB VRAM)

## Quick Start

### 1. Dev Mode (Laptop - Fast Iteration)

```bash
# First time setup
./scripts/setup-dev-mode.sh

# Run server in dev mode
./scripts/run-dev.sh

# Server runs on: http://localhost:3000
```

**Dev Mode Features:**
- Visit counts: 50-200 (vs 2000-50000 prod)
- Response time: ~0.5-2 seconds
- Only Dragon uses human net (saves VRAM)
- Uses `configs-dev/` instead of `configs/`

### 2. Test Single Spirit

```bash
# Test if a specific spirit config works
./scripts/test-spirit.sh dragon
./scripts/test-spirit.sh mantis_shrimp

# Requires KataGo to be installed first
```

### 3. Production Mode (Not Yet Implemented)

```bash
# Full setup on desktop with GPU
./scripts/setup-desktop.sh    # Downloads KataGo + models
./scripts/start-server.sh      # Production server
./scripts/stop-server.sh       # Clean shutdown
```

## Config Comparison

| Setting | Production | Dev Mode | Speedup |
|---------|-----------|----------|---------|
| Mantis Shrimp visits | 50,000 | 200 | 250x |
| Dragon visits | 3,000 | 50 | 60x |
| Other spirits visits | 2,000-5,000 | 50 | 40-100x |
| Threads | 4-8 | 2 | - |
| Human net | 5 spirits | Dragon only | -1GB VRAM |
| Response time | 5-30s | 0.5-2s | 10-15x |

## Environment Variables

```bash
# Use dev configs
export ANIMAL_GO_CONFIG_DIR="configs-dev"

# Use production configs (default)
export ANIMAL_GO_CONFIG_DIR="configs"

# Run with specific config dir
ANIMAL_GO_CONFIG_DIR="configs-dev" cargo run
```

## Development Workflow

### UI/Frontend Development
```bash
# Terminal 1: Dev server with fast AI
./scripts/run-dev.sh

# Terminal 2: Edit frontend files
cd frontend/dist
# Edit HTML/CSS/JS, refresh browser

# Frontend is served directly, no build step needed
```

### Backend Development
```bash
# Dev mode uses debug build (faster compilation)
ANIMAL_GO_CONFIG_DIR="configs-dev" cargo run

# Watch mode (requires cargo-watch)
cargo watch -x run
```

### Testing All Spirits
```bash
# Test each spirit's config
for spirit in dragon mantis_shrimp crane spider eagle lion praying_mantis jaguar crow; do
    echo "Testing $spirit..."
    ./scripts/test-spirit.sh $spirit
done
```

## File Structure

```
animal_go/
├── configs/           # Production configs (slow, accurate)
├── configs-dev/       # Dev configs (fast, good enough)
├── scripts/
│   ├── setup-dev-mode.sh       # Create dev configs
│   ├── run-dev.sh              # Run in dev mode
│   ├── test-spirit.sh          # Test single config
│   └── README.md               # This file
└── assets/
    └── katago/
        ├── katago              # Binary (install separately)
        ├── kata1-*.bin.gz      # Standard net (~400MB)
        └── b18c384nbt-humanv0.bin.gz  # Human net (~200MB)
```

## Next Steps

1. **Install KataGo** (required for testing):
   ```bash
   # Quick manual install
   mkdir -p assets/katago
   cd assets/katago
   # Download from: https://github.com/lightvector/KataGo/releases
   ```

2. **Run dev server**:
   ```bash
   ./scripts/run-dev.sh
   ```

3. **Test in browser**:
   - Open http://localhost:3000
   - Select Dragon (only spirit with human net in dev mode)
   - Choose 9×9 board for fastest testing
   - Play a few moves to verify AI responds

4. **Full desktop setup** (when ready):
   - Use scripts/setup-desktop.sh (to be implemented)
   - All 9 spirits with full strength
   - Production config with human nets
