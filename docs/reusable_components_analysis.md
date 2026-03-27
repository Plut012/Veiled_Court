# Reusable Components Analysis

## Overview

After reviewing the vision documents for `animal_go` and examining your two existing Go projects (`phone_go` and `go`), there are substantial architectural and code components that can be directly reused or adapted for the Spirit Animals project.

---

## High-Value Reusable Components

### 1. **Go Rules Engine** (from `go` Rust project)

**Location:** `../go/src/game/board.rs`, `../go/src/game/rules.rs`

**What it provides:**
- Complete board state management
- Move validation (legal placement, suicide detection)
- Capture detection and resolution (group finding, liberty counting)
- Ko rule detection using position hashing
- Comprehensive test coverage

**Why it's valuable:**
Your architecture doc explicitly states:

> **New component — do not write from scratch**
> A Go rules library handles move validation, ko detection, capture resolution, and scoring. Writing this correctly from scratch is a significant undertaking with many edge cases.

**Adaptation needed:**
- **Language:** The Rust implementation needs to be ported to Python OR you consider using Rust for the backend instead of FastAPI
- **Alternative:** If staying with Python/FastAPI, you could:
  1. Use the recommended `sgfmill` library as originally planned
  2. Extract the logic patterns from the Rust code as reference for Python implementation
  3. Port the Rust code to a Python extension module (advanced)

**Recommendation:** Use `sgfmill` as planned, but reference the Rust implementation's test cases to ensure complete coverage.

---

### 2. **KataGo Process Management & GTP Communication** (from `go` Rust project)

**Location:** `../go/src/katago/mod.rs`

**What it provides:**
- Subprocess spawn and lifecycle management
- GTP protocol communication (stdin/stdout)
- Position to GTP coordinate conversion (e.g., `(3,3)` → `"D4"`)
- Position hashing for caching analysis results
- Query/response serialization
- Process cleanup on drop

**Why it's valuable:**
This is the exact foundation needed for the KataGo Process Manager component in your architecture. The Rust implementation is production-tested and handles edge cases.

**Adaptation needed:**
- Port to Python using `subprocess.Popen` for process management
- Use `asyncio` for async communication with KataGo stdin/stdout
- The GTP coordinate conversion logic is language-agnostic and can be directly ported

**Reusable patterns:**
```python
# Direct port of GTP coordinate conversion
def position_to_gtp(x: int, y: int, board_size: int) -> str:
    col_char = chr(ord('A') + x) if x < 8 else chr(ord('A') + x + 1)  # Skip 'I'
    row_num = board_size - y
    return f"{col_char}{row_num}"

# Process management pattern (adapted to Python)
class KataGoProcess:
    def __init__(self, config_path: str):
        self.process = subprocess.Popen(
            ['katago', 'gtp', '-config', config_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )

    async def send_command(self, command: str) -> str:
        self.process.stdin.write(f"{command}\n")
        self.process.stdin.flush()
        return await self.read_response()
```

---

### 3. **Frontend UX Patterns** (from `phone_go`)

**Location:** `../phone_go/go-flow.html`

**What it provides:**
- Stone placement preview (semi-transparent stone on tap)
- Golden submit bar confirmation UX
- Pass / Resign button layout
- Board size selection dropdown (9×9, 13×13, 19×19)
- Color selection (Black / White)
- Theme selection UI pattern
- Mobile-optimized touch interaction
- PWA manifest structure

**Why it's valuable:**
Your architecture doc explicitly states:

> **Carries forward from current codebase with significant restructuring**
> Stone placement preview (existing behavior carried forward)
> Pass / Resign controls (existing behavior carried forward)
> Board size selection (9×9, 13×13, 19×19) (existing behavior carried forward)

**Reusable code:**
- CSS styles for mobile-first design
- Canvas-based board rendering logic
- Touch event handlers for stone placement preview
- Submit bar interaction pattern
- Screen transition patterns (selection → game)

