use axum::{
    Json,
    extract::connect_info::ConnectInfo,
    extract::{ws::WebSocketUpgrade, Query, State},
    http::{HeaderMap, HeaderValue, Method},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use rustboard_editor::Document;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::time::Duration;
use tower_http::cors::{AllowOrigin, CorsLayer};
use uuid::Uuid;

mod session;
mod websocket;

use session::{ClientRole, SessionManager};
use websocket::handle_websocket;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) sessions: Arc<RwLock<SessionManager>>,
    pub(crate) metrics: Arc<AppMetrics>,
    pub(crate) create_session_limiter: Arc<RateLimiter>,
    pub(crate) ws_connect_limiter: Arc<RateLimiter>,
    pub(crate) session_lookup_limiter: Arc<RateLimiter>,
    pub(crate) revoke_token_limiter: Arc<RateLimiter>,
    pub(crate) allowed_origins: Arc<Vec<String>>,
}

struct AppMetrics {
    sessions_created: AtomicU64,
    ws_connections: AtomicU64,
    ws_disconnections: AtomicU64,
    ws_errors: AtomicU64,
    operations_applied: AtomicU64,
    full_syncs_sent: AtomicU64,
    rate_limited_requests: AtomicU64,
    token_revocations: AtomicU64,
    token_rotations: AtomicU64,
}

impl AppMetrics {
    fn new() -> Self {
        Self {
            sessions_created: AtomicU64::new(0),
            ws_connections: AtomicU64::new(0),
            ws_disconnections: AtomicU64::new(0),
            ws_errors: AtomicU64::new(0),
            operations_applied: AtomicU64::new(0),
            full_syncs_sent: AtomicU64::new(0),
            rate_limited_requests: AtomicU64::new(0),
            token_revocations: AtomicU64::new(0),
            token_rotations: AtomicU64::new(0),
        }
    }
}

struct RateLimiter {
    window: Duration,
    max_requests: usize,
    events: std::sync::Mutex<HashMap<String, VecDeque<Instant>>>,
}

fn configured_allowed_origins() -> Vec<String> {
    let allowed = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
    if allowed.trim().is_empty() {
        return vec![
            "http://localhost:5173".to_string(),
            "http://127.0.0.1:5173".to_string(),
        ];
    }
    allowed
        .split(',')
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn build_cors_layer(origins: &[String]) -> CorsLayer {
    let header_values: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    let allow_origin = if header_values.is_empty() {
        AllowOrigin::exact(HeaderValue::from_static("http://localhost:5173"))
    } else {
        AllowOrigin::list(header_values)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION])
}

