use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::sync::Mutex;

use crate::game::{Game, Color};
use crate::katago::KataGoProcess;
use crate::spirits::Spirit;

/// Unique identifier for each game session
pub type SessionId = String;

/// Session data for a single-player game against KataGo
pub struct SessionData {
    pub game_state: Game,
    pub katago_process: KataGoProcess,
    pub spirit: Spirit,
    pub board_size: usize,
    pub move_number: usize,
    pub player_color: Color, // Human's color
    pub last_move: Option<String>, // GTP coordinate of most recent move
}

/// Shared application state
pub struct AppState {
    /// Active game sessions (session_id -> SessionData)
    pub sessions: Arc<Mutex<HashMap<SessionId, SessionData>>>,
    /// Unix timestamp of last WebSocket activity (for idle detection)
    pub last_activity: AtomicI64,
}

impl AppState {
    pub fn new() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            last_activity: AtomicI64::new(now),
        }
    }

    pub fn touch(&self) {
        let now = chrono::Utc::now().timestamp();
        self.last_activity.store(now, Ordering::Relaxed);
    }

    pub fn last_activity_ts(&self) -> i64 {
        self.last_activity.load(Ordering::Relaxed)
    }
}
