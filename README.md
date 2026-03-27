# Spirit Animals Go

Play Go against nine unique AI opponents, each embodying a different philosophy and playing style.

## Status: Core Implementation Complete ✅

**Working:**
- ✅ Game rules engine (move validation, ko detection, captures)
- ✅ 9 spirit configurations with unique KataGo parameters
- ✅ Complete frontend with all 9 palettes
- ✅ WebSocket server with session management
- ✅ Jaguar dynamic visit scaling (500→20,000 over 120 moves)

**Pending:**
- ⏳ KataGo binary installation
- ⏳ Pass/Resign implementation
- ⏳ Spirit portraits (placeholders)
- ⏳ Crow ko detection (placeholder)

## Quick Start

### Prerequisites
1. Rust 1.70+
2. KataGo binary and models (see Setup below)

### Setup
```bash
# 1. Install KataGo (download from https://github.com/lightvector/KataGo/releases)
mkdir -p assets/katago
# Place katago binary and models in assets/katago/

# 2. Update config files with correct paths
# Edit configs/*.cfg to point to your KataGo models

# 3. Build and run
cargo run
```

### Development
```bash
# Backend only
cargo run

# Frontend is static files - edit in frontend/
# Rebuild if needed: ./build-frontend.sh (if created)
```

Server runs on `http://localhost:3000`

## The Nine Spirits

| Spirit | Style | KataGo Config |
|--------|-------|---------------|
| 🐉 Dragon | Living Go | humanv0, 7d 1950-1970, flowing |
| 🦐 Mantis Shrimp | AI Disciple | pure AI, 50k visits |
| 🕊️ Crane | Classicist | humanv0, 7d pre-1970, classical |
| 🕷️ Spider | Trickster | humanv0, 5d 1980s, high temp+noise |
| 🦅 Eagle | Moyo Artist | humanv0, 8d 1970-1990, influence |
| 🦁 Lion | Territorial | humanv0, 7d 1970s, safe territory |
| 🦗 Praying Mantis | Blood Warrior | pure AI, fighting style |
| 🐆 Jaguar | Endgame Assassin | **dynamic visits: 500→20k** |
| 🐦‍⬛ Crow | Ko Monster | high temp, ko awareness |

## Architecture

- **Backend:** Rust + Axum (WebSocket server)
- **Frontend:** HTML/CSS/JS (vanilla, no frameworks)
- **AI:** KataGo (GTP mode, one process per game)
- **Game Logic:** Reused from production-tested `../go` project

**Session Flow:**
```
Spirit selection → InitGame → Spawn KataGo →
Human move → Validate → Bot genmove → Update → Repeat
```

## Project Structure

```
animal_go/
├── src/
│   ├── game/        # Rules engine (copied from ../go)
│   ├── katago/      # GTP process manager + middleware
│   ├── spirits/     # Spirit enum + palettes
│   ├── main.rs      # Axum server
│   ├── ws.rs        # WebSocket handler
│   └── state.rs     # Session manager
├── configs/         # 9 KataGo .cfg files
├── frontend/        # Complete UI (selection + game)
└── docs/            # Vision + specs
```

## Testing

```bash
# Run unit tests
cargo test

# Manual testing checklist
# 1. Start server: cargo run
# 2. Open http://localhost:3000
# 3. Select a spirit (try Dragon first)
# 4. Choose board size (19×19 recommended)
# 5. Play a few moves
# 6. Verify bot responds
# 7. Try Jaguar - notice it gets stronger late game
```

## Development Notes

- KataGo configs use placeholder paths - update before running
- Jaguar middleware in `src/katago/jaguar.rs`
- All palettes in `frontend/css/themes.css`
- WebSocket protocol in `src/ws.rs`

## Documentation

- `docs/intro.md` - Project vision
- `docs/roster.md` - Spirit personalities
- `docs/model_setup.md` - KataGo config specs
- `docs/theming.md` - Palette specifications
- `docs/architecture.md` - System design
- `docs/implementation_plan.md` - Build guide

## Next Steps

1. Install KataGo binary and models
2. Update config file paths
3. Implement Pass/Resign handlers
4. Generate spirit portraits
5. Add game end detection
6. Deploy to server

## License

Built with Claude Code
