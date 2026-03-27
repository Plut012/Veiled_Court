use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade, Message}, State},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::game::{Color, Position, Group};
use crate::state::{AppState, PlayerConnection};

/// Messages sent from client to server
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    ChooseColor {
        color: Color,
    },
    Move { x: usize, y: usize },
    Pass,
    Reset { board_size: usize },
}

/// Messages sent from server to client
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    State {
        board: Vec<Vec<Option<Color>>>,
        board_size: usize,
        turn: Color,
        prisoners: Prisoners,
        players: Players,
        passes: u8,
        #[serde(skip_serializing_if = "Option::is_none")]
        ownership: Option<Vec<Vec<f32>>>,
        katago_available: bool,
        groups: Vec<Group>,
    },
    OwnershipUpdate {
        ownership: Vec<Vec<f32>>,
    },
    Error {
        message: String,
    },
    YourColor {
        color: Option<Color>,
    },
}

#[derive(Debug, Serialize)]
struct Prisoners {
    black: u32,
    white: u32,
}

#[derive(Debug, Serialize)]
struct Players {
    black: bool,
    white: bool,
}

/// WebSocket connection handler
pub async fn handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let conn_id = state.new_connection_id().await;
    println!("WebSocket connection established: {}", conn_id);

    // Split socket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Create channel for this connection
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Register connection
    {
        let mut connections = state.connections.lock().await;
        connections.insert(conn_id, PlayerConnection {
            color: None,
            sender: tx,
        });
    }

    // Send initial state and color assignment
    broadcast_state(&state).await;
    send_your_color(&state, conn_id).await;

    // Spawn task to forward messages from channel to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Text(text) = msg {
                handle_message(&state_clone, conn_id, &text).await;
            }
        }
    });

    // Wait for either task to finish (connection closed)
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Clean up connection
    let mut connections = state.connections.lock().await;
    connections.remove(&conn_id);
    println!("WebSocket connection closed: {}", conn_id);

    // Broadcast updated state (color is now available)
    drop(connections);
    broadcast_state(&state).await;
}

/// Broadcast current game state to all connections
async fn broadcast_state(state: &AppState) {
    let game = state.game.lock().await;
    let connections = state.connections.lock().await;

    // Determine which colors are assigned
    let mut black_assigned = false;
    let mut white_assigned = false;

    for conn in connections.values() {
        if conn.color == Some(Color::Black) {
            black_assigned = true;
        }
        if conn.color == Some(Color::White) {
            white_assigned = true;
        }
    }

    // Check if KataGo is available
    let katago_available = state.katago.lock().await.is_some();

    // Detect groups on the board
    let groups = game.detect_groups();

    let msg = ServerMessage::State {
        board: game.get_board(),
        board_size: game.get_board_size(),
        turn: game.get_turn(),
        prisoners: Prisoners {
            black: game.get_prisoners().0,
            white: game.get_prisoners().1,
        },
        players: Players {
            black: black_assigned,
            white: white_assigned,
        },
        passes: 0, // TODO: Track consecutive passes
        ownership: None, // Ownership sent separately via broadcast_ownership
        katago_available,
        groups,
    };

    let json = serde_json::to_string(&msg).unwrap();

    // Send to all connections
    for conn in connections.values() {
        let _ = conn.sender.send(json.clone());
    }
}

/// Broadcast ownership data asynchronously (non-blocking)
async fn broadcast_ownership(state: &AppState) {
    // Check if KataGo is available
    let katago_available = {
        let katago_guard = state.katago.lock().await;
        katago_guard.is_some()
    };

    if !katago_available {
        return;
    }

    // Get current board state
    let (board, board_size) = {
        let game = state.game.lock().await;
        (game.get_board(), game.get_board_size())
    };

    // Calculate ownership (this may take time)
    let ownership = {
        let mut katago_guard = state.katago.lock().await;
        if let Some(katago_service) = katago_guard.as_mut() {
            match katago_service.get_ownership(&board, board_size) {
                Ok(ownership_data) => Some(ownership_data.ownership),
                Err(e) => {
                    eprintln!("Failed to get ownership data: {}", e);
                    None
                }
            }
        } else {
            None
        }
    };

    // If we got ownership data, broadcast it
    if let Some(ownership_data) = ownership {
        let msg = ServerMessage::OwnershipUpdate {
            ownership: ownership_data,
        };

        let json = serde_json::to_string(&msg).unwrap();

        let connections = state.connections.lock().await;
        for conn in connections.values() {
            let _ = conn.sender.send(json.clone());
        }
    }
}

