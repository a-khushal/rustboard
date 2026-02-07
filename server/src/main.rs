use axum::{
    extract::{ws::WebSocketUpgrade, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
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

use session::{ClientRole, Session, SessionManager};
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
    Query(query): Query<WebSocketQuery>,
    State(state): State<AppState>,
) -> Response {
    let token = match query.token {
        Some(token) if !token.is_empty() => token,
        _ => return (StatusCode::UNAUTHORIZED, "Missing session token").into_response(),
    };
    let role = match query.role.as_deref() {
        Some("viewer") => ClientRole::Viewer,
        _ => ClientRole::Editor,
    };

    let session = {
        let sessions = state.sessions.read().unwrap();
        sessions.get_session(&session_id)
    };
    let Some(session) = session else {
        return (StatusCode::NOT_FOUND, "Session not found").into_response();
    };
    if !session.validate_token_for_role(&token, role.clone()) {
        return (StatusCode::FORBIDDEN, "Invalid token for requested role").into_response();
    }

    ws.on_upgrade(|socket| handle_websocket(socket, session_id, role, state))
}

#[derive(Serialize)]
struct CreateSessionResponse {
    session_id: String,
    editor_token: String,
    viewer_token: String,
    editor_url: String,
    viewer_url: String,
}

#[derive(serde::Deserialize)]
struct WebSocketQuery {
    token: Option<String>,
    role: Option<String>,
}

async fn create_session(State(state): State<AppState>) -> axum::Json<CreateSessionResponse> {
    let session_id = Uuid::new_v4().to_string();
    let editor_token = Uuid::new_v4().to_string();
    let viewer_token = Uuid::new_v4().to_string();
    let mut sessions = state.sessions.write().unwrap();
    
    let document = Document::new();
    let session = Session::new(
        session_id.clone(),
        document,
        editor_token.clone(),
        viewer_token.clone(),
    );
    sessions.create_session(session);

    let editor_url = format!(
        "http://localhost:5173/?session={}&role=editor&token={}",
        session_id, editor_token
    );
    let viewer_url = format!(
        "http://localhost:5173/?session={}&role=viewer&token={}",
        session_id, viewer_token
    );
    
    axum::Json(CreateSessionResponse {
        session_id,
        editor_token,
        viewer_token,
        editor_url,
        viewer_url,
    })
}

#[derive(Serialize)]
struct GetSessionResponse {
    exists: bool,
    token_valid: bool,
}

async fn get_session(
    axum::extract::Path(session_id): axum::extract::Path<String>,
    Query(query): Query<WebSocketQuery>,
    State(state): State<AppState>,
) -> axum::Json<GetSessionResponse> {
    let sessions = state.sessions.read().unwrap();
    let session = sessions.get_session(&session_id);
    let exists = session.is_some();
    let token_valid = match (session, query.token) {
        (Some(session), Some(token)) if !token.is_empty() => session.validate_any_token(&token),
        _ => false,
    };
    axum::Json(GetSessionResponse { exists, token_valid })
}
