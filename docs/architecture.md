# Architecture Overview

## Current State

The existing codebase (`phone_go`) is a **thin proxy client**. It owns the UI and a lightweight Python Flask server that bypasses CORS and forwards traffic to OGS. All game logic, bot intelligence, and state management live on OGS's infrastructure.

```
CURRENT
──────────────────────────────────────────
Browser (HTML/JS)
    ↕  HTTP + WebSocket
Python Flask Proxy (localhost:5000)
    ↕  REST + WebSocket forwarded
OGS Server
    ├── Game state & rules
    ├── Bot processes (Amy, KataGo, Nightly)
    └── Auth & session
──────────────────────────────────────────
```

---

## Target State

The hosted server owns everything. OGS is removed entirely. KataGo runs locally as a managed subprocess pool, one process per active game. Game state, rules enforcement, session management, and real-time communication are all handled by the new backend.

**Technology Decision:** Rust + Axum (adapting from the existing `go` project which already has production-tested rules engine, KataGo integration, and WebSocket infrastructure)

```
TARGET
──────────────────────────────────────────
Browser (HTML/JS)
    ↕  WebSocket (wss://)
    ↕  HTTP (REST for session init)
Game Server (Rust + Axum + async WebSocket)
    ├── Session Manager
    │     └── maps session → game state + KataGo process
    ├── Game State Store (in-memory)
    │     ├── board position (SGF/coordinate)
    │     ├── move history
    │     ├── captured stones
    │     └── game phase (opening / midgame / endgame)
    ├── Go Rules Engine
    │     ├── move validation
    │     ├── ko detection
    │     ├── capture resolution
    │     └── scoring & game end detection
    └── KataGo Process Manager
          ├── subprocess spawn (one per game)
          ├── GTP communication layer (stdin/stdout)
          ├── spirit config loader (configs/[spirit].cfg)
          ├── Jaguar visit-scaling middleware
          └── Crow ko-awareness middleware
──────────────────────────────────────────
KataGo (per active game)
    ├── Standard net: kata1-b28c512nbt
    └── Human net: b18c384nbt-humanv0 (where configured)
──────────────────────────────────────────
```

---

## Component Breakdown

### 1. Game Server

**Replaces:** Python Flask proxy

**Technology:** Rust + Axum with async WebSocket support

**Responsibilities:**
- Serve the frontend assets (HTML, CSS, JS, images)
- Accept WebSocket connections from browser clients
- Route moves between the browser and the KataGo process
- Push board state updates to the browser after each move (human or bot)
- Manage connection drops and reconnects

**Why Rust + Axum:** The existing `go` project already has a production-tested Rust backend with Axum WebSocket support, complete rules engine, and KataGo integration. Reusing this foundation provides:
- Battle-tested Go rules implementation (move validation, ko detection, capture resolution)
- Proven KataGo subprocess management and GTP communication
- Type safety and performance
- Single binary deployment
- 60-70% of implementation already complete

---

### 2. Session Manager

**New component**

Holds the mapping from a browser session to:
- The active game state object
- The KataGo subprocess handle for that game
- The spirit configuration selected

Sessions are in-memory. No database required for single-player vs bot games. Sessions expire on disconnect or game end, and the associated KataGo subprocess is terminated and cleaned up.

```rust
// In-memory session store using Arc<Mutex<HashMap>>
struct SessionData {
    game_state: GameState,
    katago_process: KataGoProcess,
    spirit: Spirit,
    board_size: usize,
}

type Sessions = Arc<Mutex<HashMap<String, SessionData>>>;
```

---

### 3. Go Rules Engine

**Reused from `../go` project — production-tested implementation**

The rules engine from the existing Rust `go` project provides complete, battle-tested functionality:

**Location:** `../go/src/game/board.rs`, `../go/src/game/rules.rs`

**Features:**
- Move validation (legal placement, suicide detection)
- Ko detection using position hashing
- Capture resolution (group finding, liberty counting)
- Board state management
- Comprehensive test coverage (15+ test cases covering edge cases)

**Implementation:**
```rust
// From board.rs
pub fn find_group(&self, pos: Position) -> HashSet<Position>
pub fn count_liberties(&self, pos: Position) -> usize

// From rules.rs
pub fn is_suicide(board: &Board, pos: Position, color: Color) -> bool
pub fn find_captures(board: &Board, opponent_color: Color) -> Vec<Position>
pub fn is_ko_violation(board_hash: u64, history: &[u64]) -> bool
pub fn hash_board(board: &Board) -> u64
```

The rules engine sits between the browser's submitted move and the GTP command sent to KataGo — it validates the human move before passing it to the engine, and handles game end detection independently of KataGo's assessment.

---

### 4. KataGo Process Manager

**Adapted from `../go/src/katago/mod.rs` — proven subprocess management**

Manages the lifecycle of KataGo subprocesses. Each active game gets one KataGo process, spawned with the spirit's config file.

