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
mod watchdog;
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

    // On RunPod: stop the pod after IDLE_TIMEOUT without WebSocket activity
    watchdog::spawn(state.clone());

    let app = Router::new()
        .route("/ws", get(ws::handler))
        .route("/health", get(health))
        .route("/activity", get(activity))
        .nest_service("/", ServeDir::new("frontend"))
        .with_state(state);

    // Bind dual-stack ([::] accepts IPv4 too on Linux) so localhost works over
    // both v4 and v6 — rootless podman's pasta forwards ::1 into the container
    let addr: SocketAddr = "[::]:3000".parse().unwrap();
    println!("Server running on http://localhost:3000");
    println!("WebSocket endpoint: ws://localhost:3000/ws");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
