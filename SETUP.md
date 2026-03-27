# Setup Instructions

## KataGo Installation

### 1. Download KataGo

```bash
# Create assets directory
mkdir -p assets/katago

# Download latest release from:
# https://github.com/lightvector/KataGo/releases
# Get the binary for your platform (Linux/macOS/Windows)

# Place in assets/katago/katago
chmod +x assets/katago/katago
```

### 2. Download Models

**Standard net (required for most spirits):**
```bash
cd assets/katago
wget https://github.com/lightvector/KataGo/releases/download/v1.15.3/kata1-b28c512nbt-s12345678-d1234567.bin.gz
```

**Human style net (required for Dragon, Crane, Spider, Eagle, Lion):**
```bash
wget https://github.com/lightvector/KataGo/releases/download/v1.15.3/b18c384nbt-humanv0.bin.gz
```

### 3. Update Config Files

Edit all 9 files in `configs/` to point to your models:

```bash
# In each .cfg file, update:
nnModelFile = /full/path/to/animal_go/assets/katago/kata1-b28c512nbt-s12345678-d1234567.bin.gz
humanModelFile = /full/path/to/animal_go/assets/katago/b18c384nbt-humanv0.bin.gz  # if used
```

**Files to update:**
- `configs/dragon.cfg`
- `configs/mantis_shrimp.cfg`
- `configs/crane.cfg`
- `configs/spider.cfg`
- `configs/eagle.cfg`
- `configs/lion.cfg`
- `configs/praying_mantis.cfg`
- `configs/jaguar.cfg`
- `configs/crow.cfg`

### 4. Test KataGo

```bash
# Verify it works
./assets/katago/katago version

# Test with a config
./assets/katago/katago gtp -model assets/katago/kata1-*.bin.gz -config configs/dragon.cfg
```

## Server Setup

```bash
# Build
cargo build --release

# Run
cargo run --release

# Or use the binary directly
./target/release/animal_go
```

Server listens on `http://localhost:3000`

## Troubleshooting

**"KataGo binary not found"**
- Check path in code or use absolute path
- Ensure binary has execute permissions

**"Model not found"**
- Update all .cfg files with full absolute paths
- Download correct model versions

**WebSocket connection fails**
- Check firewall
- Ensure port 3000 is available
- Check browser console for errors

**Slow responses**
- Reduce visit counts in configs (especially Mantis Shrimp's 50k)
- Use smaller board size (9×9 or 13×13)
- Check CPU usage

## Next Steps

Once setup is complete:
1. Open `http://localhost:3000`
2. Select Dragon spirit
3. Choose 9×9 board
4. Play a few moves to verify it works
5. Try other spirits