**Adaptation needed:**
- Restructure from flat HTML into component-based architecture
- Replace OGS WebSocket client with local game server WebSocket
- Integrate spirit selection screen
- Add theming system with CSS custom properties

---

### 4. **WebSocket Architecture** (from both projects)

**Phone_go (client side):**
- Socket.IO client integration
- Event-based message handling
- Connection state management
- Reconnect handling

**Go (server side - Rust/Axum):**
- WebSocket handler setup
- Message routing
- Game state synchronization
- Concurrent connection management

**Why it's valuable:**
Your architecture requires WebSocket for real-time move communication between browser and server. Both projects provide proven patterns.

**Reusable patterns:**
- Message format: `{ type: "move", coord: "D4", color: "black" }`
- Board state updates: `{ type: "board_update", board: [...], last_move: "Q16", move_number: N }`
- Connection lifecycle management
- Error handling and reconnection logic

**Adaptation needed:**
- If using Python/FastAPI: Use `fastapi.WebSocket` instead of Axum
- If using Rust: Adapt Axum patterns directly
- Integrate custom events (ko flags, Jaguar palette drift updates)

---

### 5. **Theming System Architecture** (from `go`)

**Location:** `../go/themes/`, frontend Svelte components

**What it provides:**
- CSS custom properties for palette tokens
- Theme switching mechanism
- SVG-based rendering that's theme-independent
- Asset organization pattern

**Why it's valuable:**
Your theming doc describes exactly this pattern:

> **Palette as CSS variables** — All six color values per spirit stored as CSS custom properties on the root element, swapped atomically on spirit selection.

The Rust project already implements this architecture successfully.

**Reusable patterns:**
```css
:root {
  --board-primary: #0D1F1A;
  --board-secondary: #2E6B55;
  --accent: #C9A84C;
  --stone-black-tint: ...;
  --stone-white-tint: ...;
}
```

**Adaptation needed:**
- Expand to 9 spirit palettes (Dragon, Mantis Shrimp, Crane, etc.)
- Add Jaguar palette drift interpolation
- Add Crow board-dim effect on ko detection

---

### 6. **Session Management Pattern** (from `go`)

**Location:** `../go/src/state.rs`

**What it provides:**
- In-memory game state storage
- Session ID to game state mapping
- Thread-safe state access (Arc<Mutex>)

**Why it's valuable:**
Your architecture requires:

> Sessions are in-memory. No database required for single-player vs bot games.

```python
sessions = {
  "session_id": {
    "game_state": GameState,
    "katago_process": KataGoProcess,
    "spirit": "jaguar"
  }
}
```

**Adaptation needed:**
- Port to Python using `dict` with async locks
- Add KataGo process handle to session
- Add spirit configuration reference

---

## What CANNOT Be Reused (Needs New Implementation)

### 1. **Spirit-Specific Middleware**
- Jaguar dynamic visit scaling (new)
- Crow ko awareness middleware (new)
- Both are custom logic unique to this project

### 2. **Spirit Configuration Files**
- Nine KataGo `.cfg` files with specific parameters
- These are new but straightforward to create based on `model_setup.md`

### 3. **Character Portraits & Visual Assets**
- All nine spirit portraits
- Jaguar warm/cold variants
- Icon crops
- Completely new artistic assets

### 4. **Spirit Selection Screen**
- New UI component
- Portrait grid layout
- Palette preview on hover
- Session initialization flow

---

## Technology Stack Decision Point

### **Critical Choice: Python vs Rust Backend**

Your architecture doc specifies **FastAPI (Python)**, but your existing `go` project demonstrates a complete, production-tested **Rust + Axum** implementation.

#### Option A: Python/FastAPI (as specified)
**Pros:**
- Matches architecture doc
- Natural upgrade from `phone_go` Flask proxy
- Large ecosystem for KataGo integration
- Easier prototyping

**Cons:**
- Need to rewrite Go rules engine (or use `sgfmill`)
- Need to port KataGo process management
- Less type safety than Rust