**Reused from existing implementation:**
- Subprocess spawn and lifecycle management
- GTP protocol communication (stdin/stdout)
- Position to GTP coordinate conversion (`(3,3)` → `"D4"`)
- Position hashing for analysis caching
- Process cleanup on drop

**Subprocess lifecycle:**
```
Spirit selected → spawn KataGo with spirit.cfg → game begins
Move submitted → send GTP command to process stdin → read response from stdout
Game ends / disconnect → terminate process → release resources
```

**GTP communication layer** — KataGo speaks GTP over stdin/stdout. The process manager wraps this in an async reader/writer:

```
Server sends:  "play black D4\n"
KataGo sends:  "= \n\n"
Server sends:  "genmove white\n"
KataGo sends:  "= Q16\n\n"
```

**New additions to existing code:**
- Spirit config loader — loads `.cfg` file from `configs/[spirit].cfg` and passes to KataGo at spawn
- Jaguar middleware — dynamic visit count override
- Crow middleware — ko detection flags and move biasing

---

### 5. Custom Middleware (Jaguar & Crow)

These two spirits require logic that sits between the game server and the KataGo process — implemented inside the KataGo Process Manager, not inside KataGo config.

**Jaguar — Dynamic Visit Scaling**

The visit count ramps dynamically based on move number:

```python
def get_jaguar_visits(move_number: int) -> int:
    if move_number < 40:
        return 500      # opening — casual, approachable
    elif move_number < 80:
        return 2000     # midgame — slightly more present
    elif move_number < 120:
        return 8000     # late midgame — the shift begins
    else:
        return 20000    # endgame — transformation complete
```

This is implemented by overriding the `maxVisits` GTP parameter before each `genmove` call rather than in the static config file.

**Crow — Ko Awareness Middleware**

On each position, the middleware:
1. Checks the current position for active ko situations (via the rules engine)
2. If ko is active, flags this to the frontend (triggering the board-dim UI effect)
3. Optionally nudges move selection toward ko-threatening positions by temporarily adjusting KataGo's search parameters when ko is present

The ko state is also communicated to the browser via the WebSocket as a game state flag, allowing the frontend to apply the Crow's board-dim effect independently of move events.

---

### 6. Frontend

**Carries forward from current codebase with significant restructuring**

The existing flat HTML files (`go-flow.html`, `2nd_theme.html`) are replaced with a structured frontend:

**Spirit Selection Screen**
- Displays all nine spirit portraits
- Each portrait previews its palette on hover
- Selection triggers session initialization via REST, then WebSocket upgrade

**Board Renderer**
- Accepts a theme config object (CSS custom properties from `theming.md`)
- Renders board, lines, hoshi, and stones with palette applied
- Stone placement preview (existing behavior carried forward)
- Pass / Resign controls

**Theme System**
- All six palette values per spirit stored as CSS custom properties
- Swapped atomically on spirit selection
- Jaguar drift implemented as a CSS custom property updated on each move event (linear interpolation between warm and cold values from move 0 → 120)

**WebSocket Client**
- Connects on game start
- Sends human moves
- Receives board state updates, bot moves, ko flags, game end events
- Handles reconnect on drop

**Character Portrait**
- Spirit portrait displayed in corner of game screen
- Jaguar switches between `jaguar_warm.png` and `jaguar_cold.png` at move 120 threshold
- All other portraits are static

---

## Data Flow — One Full Move Cycle

```
1. Human taps intersection on board
2. Frontend previews stone (semi-transparent)
3. Human confirms move (tap golden bar — existing UX preserved)
4. Frontend sends WebSocket message: { type: "move", coord: "D4", color: "black" }
5. Server receives move
6. Rules engine validates move (legal? ko? capture?)
7. Game state updated (captures resolved, board position updated)
8. Ko state checked → if active, send { type: "ko_active", threats: [...] } to frontend
9. Server sends GTP command to KataGo: "play black D4"
10. Server sends GTP command: "genmove white"
    (Jaguar middleware: override maxVisits based on move_number before this call)
11. KataGo responds: "= Q16"
12. Rules engine applies bot move to game state
13. Game end detection (if applicable)
14. Server sends WebSocket message to frontend:
    { type: "board_update", board: [...], last_move: "Q16", move_number: N, game_over: false }
15. Frontend renders updated board with theme applied
16. If Jaguar: update CSS custom properties for palette drift
17. If Crow + ko active: apply board-dim effect
```

---

## Hosting

**Two viable approaches given the RTX 2070:**

