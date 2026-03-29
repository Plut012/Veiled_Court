# Download KataGo Models

The automated download is blocked by Cloudflare. Here's how to download manually:

## Quick Method (Command Line)

```bash
cd /data/dev/animal_go/assets/katago

# Download latest standard network from katagotraining.org
# Visit https://katagotraining.org/ and find the latest kata1 network
# OR use these direct commands:

# Option 1: Download from archive (stable)
wget "https://katagoarchive.org/g170/neuralnets/kata1/kata1-b28c512nbt-s8965856768-d4917590699.bin.gz"

# Option 2: Use smaller network for testing (faster download, ~200MB)
wget "https://katagoarchive.org/g170/neuralnets/kata1/kata1-b18c384nbt-s6582191360-d3422816034.bin.gz"

# Download human network
wget "https://github.com/lightvector/KataGo/releases/download/v1.15.0/b18c384nbt-humanv0.bin.gz"

# Create symlinks
ln -sf kata1-b28c512nbt-s8965856768-d4917590699.bin.gz kata1-b28c512nbt.bin.gz
ln -sf b18c384nbt-humanv0.bin.gz b18c384nbt-humanv0.bin.gz

# Update configs
cd ../..
./scripts/update-config-paths.sh
```

## Browser Method (If Command Line Fails)

1. **Standard Network** (~400MB):
   - Go to: https://katagotraining.org/
   - Look for latest `kata1-b28c512nbt-*.bin.gz`
   - Download and save to: `/data/dev/animal_go/assets/katago/`

2. **Human Network** (~200MB):
   - Go to: https://github.com/lightvector/KataGo/releases/tag/v1.15.0
   - Download: `b18c384nbt-humanv0.bin.gz`
   - Save to: `/data/dev/animal_go/assets/katago/`

3. **Update paths**:
   ```bash
   cd /data/dev/animal_go
   ./scripts/update-config-paths.sh
   ```

## Minimal Setup (Just to Test)

For quick testing, you can use just the standard network without the human network:

```bash
cd /data/dev/animal_go/assets/katago

# Download smaller standard network (~200MB instead of 400MB)
wget "https://katagoarchive.org/g170/neuralnets/kata1/kata1-b18c384nbt-s6582191360-d3422816034.bin.gz"

# Symlink it
ln -sf kata1-b18c384nbt-s6582191360-d3422816034.bin.gz kata1-b28c512nbt.bin.gz

# Test spirits that don't need human net:
# - Mantis Shrimp
# - Praying Mantis
# - Jaguar
# - Crow
```

## Verify Installation

```bash
cd /data/dev/animal_go

# Check files exist
ls -lh assets/katago/*.bin.gz

# Test KataGo with a network
assets/katago/katago benchmark -model assets/katago/kata1-*.bin.gz

# If that works, test the dev server
./scripts/run-dev.sh
```

## Troubleshooting

**"File not found"**
- Make sure you're in `/data/dev/animal_go/assets/katago/` directory
- Check the URL hasn't changed on katagotraining.org

**"Network error" / "403 Forbidden"**
- Try downloading from your browser instead
- Check your firewall/proxy settings

**"Out of space"**
- Both networks together: ~600MB
- Just standard network: ~200-400MB (depending on size)
- Free up space and try again

## Next Steps

Once models are downloaded:

```bash
# Update config paths
./scripts/update-config-paths.sh

# Run dev server
./scripts/run-dev.sh

# Open browser to http://localhost:3000
```

For minimal VRAM usage on your Quadro P2000:
- Start with 9×9 board
- Test Mantis Shrimp first (no human net needed)
- Then try Dragon (uses human net in dev mode)