**Reuse strategy:**
- Use `sgfmill` for rules
- Port KataGo patterns to Python
- Reuse frontend UX patterns directly
- Reuse theming CSS architecture

#### Option B: Rust/Axum (adapt from `go` project)
**Pros:**
- Reuse battle-tested rules engine directly
- Reuse KataGo integration directly
- Reuse WebSocket server directly
- Better performance, type safety

**Cons:**
- Diverges from architecture doc
- Steeper learning curve if less familiar with Rust

**Reuse strategy:**
- Copy entire `game/` module (rules engine)
- Copy entire `katago/` module
- Adapt `ws.rs` and `state.rs`
- Add Jaguar/Crow middleware to `katago/`
- Reuse frontend patterns

---

## Recommended Reuse Strategy

### **Phase 1: Foundation (High Reuse)**

1. **Rules Engine:**
   - Use `sgfmill` (Python) as planned
   - Reference Rust test cases for validation coverage

2. **KataGo Process Manager:**
   - Port Rust patterns to Python/asyncio
   - Directly copy GTP coordinate conversion logic
   - Directly copy position hashing algorithm

3. **Frontend Scaffolding:**
   - Copy stone placement preview UX from `phone_go`
   - Copy pass/resign button layout
   - Copy board size/color selection patterns

4. **Theming CSS:**
   - Copy CSS custom property architecture from `go`
   - Expand to 9 spirit palettes

### **Phase 2: Custom Logic (Low Reuse)**

1. **Spirit Middleware:**
   - Implement Jaguar visit scaling (new)
   - Implement Crow ko awareness (new)

2. **Spirit Selection Screen:**
   - New component (no equivalent in existing projects)

3. **Character Portraits:**
   - Commission/generate all new assets

### **Phase 3: Integration (Medium Reuse)**

1. **WebSocket Flow:**
   - Adapt patterns from both projects
   - Integrate custom events (ko flags, palette updates)

2. **Session Manager:**
   - Port patterns from Rust `state.rs`
   - Add spirit configuration mapping

---

## File-by-File Reuse Checklist

### From `../go/src/`:

| File | Reusable? | How to Reuse |
|------|-----------|--------------|
| `game/board.rs` | ✅ High | Port to Python or use sgfmill, reference tests |
| `game/rules.rs` | ✅ High | Port logic patterns, especially ko detection |
| `katago/mod.rs` | ✅ Very High | Port entire structure to Python KataGoProcess |
| `ws.rs` | ⚠️ Medium | Adapt WebSocket patterns to FastAPI |
| `state.rs` | ⚠️ Medium | Adapt session management pattern |
| `main.rs` | ⚠️ Low | Reference server structure, rewrite for FastAPI |

### From `../phone_go/`:

| File | Reusable? | How to Reuse |
|------|-----------|--------------|
| `go-flow.html` (lines 1-100) | ✅ Very High | Copy UX patterns, restructure into components |
| Stone preview logic | ✅ Very High | Copy directly |
| Canvas rendering | ✅ High | Adapt for theming system |
| Pass/Resign controls | ✅ Very High | Copy directly |
| Board size selection | ✅ Very High | Copy directly |
| `proxy.py` WebSocket client | ⚠️ Medium | Adapt patterns for local server |

---

## Conclusion

**Reuse potential: ~60-70% of implementation**

The highest-value reusable components are:

1. **KataGo process management patterns** (Rust → Python port)
2. **Frontend UX flows** (stone preview, controls, selection)
3. **Theming architecture** (CSS custom properties)
4. **Rules engine test coverage** (reference for validation)

The components requiring new implementation are:

1. Spirit-specific middleware (Jaguar, Crow)
2. Spirit selection screen
3. Visual assets (portraits)
4. Nine KataGo config files

This analysis suggests you can accelerate development significantly by porting proven patterns rather than building from scratch, while still implementing the unique "Spirit Animals" concept that makes this project distinctive.
