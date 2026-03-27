use axum::{
    Router,
    routing::get,
};
use tower_http::services::ServeDir;
use std::{net::SocketAddr, sync::Arc};

mod game;
mod katago;
mod state;
mod ws;

use state::AppState;

#[tokio::main]
async fn main() {
    // Initialize shared state
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/ws", get(ws::handler))
        .nest_service("/themes", ServeDir::new("themes"))
        .nest_service("/", ServeDir::new("frontend/dist"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on http://localhost:3000");
    println!("WebSocket endpoint: ws://localhost:3000/ws");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