fn origin_allowed(headers: &HeaderMap, allowed_origins: &[String]) -> bool {
    if allowed_origins.is_empty() {
        return true;
    }
    let Some(origin) = headers.get("origin") else {
        return false;
    };
    let Ok(origin_value) = origin.to_str() else {
        return false;
    };
    allowed_origins.iter().any(|allowed_origin| allowed_origin == origin_value)
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            window,
            max_requests,
            events: std::sync::Mutex::new(HashMap::new()),
        }
    }

    fn allow(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut events = self.events.lock().unwrap();
        let queue = events.entry(key.to_string()).or_default();
        while let Some(timestamp) = queue.front() {
            if now.duration_since(*timestamp) > self.window {
                queue.pop_front();
            } else {
                break;
            }
        }
        if queue.len() >= self.max_requests {
            return false;
        }
        queue.push_back(now);
        true
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3001);

    let session_ttl_secs = std::env::var("SESSION_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(60 * 60 * 24);
    let session_store_path = std::env::var("SESSION_STORE_PATH")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("server_data/sessions.json"));
    let token_ttl_secs = std::env::var("SESSION_TOKEN_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(60 * 60 * 24 * 14);
    let allowed_origins = configured_allowed_origins();

    let state = AppState {
        sessions: Arc::new(RwLock::new(SessionManager::new_with_persistence(
            session_store_path,
            session_ttl_secs,
            token_ttl_secs,
        ))),
        metrics: Arc::new(AppMetrics::new()),
        create_session_limiter: Arc::new(RateLimiter::new(10, Duration::from_secs(60))),
        ws_connect_limiter: Arc::new(RateLimiter::new(60, Duration::from_secs(60))),
        session_lookup_limiter: Arc::new(RateLimiter::new(120, Duration::from_secs(60))),
        revoke_token_limiter: Arc::new(RateLimiter::new(30, Duration::from_secs(60))),
        allowed_origins: Arc::new(allowed_origins.clone()),
    };

    let sessions_for_maintenance = state.sessions.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let mut sessions = sessions_for_maintenance.write().unwrap();
            let removed = sessions.cleanup_expired_sessions();
            if removed > 0 {
                tracing::info!("Cleaned up {removed} expired sessions");
            }
            if let Err(err) = sessions.persist_all() {
                tracing::warn!("Failed to persist sessions: {err}");
            }
        }
    });

    let app = Router::new()
        .route("/ws/:session_id", get(websocket_handler))
        .route("/api/sessions", get(create_session))
        .route("/api/sessions/:session_id", get(get_session))
        .route("/api/sessions/:session_id/revoke", post(revoke_session_token))
        .route("/api/sessions/:session_id/rotate", post(rotate_session_token))
        .route("/api/sessions/:session_id/invite", post(issue_invite_token))
        .route("/healthz", get(healthz))
        .route("/metrics", get(metrics))
        .layer(build_cors_layer(&allowed_origins))
        .with_state(state);

    let bind_address = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    tracing::info!("Server running on http://{bind_address}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
    Query(query): Query<WebSocketQuery>,
    State(state): State<AppState>,
) -> Response {
    if !origin_allowed(&headers, &state.allowed_origins) {
        return (StatusCode::FORBIDDEN, "Origin not allowed").into_response();
    }

    let client_ip = addr.ip().to_string();
    if !state.ws_connect_limiter.allow(&client_ip) {
        state
            .metrics
            .rate_limited_requests
            .fetch_add(1, Ordering::Relaxed);
        return (StatusCode::TOO_MANY_REQUESTS, "Too many websocket attempts").into_response();
    }

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
        sessions.mark_session_active(&session_id);
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

async fn create_session(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> Result<axum::Json<CreateSessionResponse>, (StatusCode, &'static str)> {
    let client_ip = addr.ip().to_string();
    if !state.create_session_limiter.allow(&client_ip) {
        state
            .metrics
            .rate_limited_requests
            .fetch_add(1, Ordering::Relaxed);
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many session creations"));
    }

    let session_id = Uuid::new_v4().to_string();
    let mut sessions = state.sessions.write().unwrap();

    let document = Document::new();
    let session = sessions.create_new_session(session_id.clone(), document);
    let editor_token = session.editor_token.clone();
    let viewer_token = session.viewer_token.clone();
    state
        .metrics
        .sessions_created
        .fetch_add(1, Ordering::Relaxed);

    let editor_url = format!(
        "http://localhost:5173/?session={}&role=editor&token={}",
        session_id, editor_token
    );
    let viewer_url = format!(
        "http://localhost:5173/?session={}&role=viewer&token={}",
        session_id, viewer_token
    );
    
    Ok(axum::Json(CreateSessionResponse {
        session_id,
        editor_token,
        viewer_token,
        editor_url,
        viewer_url,
    }))
}

#[derive(Serialize)]
struct GetSessionResponse {
    exists: bool,
    token_valid: bool,
}

async fn get_session(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
    Query(query): Query<WebSocketQuery>,
    State(state): State<AppState>,
) -> Result<axum::Json<GetSessionResponse>, (StatusCode, &'static str)> {
    let client_ip = addr.ip().to_string();
    if !state.session_lookup_limiter.allow(&client_ip) {
        state
            .metrics
            .rate_limited_requests
            .fetch_add(1, Ordering::Relaxed);
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many session lookups"));
    }

    let sessions = state.sessions.read().unwrap();
    sessions.mark_session_active(&session_id);
    let session = sessions.get_session(&session_id);
    let exists = session.is_some();
    let token_valid = match (session, query.token) {
        (Some(session), Some(token)) if !token.is_empty() => session.validate_any_token(&token),
        _ => false,
    };
    Ok(axum::Json(GetSessionResponse { exists, token_valid }))
}

#[derive(serde::Deserialize)]
struct RevokeTokenRequest {
    editor_token: String,
    token_to_revoke: String,
}

#[derive(serde::Deserialize)]
struct RotateTokenRequest {
    editor_token: String,
    role: String,
}

#[derive(serde::Deserialize)]
struct InviteTokenRequest {
    editor_token: String,
    role: String,
    ttl_secs: Option<u64>,
}

#[derive(Serialize)]
struct RevokeTokenResponse {
    revoked: bool,
}

#[derive(Serialize)]
struct RotateTokenResponse {
    rotated: bool,
    token: Option<String>,
}

#[derive(Serialize)]
struct InviteTokenResponse {
    issued: bool,
    token: Option<String>,
}

async fn revoke_session_token(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<RevokeTokenRequest>,
) -> Result<Json<RevokeTokenResponse>, (StatusCode, &'static str)> {
    let client_ip = addr.ip().to_string();
    if !state.revoke_token_limiter.allow(&client_ip) {
        state
            .metrics
            .rate_limited_requests
            .fetch_add(1, Ordering::Relaxed);
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many token operations"));
    }

    let mut sessions = state.sessions.write().unwrap();
    let Some(session) = sessions.get_session(&session_id) else {
        return Err((StatusCode::NOT_FOUND, "Session not found"));
    };

    if !session.validate_token_for_role(&payload.editor_token, ClientRole::Editor) {
        return Err((StatusCode::FORBIDDEN, "Editor token required"));
    }

    let revoked = sessions.revoke_token(&session_id, &payload.token_to_revoke);
    if revoked {
        state
            .metrics
            .token_revocations
            .fetch_add(1, Ordering::Relaxed);
        tracing::info!("Token revoked for session {session_id}");
    }
    Ok(Json(RevokeTokenResponse { revoked }))
}

async fn rotate_session_token(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<RotateTokenRequest>,
) -> Result<Json<RotateTokenResponse>, (StatusCode, &'static str)> {
    let client_ip = addr.ip().to_string();
    if !state.revoke_token_limiter.allow(&client_ip) {
        state
            .metrics
            .rate_limited_requests
            .fetch_add(1, Ordering::Relaxed);
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many token operations"));
    }

    let mut sessions = state.sessions.write().unwrap();
    let Some(session) = sessions.get_session(&session_id) else {
        return Err((StatusCode::NOT_FOUND, "Session not found"));
    };

    if !session.validate_token_for_role(&payload.editor_token, ClientRole::Editor) {
        return Err((StatusCode::FORBIDDEN, "Editor token required"));
    }

    let token = match payload.role.as_str() {
        "viewer" => sessions.rotate_viewer_token(&session_id),
        "editor" => sessions.rotate_editor_token(&session_id),
        _ => return Err((StatusCode::BAD_REQUEST, "role must be viewer or editor")),
    };

    let rotated = token.is_some();
    if rotated {
        state
            .metrics
            .token_rotations
            .fetch_add(1, Ordering::Relaxed);
        tracing::info!("Token rotated for session {session_id}, role={}", payload.role);
    }

    Ok(Json(RotateTokenResponse { rotated, token }))
}

async fn issue_invite_token(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<InviteTokenRequest>,
) -> Result<Json<InviteTokenResponse>, (StatusCode, &'static str)> {
    let client_ip = addr.ip().to_string();
    if !state.revoke_token_limiter.allow(&client_ip) {
        state
            .metrics
            .rate_limited_requests
            .fetch_add(1, Ordering::Relaxed);
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many token operations"));
    }

    let mut sessions = state.sessions.write().unwrap();
    let Some(session) = sessions.get_session(&session_id) else {
        return Err((StatusCode::NOT_FOUND, "Session not found"));
    };

    if !session.validate_token_for_role(&payload.editor_token, ClientRole::Editor) {
        return Err((StatusCode::FORBIDDEN, "Editor token required"));
    }

    let role = match payload.role.as_str() {
        "viewer" => ClientRole::Viewer,
        "editor" => ClientRole::Editor,
        _ => return Err((StatusCode::BAD_REQUEST, "role must be viewer or editor")),
    };
    let ttl_secs = payload.ttl_secs.map(|ttl| ttl.clamp(60, 60 * 60 * 24 * 30));
    let token = sessions.issue_invite_token(&session_id, role, ttl_secs);
    let issued = token.is_some();
    if issued {
        tracing::info!("Invite token issued for session {session_id}");
    }
    Ok(Json(InviteTokenResponse { issued, token }))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn healthz() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse { status: "ok" })
}

#[derive(Serialize)]
struct MetricsResponse {
    active_sessions: usize,
    sessions_created: u64,
    ws_connections: u64,
    ws_disconnections: u64,
    ws_errors: u64,
    operations_applied: u64,
    full_syncs_sent: u64,
    rate_limited_requests: u64,
    token_revocations: u64,
    token_rotations: u64,
}

async fn metrics(State(state): State<AppState>) -> axum::Json<MetricsResponse> {
    let active_sessions = {
        let sessions = state.sessions.read().unwrap();
        sessions.session_count()
    };
    axum::Json(MetricsResponse {
        active_sessions,
        sessions_created: state.metrics.sessions_created.load(Ordering::Relaxed),
        ws_connections: state.metrics.ws_connections.load(Ordering::Relaxed),
        ws_disconnections: state.metrics.ws_disconnections.load(Ordering::Relaxed),
        ws_errors: state.metrics.ws_errors.load(Ordering::Relaxed),
        operations_applied: state.metrics.operations_applied.load(Ordering::Relaxed),
        full_syncs_sent: state.metrics.full_syncs_sent.load(Ordering::Relaxed),
        rate_limited_requests: state.metrics.rate_limited_requests.load(Ordering::Relaxed),
        token_revocations: state.metrics.token_revocations.load(Ordering::Relaxed),
        token_rotations: state.metrics.token_rotations.load(Ordering::Relaxed),
    })
}