/// Send color assignment to a specific connection
async fn send_your_color(state: &AppState, conn_id: u64) {
    let connections = state.connections.lock().await;

    if let Some(conn) = connections.get(&conn_id) {
        let msg = ServerMessage::YourColor {
            color: conn.color,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let _ = conn.sender.send(json);
    }
}

/// Send error message to a specific connection
async fn send_error(state: &AppState, conn_id: u64, message: String) {
    let connections = state.connections.lock().await;

    if let Some(conn) = connections.get(&conn_id) {
        let msg = ServerMessage::Error { message };
        let json = serde_json::to_string(&msg).unwrap();
        let _ = conn.sender.send(json);
    }
}

/// Handle incoming message from client
async fn handle_message(state: &Arc<AppState>, conn_id: u64, text: &str) {
    let client_msg: ClientMessage = match serde_json::from_str(text) {
        Ok(msg) => msg,
        Err(e) => {
            send_error(state, conn_id, format!("Invalid message: {}", e)).await;
            return;
        }
    };

    match client_msg {
        ClientMessage::ChooseColor { color } => {
            handle_choose_color(state, conn_id, color).await;
        }
        ClientMessage::Move { x, y } => {
            handle_move(state, conn_id, x, y).await;
        }
        ClientMessage::Pass => {
            handle_pass(state, conn_id).await;
        }
        ClientMessage::Reset { board_size } => {
            handle_reset(state, board_size).await;
        }
    }
}

/// Handle color selection
async fn handle_choose_color(state: &AppState, conn_id: u64, color: Color) {
    let mut connections = state.connections.lock().await;

    // Check if color is already taken
    let color_taken = connections.values().any(|conn| conn.color == Some(color));

    if color_taken {
        drop(connections);
        send_error(state, conn_id, "Color already taken".to_string()).await;
        return;
    }

    // Assign color to the player
    if let Some(conn) = connections.get_mut(&conn_id) {
        conn.color = Some(color);
    }

    drop(connections);

    // Broadcast updated state
    broadcast_state(state).await;
    send_your_color(state, conn_id).await;
}

/// Handle move attempt
async fn handle_move(state: &Arc<AppState>, conn_id: u64, x: usize, y: usize) {
    let connections = state.connections.lock().await;

    // Get player's color
    let color = match connections.get(&conn_id).and_then(|c| c.color) {
        Some(c) => c,
        None => {
            drop(connections);
            send_error(state, conn_id, "You must choose a color first".to_string()).await;
            return;
        }
    };

    drop(connections);

    // Attempt move
    let mut game = state.game.lock().await;
    let result = game.place_stone(Position::new(x, y), color);

    drop(game);

    match result {
        Ok(()) => {
            broadcast_state(state).await;

            // Spawn async task to calculate and broadcast ownership (non-blocking)
            let state_clone = state.clone();
            tokio::spawn(async move {
                broadcast_ownership(&state_clone).await;
            });
        }
        Err(e) => {
            send_error(state, conn_id, e).await;
        }
    }
}

/// Handle pass
async fn handle_pass(state: &Arc<AppState>, conn_id: u64) {
    let connections = state.connections.lock().await;

    // Get player's color
    let color = match connections.get(&conn_id).and_then(|c| c.color) {
        Some(c) => c,
        None => {
            drop(connections);
            send_error(state, conn_id, "You must choose a color first".to_string()).await;
            return;
        }
    };

    drop(connections);

    let mut game = state.game.lock().await;

    // Check if it's their turn
    if game.get_turn() != color {
        drop(game);
        send_error(state, conn_id, "Not your turn".to_string()).await;
        return;
    }

    game.pass();
    drop(game);

    broadcast_state(state).await;

    // Spawn async task to calculate and broadcast ownership (non-blocking)
    let state_clone = state.clone();
    tokio::spawn(async move {
        broadcast_ownership(&state_clone).await;
    });
}

/// Handle game reset
async fn handle_reset(state: &AppState, board_size: usize) {
    let mut game = state.game.lock().await;
    game.reset_with_size(board_size);
    drop(game);

    // Clear color assignments
    let mut connections = state.connections.lock().await;
    for conn in connections.values_mut() {
        conn.color = None;
    }
    drop(connections);

    broadcast_state(state).await;
}

