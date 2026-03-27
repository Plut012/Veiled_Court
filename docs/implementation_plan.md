# Implementation Plan — Spirit Animals Go

## Strategy: Adapt & Extend

This project reuses 60-70% of the `../go` codebase, adding spirit-specific behavior on top of proven infrastructure.

---

## Phase 1: Foundation (Copy & Adapt)

### 1.1 Project Scaffold

**Goal:** Set up Rust project structure

**Tasks:**
- Create `Cargo.toml` based on `../go/Cargo.toml`
- Update project name to `animal_go`
- Add dependencies: `axum`, `tokio`, `serde`, `serde_json`, `tower-http`
- Create `src/` directory structure

**Files to create:**
```
animal_go/
├── Cargo.toml
├── src/
│   └── main.rs (placeholder)
```

**Reference:** `../go/Cargo.toml`

---

### 1.2 Game Module (Direct Copy)

**Goal:** Copy complete, tested Go rules engine

**Tasks:**
- Copy `../go/src/game/` directory entirely to `./src/game/`
- Files to copy:
  - `mod.rs` (module exports)
  - `types.rs` (Color, Position, GameState)
  - `board.rs` (Board struct, find_group, count_liberties)
  - `rules.rs` (is_suicide, find_captures, is_ko_violation, hash_board)

**Verification:**
- Run `cargo build` to ensure module compiles
- Run `cargo test` to verify all 15+ test cases pass

**No changes needed** — these files work as-is

---

### 1.3 KataGo Base (Adapt from Analysis to GTP)

**Goal:** Adapt KataGo integration from analysis mode to GTP mode for move generation

**Current state (in `../go`):**
- Uses KataGo analysis mode for territory estimation
- Sends JSON queries, receives ownership data

**Target state:**
- Use KataGo GTP mode for move generation
- Send GTP commands like `genmove white`, receive `= Q16`

**Tasks:**
- Copy `../go/src/katago/mod.rs` to `./src/katago/mod.rs`
- Simplify: Remove `OwnershipData`, `AnalysisQuery`, `AnalysisResponse`
- Add: GTP command/response handling
- Add: Spirit config file loading

**New GTP interface:**
```rust
pub struct KataGoProcess {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl KataGoProcess {
    pub fn spawn(spirit_config: &str) -> Result<Self, String>
    pub fn send_command(&mut self, cmd: &str) -> Result<String, String>
    pub fn genmove(&mut self, color: Color) -> Result<Position, String>
    pub fn play(&mut self, color: Color, pos: Position) -> Result<(), String>
}
```

**Reference patterns:**
- Subprocess spawn: `../go/src/katago/mod.rs:88-99`
- GTP coordinate conversion: `../go/src/katago/mod.rs:217-231`

---

### 1.4 Server Scaffold (Adapt)

**Goal:** Copy Axum server structure, adapt routes for spirit selection

**Tasks:**
- Copy `../go/src/main.rs` → `./src/main.rs`
- Copy `../go/src/ws.rs` → `./src/ws.rs`
- Copy `../go/src/state.rs` → `./src/state.rs`

**Changes needed:**

**In `main.rs`:**
- Update routes:
  - `GET /` → serve selection screen (index.html)
  - `GET /game` → serve game screen (game.html)
  - `GET /ws` → WebSocket handler
  - Static serving: `/assets`, `/css`, `/js`

**In `state.rs`:**
- Update `AppState` to include:
```rust
pub struct SessionData {
    pub game_state: GameState,
    pub katago_process: KataGoProcess,
    pub spirit: Spirit,
    pub board_size: usize,
    pub move_number: usize,
}

pub struct AppState {
    pub sessions: Arc<Mutex<HashMap<String, SessionData>>>,
}
```

**In `ws.rs`:**
- Add message types:
```rust
#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    InitGame { spirit: String, board_size: usize, player_color: String },
    Move { coord: String },
    Pass,
    Resign,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerMessage {
    GameStarted { session_id: String, board_size: usize },
    BoardUpdate { board: Vec<Vec<Option<Color>>>, last_move: Option<String>, move_number: usize },
    KoActive { threats: Vec<String> },
    GameOver { winner: String },
}
```

