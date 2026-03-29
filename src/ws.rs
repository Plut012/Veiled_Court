use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::game::{Color, Position, Game};
use crate::katago::{KataGoProcess, jaguar};
use crate::spirits::Spirit;
use crate::state::{AppState, SessionData, SessionId};

/// Messages sent from client to server
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    InitGame {
        spirit: String,
        board_size: usize,
        player_color: String,
    },
    Move {
        coord: String, // e.g. "D4"
    },
    Pass,
    Resign,
}

/// Messages sent from server to client
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    GameStarted {
        session_id: String,
        board_size: usize,
    },
    BoardUpdate {
        board: Vec<Vec<Option<String>>>, // "black", "white", or null
        last_move: Option<String>,
        move_number: usize,
    },
    KoActive {
        threats: Vec<String>, // Crow only - placeholder for now
    },
    GameOver {
        winner: String,
    },
    Error {
        message: String,
    },
}

/// WebSocket connection handler
pub async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    println!("WebSocket connection established");

    let (mut sender, mut receiver) = socket.split();

    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            let response = handle_message(&state, &text).await;

            // Send response
            let response_json = serde_json::to_string(&response).unwrap();
            if sender.send(Message::Text(response_json)).await.is_err() {
                break;
            }
        }
    }

    println!("WebSocket connection closed");
}

/// Handle incoming message from client
async fn handle_message(state: &Arc<AppState>, text: &str) -> ServerMessage {
    let client_msg: ClientMessage = match serde_json::from_str(text) {
        Ok(msg) => msg,
        Err(e) => {
            return ServerMessage::Error {
                message: format!("Invalid message: {}", e),
            };
        }
    };

    match client_msg {
        ClientMessage::InitGame {
            spirit,
            board_size,
            player_color,
        } => handle_init_game(state, spirit, board_size, player_color).await,

        ClientMessage::Move { coord } => {
            // For now, we'll need to track session ID per connection
            // This is a simplified version - production would track session per WebSocket
            handle_move(state, coord).await
        }

        ClientMessage::Pass => {
            ServerMessage::Error {
                message: "Pass not yet implemented".to_string(),
            }
        }

        ClientMessage::Resign => {
            ServerMessage::Error {
                message: "Resign not yet implemented".to_string(),
            }
        }
    }
}

/// Handle InitGame message
async fn handle_init_game(
    state: &Arc<AppState>,
    spirit_str: String,
    board_size: usize,
    player_color_str: String,
) -> ServerMessage {
    // Parse spirit
    let spirit = match Spirit::from_string(&spirit_str) {
        Some(s) => s,
        None => {
            return ServerMessage::Error {
                message: format!("Invalid spirit: {}", spirit_str),
            };
        }
    };

    // Parse player color
    let player_color = match player_color_str.to_lowercase().as_str() {
        "black" => Color::Black,
        "white" => Color::White,
        _ => {
            return ServerMessage::Error {
                message: format!("Invalid color: {}", player_color_str),
            };
        }
    };

    // Validate board size
    if board_size != 9 && board_size != 13 && board_size != 19 {
        return ServerMessage::Error {
            message: "Board size must be 9, 13, or 19".to_string(),
        };
    }

    // Spawn KataGo with spirit config
    let config_path = spirit.config_file();
    let mut katago_process = match KataGoProcess::spawn(&config_path) {
        Ok(process) => process,
        Err(e) => {
            return ServerMessage::Error {
                message: format!("Failed to spawn KataGo: {}", e),
            };
        }
    };

    // Initialize KataGo board
    if let Err(e) = katago_process.set_boardsize(board_size) {
        return ServerMessage::Error {
            message: format!("Failed to set board size: {}", e),
        };
    }

    if let Err(e) = katago_process.clear_board() {
        return ServerMessage::Error {
            message: format!("Failed to clear board: {}", e),
        };
    }

    // Create game state
    let game_state = Game::with_size(board_size);

    // Generate session ID
    let session_id = Uuid::new_v4().to_string();

    // Create session data
    let session_data = SessionData {
        game_state,
        katago_process,
        spirit,
        board_size,
        move_number: 0,
        player_color,
    };

    // Store session
    let mut sessions = state.sessions.lock().await;
    sessions.insert(session_id.clone(), session_data);
    drop(sessions);

    // If bot plays first (player is white), generate bot move
    if player_color == Color::White {
        // Bot is black and plays first
        if let Err(e) = make_bot_move(state, &session_id).await {
            return ServerMessage::Error {
                message: format!("Bot failed to make opening move: {}", e),
            };
        }
    }

    ServerMessage::GameStarted {
        session_id,
        board_size,
    }
}

