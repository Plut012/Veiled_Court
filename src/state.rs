use std::collections::HashMap;
use std::sync::Arc;
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
    pub move_history: Vec<(String, String)>, // (color, gtp_coord) per move
}

/// Data preserved after game end for review analysis
pub struct PendingReview {
    pub move_history: Vec<(String, String)>,
    pub board_size: usize,
}

/// Shared application state
pub struct AppState {
    /// Active game sessions (session_id -> SessionData)
    pub sessions: Arc<Mutex<HashMap<SessionId, SessionData>>>,
    /// Move history saved after game end, awaiting review request
    pub pending_review: Arc<Mutex<Option<PendingReview>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            pending_review: Arc::new(Mutex::new(None)),
        }
    }
}
