use axum::{
    Router,
    routing::get,
    extract::State,
    response::Json,
};
use tower_http::services::ServeDir;
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};

mod game;
mod katago;
mod spirits;
mod state;
mod ws;

use state::AppState;

async fn health() -> &'static str {
    "ok"
}

async fn activity(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let ts = state.last_activity_ts();
    let now = chrono::Utc::now().timestamp();
    Json(json!({
        "last_activity": ts,
        "idle_seconds": now - ts
    }))
}

#[tokio::main]
async fn main() {
    println!("Veiled Court — starting server...");

    // Initialize shared state
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/ws", get(ws::handler))
        .route("/health", get(health))
        .route("/activity", get(activity))
        .nest_service("/", ServeDir::new("frontend"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on http://localhost:3000");
    println!("WebSocket endpoint: ws://localhost:3000/ws");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
