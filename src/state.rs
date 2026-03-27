use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use crate::game::{Game, Color};
use crate::katago::{KataGoService, KataGoConfig};

/// Unique identifier for each WebSocket connection
pub type ConnectionId = u64;

/// Information about a connected player
pub struct PlayerConnection {
    pub color: Option<Color>,
    pub sender: mpsc::UnboundedSender<String>, // Channel to send messages to this connection
}

/// Shared application state
pub struct AppState {
    pub game: Arc<Mutex<Game>>,
    pub connections: Arc<Mutex<HashMap<ConnectionId, PlayerConnection>>>,
    pub next_connection_id: Arc<Mutex<ConnectionId>>,
    pub katago: Arc<Mutex<Option<KataGoService>>>,
}

impl AppState {
    pub fn new() -> Self {
        // Try to initialize KataGo service (graceful fallback if unavailable)
        let katago_service = match KataGoService::new(KataGoConfig::default()) {
            Ok(service) => {
                println!("✓ KataGo service initialized successfully");
                Some(service)
            }
            Err(e) => {
                println!("⚠ KataGo not available: {}", e);
                println!("  AI opponent and territory estimation features will be disabled");
                None
            }
        };

        Self {
            game: Arc::new(Mutex::new(Game::new())),
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_connection_id: Arc::new(Mutex::new(0)),
            katago: Arc::new(Mutex::new(katago_service)),
        }
    }

    /// Generate a unique connection ID
    pub async fn new_connection_id(&self) -> ConnectionId {
        let mut id = self.next_connection_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }

    /// Check if KataGo service is available
    pub async fn has_katago(&self) -> bool {
        self.katago.lock().await.is_some()
    }
}