**Reference:**
- WebSocket handler: `../go/src/ws.rs`
- Static serving: `../go/src/main.rs:22-23`

---

## Phase 2: Spirit System (New Implementation)

### 2.1 Spirit Definitions

**Goal:** Define the nine spirits with their metadata

**Tasks:**
- Create `src/spirits/mod.rs`

**Implementation:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Spirit {
    Dragon,
    MantisShrimp,
    Crane,
    Spider,
    Eagle,
    Lion,
    PrayingMantis,
    Jaguar,
    Crow,
}

impl Spirit {
    pub fn config_file(&self) -> &'static str {
        match self {
            Spirit::Dragon => "configs/dragon.cfg",
            Spirit::MantisShrimp => "configs/mantis_shrimp.cfg",
            Spirit::Crane => "configs/crane.cfg",
            Spirit::Spider => "configs/spider.cfg",
            Spirit::Eagle => "configs/eagle.cfg",
            Spirit::Lion => "configs/lion.cfg",
            Spirit::PrayingMantis => "configs/praying_mantis.cfg",
            Spirit::Jaguar => "configs/jaguar.cfg",
            Spirit::Crow => "configs/crow.cfg",
        }
    }

    pub fn from_string(s: &str) -> Option<Spirit> {
        match s.to_lowercase().as_str() {
            "dragon" => Some(Spirit::Dragon),
            "mantis_shrimp" => Some(Spirit::MantisShrimp),
            "crane" => Some(Spirit::Crane),
            "spider" => Some(Spirit::Spider),
            "eagle" => Some(Spirit::Eagle),
            "lion" => Some(Spirit::Lion),
            "praying_mantis" => Some(Spirit::PrayingMantis),
            "jaguar" => Some(Spirit::Jaguar),
            "crow" => Some(Spirit::Crow),
            _ => None,
        }
    }

    pub fn palette(&self) -> Palette {
        match self {
            Spirit::Dragon => Palette {
                board_primary: "#0D1F1A",
                board_secondary: "#2E6B55",
                accent: "#C9A84C",
            },
            // ... remaining spirits
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Palette {
    pub board_primary: &'static str,
    pub board_secondary: &'static str,
    pub accent: &'static str,
}
```

**Reference:** `docs/theming.md` for all nine palettes

---

### 2.2 Spirit Config Files

**Goal:** Create nine KataGo configuration files

**Tasks:**
- Create `configs/` directory
- Create nine `.cfg` files based on `docs/model_setup.md`

**Template structure (from KataGo docs):**
```
# Dragon Configuration
# Japanese professional style, 7d, 1950-1970 era

# Neural networks
nnModelFile = path/to/kata1-b28c512nbt.gz
humanModelFile = path/to/b18c384nbt-humanv0.bin.gz

# Search parameters
maxVisits = 3000
humanSLProfile = rank7d,era1950-1970

# Temperature settings
rootPolicyTemperature = 1.0
chosenMoveTemperature = 1.0

# Utility factors
winLossUtilityFactor = 1.0
staticScoreUtilityFactor = 0.3

# Noise
rootNoiseEnabled = false
```

**Files to create:**
1. `configs/dragon.cfg` (humanv0, 7d 1950-1970, visits=3000)
2. `configs/mantis_shrimp.cfg` (standard only, visits=50000+)
3. `configs/crane.cfg` (humanv0, 7d 1940-1970, temp=0.8)
4. `configs/spider.cfg` (humanv0, 5d 1980-1990, temp=1.4, noise=true)
5. `configs/eagle.cfg` (humanv0, 8d 1970-1990, low staticScore)
6. `configs/lion.cfg` (humanv0, 7d 1970-1980, high staticScore)
7. `configs/praying_mantis.cfg` (standard, visits=2000, very high winLoss)
8. `configs/jaguar.cfg` (standard, visits=500 base — see middleware)
9. `configs/crow.cfg` (standard, visits=2000, temp=1.4, noise=true)

**Reference:** `docs/model_setup.md` lines 29-148 for full specifications

---

### 2.3 Jaguar Middleware

**Goal:** Implement dynamic visit scaling based on move number

**Tasks:**
- Create `src/katago/jaguar.rs`

**Implementation:**
```rust
pub fn get_jaguar_visits(move_number: usize) -> u32 {
    if move_number < 40 {
        500      // Opening — casual, approachable
    } else if move_number < 80 {
        2000     // Midgame — slightly more present
    } else if move_number < 120 {
        8000     // Late midgame — the shift begins
    } else {
        20000    // Endgame — transformation complete
    }
}

pub fn apply_jaguar_genmove(
    process: &mut KataGoProcess,
    move_number: usize,
    color: Color,
) -> Result<Position, String> {
    let visits = get_jaguar_visits(move_number);

    // Override maxVisits via GTP command before genmove
    process.send_command(&format!("kata-set-param maxVisits {}", visits))?;

    // Generate move
    process.genmove(color)
}
```

**Reference:** `docs/architecture.md` lines 149-163

---

### 2.4 Crow Middleware

**Goal:** Detect ko situations and flag them to the frontend

**Tasks:**
- Create `src/katago/crow.rs`

**Implementation:**
```rust
use crate::game::{Board, Position, rules};

pub fn check_ko_active(board: &Board) -> Option<KoInfo> {
    // Check if the current position has an active ko
    // This would require tracking recent board history
    // For now, simplified: check if last move created a ko shape

    // Return ko position and potential threats if detected
    None // Placeholder
}

#[derive(Debug, Serialize)]
pub struct KoInfo {
    pub ko_position: Position,
    pub threats: Vec<Position>,
}

pub fn apply_crow_genmove(
    process: &mut KataGoProcess,
    board: &Board,
    color: Color,
) -> Result<(Position, Option<KoInfo>), String> {
    // Check for ko before move
    let ko_info = check_ko_active(board);

    // If ko is active, optionally bias search toward ko threats
    if ko_info.is_some() {
        // Could adjust search parameters here
        // For MVP, just detect and flag
    }

    let move_pos = process.genmove(color)?;

    Ok((move_pos, ko_info))
}
```

**Reference:** `docs/architecture.md` lines 167-175, `docs/roster.md` lines 13-14

---

## Phase 3: Frontend (Copy & Adapt)

### 3.1 Frontend Scaffold

**Goal:** Copy board rendering from phone_go, restructure into clean HTML/CSS/JS

**Tasks:**
- Create `frontend/` directory structure:
```
frontend/
├── index.html       # Spirit selection screen
├── game.html        # Game board
├── css/
│   ├── themes.css   # All nine spirit palettes
│   └── board.css    # Board rendering styles
├── js/
│   ├── board.js     # Canvas rendering logic
│   ├── websocket.js # WebSocket client
│   ├── theme.js     # Palette switching
│   └── selection.js # Spirit selection
└── assets/
    └── portraits/   # (placeholder for portraits)
```

**Copy patterns from:**
- Board canvas rendering: `../phone_go/go-flow.html` lines ~300-600
- Stone placement preview: `../phone_go/go-flow.html` lines ~400-450
- Touch event handling: `../phone_go/go-flow.html` lines ~500-550

---

### 3.2 Selection Screen

**Goal:** Create spirit selection UI with portrait grid

**Tasks:**
- Create `frontend/index.html`

**Layout:**
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Spirit Animals — Go</title>
  <link rel="stylesheet" href="/css/themes.css">
  <link rel="stylesheet" href="/css/board.css">
</head>
<body>
  <div id="selection-screen">
    <h1>The Spirit Animals of Go</h1>
    <p>Choose your opponent. Study their nature. The board will reveal your own.</p>

    <div id="spirit-grid">
      <!-- 9 spirit portraits, 3x3 grid -->
      <div class="spirit-card" data-spirit="dragon">
        <img src="/assets/portraits/dragon.png" alt="Dragon">
        <h3>🐉 Dragon</h3>
        <p>Living Go</p>
      </div>
      <!-- ... 8 more spirits ... -->
    </div>

    <div id="game-options">
      <select id="board-size">
        <option value="9">9×9</option>
        <option value="13">13×13</option>
        <option value="19" selected>19×19</option>
      </select>

      <select id="player-color">
        <option value="black">Black</option>
        <option value="white" selected>White</option>
      </select>
    </div>
  </div>

  <script src="/js/selection.js"></script>
</body>
</html>
```

**Reference:** `docs/visual_direction.md` for spirit descriptions

---

### 3.3 Game Screen

**Goal:** Create game board with controls

**Tasks:**
- Create `frontend/game.html`

**Layout:**
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Game — Spirit Animals</title>
  <link rel="stylesheet" href="/css/themes.css">
  <link rel="stylesheet" href="/css/board.css">
</head>
<body class="theme-dragon"> <!-- Class updated on spirit selection -->
  <div id="game-screen">
    <div id="board-container">
      <canvas id="board"></canvas>
    </div>

    <div id="controls">
      <button id="pass-btn">Pass</button>
      <button id="resign-btn">Resign</button>
    </div>

    <div id="info">
      <span id="move-count">Move 0</span>
      <span id="last-move"></span>
    </div>

    <img id="spirit-portrait" src="" alt="">
  </div>

  <script src="/js/board.js"></script>
  <script src="/js/websocket.js"></script>
  <script src="/js/theme.js"></script>
</body>
</html>
```

**Reference:** Copy canvas rendering from `../phone_go/go-flow.html`

---

### 3.4 Theme System

**Goal:** Create CSS custom properties for all nine spirit palettes

**Tasks:**
- Create `frontend/css/themes.css`

**Implementation:**
```css
:root {
  /* Default palette (Dragon) */
  --board-primary: #0D1F1A;
  --board-secondary: #2E6B55;
  --accent: #C9A84C;
}

.theme-dragon {
  --board-primary: #0D1F1A;
  --board-secondary: #2E6B55;
  --accent: #C9A84C;
}

.theme-mantis-shrimp {
  --board-primary: #080B10;
  --board-secondary: #0D4D5E;
  --accent: #9B5DE5;
}

.theme-crane {
  --board-primary: #E8E4DC;
  --board-secondary: #4A4A52;
  --accent: #A8B8C8;
}

/* ... 6 more themes ... */

/* Jaguar warm/cold variants */
.theme-jaguar-warm {
  --board-primary: #1C1508;
  --board-secondary: #5C3D1A;
  --accent: #C9A870;
}

.theme-jaguar-cold {
  --board-primary: #0A0C10;
  --board-secondary: #2A3040;
  --accent: #B0C4D8;
}
```

**Reference:** `docs/theming.md` for all palettes

---

### 3.5 WebSocket Client

**Goal:** Connect frontend to Rust backend via WebSocket

**Tasks:**
- Create `frontend/js/websocket.js`

**Implementation:**
```javascript
class GameClient {
  constructor() {
    this.ws = null;
    this.sessionId = null;
  }

  connect() {
    this.ws = new WebSocket(`ws://${window.location.host}/ws`);

    this.ws.onopen = () => {
      console.log('Connected to game server');
    };

    this.ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      this.handleMessage(msg);
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }

  initGame(spirit, boardSize, playerColor) {
    this.send({
      type: 'InitGame',
      spirit: spirit,
      board_size: boardSize,
      player_color: playerColor
    });
  }

  makeMove(coord) {
    this.send({
      type: 'Move',
      coord: coord
    });
  }

  pass() {
    this.send({ type: 'Pass' });
  }

  resign() {
    this.send({ type: 'Resign' });
  }

  send(message) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  handleMessage(msg) {
    switch (msg.type) {
      case 'GameStarted':
        this.sessionId = msg.session_id;
        // Navigate to game screen
        window.location.href = `/game?session=${msg.session_id}`;
        break;

      case 'BoardUpdate':
        // Update board rendering
        updateBoard(msg.board, msg.last_move, msg.move_number);
        break;

      case 'KoActive':
        // Apply Crow board-dim effect
        if (currentSpirit === 'crow') {
          applyKoDim(msg.threats);
        }
        break;

      case 'GameOver':
        // Show game result
        showGameOver(msg.winner);
        break;
    }
  }
}
```

**Reference:** `../phone_go/go-flow.html` Socket.IO patterns (adapt to raw WebSocket)

---

## Phase 4: Integration

### 4.1 Session Flow

**Goal:** Connect all pieces: selection → init → spawn → WebSocket upgrade

**Implementation in `ws.rs`:**
```rust
async fn handle_client_message(
    msg: ClientMessage,
    state: Arc<AppState>,
    session_id: Option<String>,
) -> Result<ServerMessage, String> {
    match msg {
        ClientMessage::InitGame { spirit, board_size, player_color } => {
            // Parse spirit
            let spirit_enum = Spirit::from_string(&spirit)
                .ok_or("Invalid spirit")?;

            // Generate session ID
            let session_id = uuid::Uuid::new_v4().to_string();

            // Spawn KataGo process with spirit config
            let config_path = spirit_enum.config_file();
            let katago = KataGoProcess::spawn(config_path)?;

            // Create game state
            let game_state = GameState::new(board_size);

            // Store session
            let session_data = SessionData {
                game_state,
                katago_process: katago,
                spirit: spirit_enum,
                board_size,
                move_number: 0,
            };

            state.sessions.lock().unwrap()
                .insert(session_id.clone(), session_data);

            Ok(ServerMessage::GameStarted {
                session_id,
                board_size,
            })
        }

        ClientMessage::Move { coord } => {
            // Validate session
            let session_id = session_id.ok_or("No session")?;

            let mut sessions = state.sessions.lock().unwrap();
            let session = sessions.get_mut(&session_id)
                .ok_or("Session not found")?;

            // Parse coordinate
            let pos = parse_gtp_coord(&coord, session.board_size)?;

            // Validate move
            validate_move(&session.game_state.board, pos)?;

            // Apply human move to board
            session.game_state.apply_move(pos, Color::Black);

            // Send to KataGo
            session.katago_process.play(Color::Black, pos)?;

            // Get bot response
            let bot_move = match session.spirit {
                Spirit::Jaguar => {
                    jaguar::apply_jaguar_genmove(
                        &mut session.katago_process,
                        session.move_number,
                        Color::White
                    )?
                }
                Spirit::Crow => {
                    let (pos, ko_info) = crow::apply_crow_genmove(
                        &mut session.katago_process,
                        &session.game_state.board,
                        Color::White
                    )?;
                    // Send ko info separately if present
                    pos
                }
                _ => {
                    session.katago_process.genmove(Color::White)?
                }
            };

            // Apply bot move
            session.game_state.apply_move(bot_move, Color::White);
            session.move_number += 1;

            Ok(ServerMessage::BoardUpdate {
                board: session.game_state.board.to_2d_array(),
                last_move: Some(position_to_gtp(bot_move)),
                move_number: session.move_number,
            })
        }

        // ... Pass, Resign handlers
    }
}
```

---

### 4.2 Jaguar Palette Drift

**Goal:** Update CSS custom properties on each move for Jaguar

**Implementation in `frontend/js/theme.js`:**
```javascript
function updateJaguarPalette(moveNumber) {
  const root = document.documentElement;

  if (moveNumber >= 120) {
    // Snap to cold palette at move 120
    root.style.setProperty('--board-primary', '#0A0C10');
    root.style.setProperty('--board-secondary', '#2A3040');
    root.style.setProperty('--accent', '#B0C4D8');
  } else {
    // Linear interpolation from warm to cold (0 → 120)
    const progress = moveNumber / 120;

    const boardPrimary = interpolateColor('#1C1508', '#0A0C10', progress);
    const boardSecondary = interpolateColor('#5C3D1A', '#2A3040', progress);
    const accent = interpolateColor('#C9A870', '#B0C4D8', progress);

    root.style.setProperty('--board-primary', boardPrimary);
    root.style.setProperty('--board-secondary', boardSecondary);
    root.style.setProperty('--accent', accent);
  }

  // Update portrait at move 120
  if (moveNumber === 120) {
    document.getElementById('spirit-portrait').src =
      '/assets/portraits/jaguar_cold.png';
  }
}

function interpolateColor(color1, color2, progress) {
  const c1 = hexToRgb(color1);
  const c2 = hexToRgb(color2);

  const r = Math.round(c1.r + (c2.r - c1.r) * progress);
  const g = Math.round(c1.g + (c2.g - c1.g) * progress);
  const b = Math.round(c1.b + (c2.b - c1.b) * progress);

  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
}
```

**Reference:** `docs/theming.md` lines 129-143

---

## Phase 5: Polish

### 5.1 Build Script

**Goal:** Single command to build everything

**Tasks:**
- Copy `../go/build.sh` to `./build.sh`
- Update paths for animal_go

**Script:**
```bash
#!/bin/bash
set -e

echo "Building Spirit Animals Go..."

# Build Rust backend
echo "Building Rust backend..."
cargo build --release

# Frontend is already static HTML/CSS/JS - no build needed

echo "Build complete!"
echo "Run: ./target/release/animal_go"
```

---

### 5.2 README

**Goal:** Document project setup and running

**Tasks:**
- Create `README.md`

**Content:**
```markdown
# Spirit Animals — Go

Play Go against nine unique AI opponents, each embodying a different philosophy.

## Quick Start

**Development:**
```bash
cargo run
# Open http://localhost:3000
```

**Production:**
```bash
./build.sh
./target/release/animal_go
```

## Requirements

- Rust 1.70+
- KataGo binary and models (place in `assets/katago/`)
- Modern browser (Chrome, Firefox, Safari)

## The Nine Spirits

- 🐉 **Dragon** — Living Go
- 🦐 **Mantis Shrimp** — The AI Disciple
- 🕊️ **Crane** — The Classicist
- 🕷️ **Spider** — The Trickster
- 🦅 **Eagle** — The Moyo Artist
- 🦁 **Lion** — The Territorial Builder
- 🦗 **Praying Mantis** — The Blood Warrior
- 🐆 **Jaguar** — The Endgame Assassin
- 🐦‍⬛ **Crow** — The Ko Monster

See `docs/roster.md` for full descriptions.
```

---

## Success Criteria

### MVP Complete When:

1. ✅ Can select a spirit from the selection screen
2. ✅ Game initializes with correct board size
3. ✅ Can place stones (preview + submit)
4. ✅ KataGo responds with valid moves
5. ✅ Captures are resolved correctly
6. ✅ Ko rule is enforced
7. ✅ All nine spirits have distinct configurations
8. ✅ Jaguar palette drifts from warm to cold over 120 moves
9. ✅ Crow detects and flags ko situations

### Polish Complete When:

10. ✅ All nine spirit portraits are integrated
11. ✅ Pass/Resign work correctly
12. ✅ Game end detection works
13. ✅ Mobile touch input works smoothly
14. ✅ All palettes look correct
15. ✅ Build script produces single deployable binary

---

## Next Steps for Implementation Agent

**Start with Phase 1:**
1. Create Cargo.toml
2. Copy game/ module from ../go
3. Copy katago/ base from ../go
4. Copy server scaffold (main.rs, ws.rs, state.rs)
5. Verify compilation with `cargo build`

**Then proceed to Phase 2** (Spirit system)

**Reference throughout:**
- `../go/src/` for proven Rust patterns
- `../phone_go/go-flow.html` for frontend UX patterns
- `docs/` for spirit specifications