/// Handle Move message
/// Note: This simplified version assumes only one active session
/// Production version would track session ID per WebSocket connection
async fn handle_move(state: &Arc<AppState>, coord: String) -> ServerMessage {
    // Find the first (only) active session - simplified for MVP
    let session_id = {
        let sessions = state.sessions.lock().await;
        sessions.keys().next().cloned()
    };

    let session_id = match session_id {
        Some(id) => id,
        None => {
            return ServerMessage::Error {
                message: "No active game session".to_string(),
            };
        }
    };

    // Parse GTP coordinate
    let position = {
        let sessions = state.sessions.lock().await;
        let session = match sessions.get(&session_id) {
            Some(s) => s,
            None => {
                return ServerMessage::Error {
                    message: "Session not found".to_string(),
                };
            }
        };

        match KataGoProcess::parse_gtp_move(&coord, session.board_size) {
            Ok(pos) => pos,
            Err(e) => {
                return ServerMessage::Error {
                    message: format!("Invalid coordinate: {}", e),
                };
            }
        }
    };

    // Make human move
    if let Err(e) = make_human_move(state, &session_id, position).await {
        return ServerMessage::Error {
            message: format!("Invalid move: {}", e),
        };
    }

    // Make bot move
    if let Err(e) = make_bot_move(state, &session_id).await {
        return ServerMessage::Error {
            message: format!("Bot failed to respond: {}", e),
        };
    }

    // Return board update
    get_board_update(state, &session_id).await
}

/// Make a human move
async fn make_human_move(
    state: &Arc<AppState>,
    session_id: &SessionId,
    position: Position,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().await;
    let session = sessions.get_mut(session_id).ok_or("Session not found")?;

    // Validate it's the player's turn
    if session.game_state.get_turn() != session.player_color {
        return Err("Not your turn".to_string());
    }

    // Apply move to game state
    session.game_state.place_stone(position, session.player_color)?;

    // Send move to KataGo
    session
        .katago_process
        .play(session.player_color, position, session.board_size)?;

    // Increment move number
    session.move_number += 1;

    Ok(())
}

/// Make a bot move
async fn make_bot_move(
    state: &Arc<AppState>,
    session_id: &SessionId,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().await;
    let session = sessions.get_mut(session_id).ok_or("Session not found")?;

    let bot_color = session.player_color.opposite();

    // Generate bot move with spirit-specific logic
    let bot_position = match session.spirit {
        Spirit::Jaguar => {
            // Use dynamic visit scaling for Jaguar
            let visits = jaguar::get_jaguar_visits(session.move_number);
            session
                .katago_process
                .genmove_with_visits(bot_color, visits)?
        }
        _ => {
            // Standard genmove for other spirits
            session.katago_process.genmove(bot_color)?
        }
    };

    // Apply bot move to game state
    session.game_state.place_stone(bot_position, bot_color)?;

    // Increment move number
    session.move_number += 1;

    Ok(())
}

/// Get current board update for a session
async fn get_board_update(
    state: &Arc<AppState>,
    session_id: &SessionId,
) -> ServerMessage {
    let sessions = state.sessions.lock().await;
    let session = match sessions.get(session_id) {
        Some(s) => s,
        None => {
            return ServerMessage::Error {
                message: "Session not found".to_string(),
            };
        }
    };

    // Convert board to string representation
    let board_raw = session.game_state.get_board();
    let board: Vec<Vec<Option<String>>> = board_raw
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| {
                    cell.map(|color| match color {
                        Color::Black => "black".to_string(),
                        Color::White => "white".to_string(),
                    })
                })
                .collect()
        })
        .collect();

    ServerMessage::BoardUpdate {
        board,
        last_move: None, // TODO: Track last move
        move_number: session.move_number,
    }
}
