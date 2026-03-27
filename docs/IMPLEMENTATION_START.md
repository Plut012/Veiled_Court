# Implementation Start — Quick Reference

## Project Goal

Build a web-based Go game where you play against nine unique AI "spirit animals," each with distinct playing styles, visual themes, and personalities.

## Technology Stack

- **Backend:** Rust + Axum (reusing 60-70% from `../go`)
- **Frontend:** HTML/CSS/JS (adapted from `../phone_go`)
- **AI Engine:** KataGo (GTP mode, one process per game)
- **Deployment:** Single binary + static assets

## Key Files to Reference

### From `../go` (Rust foundation):
- `src/game/board.rs` — Complete, tested Go rules engine (copy as-is)
- `src/game/rules.rs` — Move validation, ko detection, capture resolution (copy as-is)
- `src/katago/mod.rs` — KataGo subprocess management (adapt from analysis→GTP)
- `src/main.rs` — Axum server setup (copy and modify routes)
- `src/ws.rs` — WebSocket handler (copy and extend messages)
- `src/state.rs` — Session management (copy and extend for spirits)

### From `../phone_go` (Frontend UX):
- `go-flow.html` — Canvas board rendering, stone preview, touch handling

### Project Docs:
- `docs/implementation_plan.md` — Full detailed implementation guide
- `docs/architecture.md` — Updated architecture with Rust decision
- `docs/model_setup.md` — KataGo config specs for all nine spirits
- `docs/theming.md` — CSS palette specs for all nine spirits
- `docs/roster.md` — Spirit personalities and philosophies

## Implementation Phases

### Phase 1: Foundation (Start Here)
1. Create `Cargo.toml` (base on `../go/Cargo.toml`)
2. Copy `src/game/` from `../go` → `./src/game/` (entire directory)
3. Copy `src/katago/mod.rs` from `../go` → adapt for GTP mode
4. Copy `src/main.rs`, `src/ws.rs`, `src/state.rs` from `../go` → adapt for spirits
5. Verify: `cargo build` succeeds, tests pass

### Phase 2: Spirit System
6. Create `src/spirits/mod.rs` (Spirit enum, palette definitions)
7. Create `configs/*.cfg` (nine KataGo config files)
8. Create `src/katago/jaguar.rs` (dynamic visit scaling)
9. Create `src/katago/crow.rs` (ko awareness)

### Phase 3: Frontend
10. Create `frontend/index.html` (spirit selection screen)
11. Create `frontend/game.html` (game board)
12. Create `frontend/css/themes.css` (nine palettes)
13. Create `frontend/js/board.js` (copy from phone_go)
14. Create `frontend/js/websocket.js` (WebSocket client)

### Phase 4: Integration
15. Wire up: selection → init → KataGo spawn → WebSocket upgrade
16. Implement move flow: human move → validation → bot move → update
17. Add Jaguar palette drift (interpolate CSS vars on move events)
18. Add Crow ko detection flags

### Phase 5: Polish
19. Create `build.sh`
20. Create `README.md`
21. Test all nine spirits

## The Nine Spirits

| Spirit | Archetype | Key Behavior |
|--------|-----------|--------------|
| 🐉 Dragon | Living Go | Flowing, connected play |
| 🦐 Mantis Shrimp | AI Disciple | Pure alien logic, 50k+ visits |
| 🕊️ Crane | Classicist | Perfect form, classical joseki |
| 🕷️ Spider | Trickster | Unexpected moves, high temperature |
| 🦅 Eagle | Moyo Artist | Influence over territory |
| 🦁 Lion | Territorial Builder | Solid, safe territory |
| 🦗 Praying Mantis | Blood Warrior | Aggressive fighting |
| 🐆 Jaguar | Endgame Assassin | **Weak early → strong late** (visits: 500→20k) |
| 🐦‍⬛ Crow | Ko Monster | **Ko detection middleware** |

## Critical Implementation Notes

### Jaguar (visits scale dynamically):
```rust
fn get_jaguar_visits(move_number: usize) -> u32 {
    if move_number < 40 { 500 }
    else if move_number < 80 { 2000 }
    else if move_number < 120 { 8000 }
    else { 20000 }
}
```

Override `maxVisits` via GTP before each `genmove`

### Crow (ko detection):
Check board state for ko patterns, flag to frontend:
```rust
ServerMessage::KoActive { threats: Vec<Position> }
```

Frontend dims board except ko-related intersections

### Palette System:
CSS custom properties swapped on spirit selection:
```css
.theme-dragon {
  --board-primary: #0D1F1A;
  --board-secondary: #2E6B55;
  --accent: #C9A84C;
}
```

Jaguar interpolates from warm→cold over 120 moves

## First Commands

```bash
cd /data/dev/animal_go

# Create Cargo.toml
# Copy game module from ../go
cp -r ../go/src/game ./src/

# Copy katago base
cp ../go/src/katago/mod.rs ./src/katago/

# Copy server scaffold
cp ../go/src/main.rs ./src/
cp ../go/src/ws.rs ./src/
cp ../go/src/state.rs ./src/

# Build
cargo build
```

## Success Criteria for Phase 1

- [ ] `cargo build` succeeds
- [ ] `cargo test` passes (game module tests)
- [ ] Server starts without errors
- [ ] Can serve static files at `/`

Then proceed to Phase 2 (Spirit system).

## Reference Chain

```
implementation_plan.md (detailed steps)
    ↓
architecture.md (system design)
    ↓
model_setup.md (KataGo configs)
theming.md (CSS palettes)
roster.md (spirit personalities)
    ↓
../go/src/ (reusable Rust code)
../phone_go/go-flow.html (reusable frontend UX)
```

---

**Ready to implement!** Start with Phase 1, verify each step compiles, then proceed sequentially through the phases.