**Option A — Local machine as server**
Run the game server directly on the RTX 2070 machine. KataGo uses the GPU directly via CUDA. Expose via a reverse proxy (Nginx) with HTTPS (Cloudflare tunnel or Let's Encrypt). Simple. No latency between server and GPU.

**Option B — VPS + local KataGo**
Host the web server on a cheap VPS (for uptime and public IP). The VPS calls back to the local machine for KataGo moves via a small internal API. More complex, introduces network latency between VPS and KataGo, but separates hosting concerns from GPU machine uptime.

**Recommendation: Option A** for a personal or small-scale project. The RTX 2070 handles both the game server and KataGo without conflict. Use a Cloudflare tunnel for HTTPS without a static IP.

---

## File Structure (Target)

```
animal_go/
├── src/
│   ├── main.rs                 # Axum app, WebSocket handler, routes
│   ├── state.rs                # Session manager, in-memory game store
│   ├── ws.rs                   # WebSocket message handling
│   ├── game/
│   │   ├── mod.rs              # Module exports
│   │   ├── types.rs            # Color, Position, GameState
│   │   ├── board.rs            # ✅ Reused from ../go
│   │   └── rules.rs            # ✅ Reused from ../go
│   ├── katago/
│   │   ├── mod.rs              # ✅ Adapted from ../go
│   │   ├── gtp.rs              # GTP protocol (optional split)
│   │   ├── spirit.rs           # Spirit enum and config mapping
│   │   ├── jaguar.rs           # Jaguar dynamic visit scaling
│   │   └── crow.rs             # Crow ko awareness middleware
│   └── spirits/
│       └── mod.rs              # Spirit definitions and palette data
├── configs/
│   ├── dragon.cfg
│   ├── mantis_shrimp.cfg
│   ├── crane.cfg
│   ├── spider.cfg
│   ├── eagle.cfg
│   ├── lion.cfg
│   ├── praying_mantis.cfg
│   ├── jaguar.cfg
│   └── crow.cfg
├── frontend/
│   ├── index.html              # Selection screen
│   ├── game.html               # Board + game UI
│   ├── css/
│   │   ├── themes.css          # CSS custom properties per spirit
│   │   └── board.css           # Board renderer styles
│   ├── js/
│   │   ├── board.js            # ✅ Adapted from ../phone_go
│   │   ├── websocket.js        # WebSocket client
│   │   ├── theme.js            # Theme switching, Jaguar drift, Crow dim
│   │   └── selection.js        # Spirit selection screen
│   └── assets/
│       ├── portraits/          # Nine spirit portraits + jaguar_warm/cold
│       └── icons/              # Corner crop versions
├── docs/
│   ├── intro.md
│   ├── roster.md
│   ├── model_setup.md
│   ├── theming.md
│   ├── visual_direction.md
│   ├── architecture.md
│   └── reusable_components_analysis.md
├── Cargo.toml
├── Cargo.lock
├── build.sh                    # ✅ Reused from ../go
└── README.md
```

---

## What Carries Forward

| Component | Status |
|---|---|
| Stone placement preview UX | ✅ Carry forward |
| Pass / Resign controls | ✅ Carry forward |
| Board size selection (9×9, 13×13, 19×19) | ✅ Carry forward |
| Color selection (Black / White) | ✅ Carry forward |
| SGF review / load | ✅ Carry forward |
| Flask proxy | ❌ Replace with FastAPI |
| OGS API integration | ❌ Remove entirely |
| OGS authentication | ❌ Remove — replace with anonymous sessions |
| Flat HTML files | ❌ Restructure into frontend/ |

---

## Implementation Sequence

### Phase 1: Foundation (Copy & Adapt)
1. **Project scaffold** — Copy Cargo.toml from `../go`, update dependencies
2. **Game module** — Copy `src/game/` from `../go` (board.rs, rules.rs, types.rs, mod.rs)
3. **KataGo base** — Copy `src/katago/mod.rs` from `../go`, adapt for GTP instead of analysis mode
4. **Server scaffold** — Copy `src/main.rs`, `src/ws.rs`, `src/state.rs` from `../go`

### Phase 2: Spirit System (New)
5. **Spirit definitions** — Create `src/spirits/mod.rs` with Spirit enum and palette data
6. **Spirit config files** — Create all nine `.cfg` files in `configs/` based on `model_setup.md`
7. **Jaguar middleware** — Create `src/katago/jaguar.rs` with dynamic visit scaling
8. **Crow middleware** — Create `src/katago/crow.rs` with ko awareness and move biasing
9. **Spirit config loader** — Update KataGo spawn to load spirit-specific `.cfg` files

### Phase 3: Frontend (Copy & Adapt)
10. **Frontend scaffold** — Copy board rendering logic from `../phone_go/go-flow.html`
11. **Selection screen** — Create `frontend/index.html` with spirit portrait grid
12. **Game screen** — Create `frontend/game.html` with board + controls
13. **Theme system** — Create `frontend/css/themes.css` with all nine palettes
14. **WebSocket client** — Create `frontend/js/websocket.js` adapted from phone_go patterns

### Phase 4: Integration
15. **Session flow** — Connect spirit selection → game init → KataGo spawn → WebSocket upgrade
16. **Move flow** — Human move → validation → KataGo genmove → board update → WebSocket push
17. **Spirit behaviors** — Jaguar palette drift on move events, Crow board-dim on ko
18. **Character portraits** — Generate/integrate nine spirit portraits + Jaguar variants

### Phase 5: Polish
19. **Build script** — Create build.sh to compile frontend + Rust binary
20. **Deployment** — Single binary + configs + frontend assets + KataGo binaries
21. **Testing** — Play against all nine spirits, verify behaviors
