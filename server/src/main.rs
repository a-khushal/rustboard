use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    routing::get,
    Router,
};
use rustboard_editor::Document;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

mod session;
mod websocket;

use session::{Session, SessionManager};
use websocket::handle_websocket;

#[derive(Clone)]
struct AppState {
    sessions: Arc<RwLock<SessionManager>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        sessions: Arc::new(RwLock::new(SessionManager::new())),
    };

    let app = Router::new()
        .route("/ws/:session_id", get(websocket_handler))
        .route("/api/sessions", get(create_session))
        .route("/api/sessions/:session_id", get(get_session))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("Server running on http://0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    axum::extract::Path(session_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, session_id, state))
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
    url: String,
}

async fn create_session(State(state): State<AppState>) -> axum::Json<CreateSessionResponse> {
    let session_id = Uuid::new_v4().to_string();
    let mut sessions = state.sessions.write().unwrap();
    
    let document = Document::new();
    let session = Session::new(session_id.clone(), document);
    sessions.create_session(session);

    let url = format!("http://localhost:5173/?session={}", session_id);
    
    axum::Json(CreateSessionResponse { session_id, url })
}

#[derive(Serialize)]
struct GetSessionResponse {
    exists: bool,
}

async fn get_session(
    axum::extract::Path(session_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> axum::Json<GetSessionResponse> {
    let sessions = state.sessions.read().unwrap();
    let exists = sessions.session_exists(&session_id);
    axum::Json(GetSessionResponse { exists })
}

