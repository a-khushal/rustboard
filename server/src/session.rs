use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use rustboard_editor::Document;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::websocket::ServerMessage;

fn now_unix_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

const SESSION_STORE_SCHEMA_VERSION: u32 = 2;
const SESSION_DOCUMENT_SCHEMA_VERSION: u32 = 1;
const DEFAULT_TOKEN_TTL_SECS: u64 = 60 * 60 * 24 * 14;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedTokenPayload {
    sid: String,
    role: ClientRole,
    exp: u64,
    jti: String,
    version: u32,
}

#[derive(Clone)]
pub struct Session {
    pub id: String,
    pub document: Arc<RwLock<Document>>,
    pub clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
    pub client_roles: Arc<RwLock<HashMap<String, ClientRole>>>,
    pub client_id_maps: Arc<RwLock<HashMap<String, HashMap<u64, u64>>>>,
    pub operation_seq: Arc<AtomicU64>,
    pub broadcast_tx: Arc<broadcast::Sender<ServerMessage>>,
    pub editor_token: String,
    pub viewer_token: String,
    pub token_secret: String,
    pub token_ttl_secs: u64,
    pub revoked_token_ids: Arc<RwLock<HashSet<String>>>,
    pub allow_legacy_tokens: bool,
    pub created_at: Arc<AtomicU64>,
    pub last_active_at: Arc<AtomicU64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClientRole {
    Editor,
    Viewer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: String,
    pub name: String,
    pub color: String,
    pub role: ClientRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedSession {
    schema_version: Option<u32>,
    id: String,
    editor_token: String,
    viewer_token: String,
    token_secret: Option<String>,
    token_ttl_secs: Option<u64>,
    revoked_token_ids: Option<Vec<String>>,
    allow_legacy_tokens: Option<bool>,
    document_schema_version: Option<u32>,
    document: String,
    created_at: u64,
    last_active_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedSessionStore {
    version: Option<u32>,
    sessions: Vec<PersistedSession>,
}

impl Session {
    pub fn new(id: String, document: Document, token_ttl_secs: u64) -> Self {
        let now = now_unix_ts();
        let token_secret = format!("{}{}", Uuid::new_v4(), Uuid::new_v4());
        let mut session = Self::new_with_timestamps(
            id,
            document,
            token_secret,
            token_ttl_secs,
            String::new(),
            String::new(),
            false,
            HashSet::new(),
            now,
            now,
        );
        session.editor_token = session.issue_signed_token(ClientRole::Editor);
        session.viewer_token = session.issue_signed_token(ClientRole::Viewer);
        session
    }

    pub fn new_with_timestamps(
        id: String,
        document: Document,
        token_secret: String,
        token_ttl_secs: u64,
        editor_token: String,
        viewer_token: String,
        allow_legacy_tokens: bool,
        revoked_token_ids: HashSet<String>,
        created_at: u64,
        last_active_at: u64,
    ) -> Self {
        let (tx, _) = broadcast::channel::<ServerMessage>(10000);
        Self {
            id,
            document: Arc::new(RwLock::new(document)),
            clients: Arc::new(RwLock::new(HashMap::new())),
            client_roles: Arc::new(RwLock::new(HashMap::new())),
            client_id_maps: Arc::new(RwLock::new(HashMap::new())),
            operation_seq: Arc::new(AtomicU64::new(0)),
            broadcast_tx: Arc::new(tx),
            editor_token,
            viewer_token,
            token_secret,
            token_ttl_secs,
            revoked_token_ids: Arc::new(RwLock::new(revoked_token_ids)),
            allow_legacy_tokens,
            created_at: Arc::new(AtomicU64::new(created_at)),
            last_active_at: Arc::new(AtomicU64::new(last_active_at)),
        }
    }

    fn sign_payload(&self, payload_bytes: &[u8]) -> Option<Vec<u8>> {
        let mut mac = HmacSha256::new_from_slice(self.token_secret.as_bytes()).ok()?;
        mac.update(payload_bytes);
        Some(mac.finalize().into_bytes().to_vec())
    }

    fn issue_signed_token(&self, role: ClientRole) -> String {
        self.issue_token_for_role(role, None)
    }

    fn parse_signed_token(&self, token: &str) -> Option<SignedTokenPayload> {
        let mut parts = token.split('.');
        let payload_part = parts.next()?;
        let signature_part = parts.next()?;
        if parts.next().is_some() {
            return None;
        }

        let expected_signature = self.sign_payload(payload_part.as_bytes())?;
        let incoming_signature = URL_SAFE_NO_PAD.decode(signature_part.as_bytes()).ok()?;
        if expected_signature != incoming_signature {
            return None;
        }

        let payload_raw = URL_SAFE_NO_PAD.decode(payload_part.as_bytes()).ok()?;
        serde_json::from_slice::<SignedTokenPayload>(&payload_raw).ok()
    }

    fn is_token_revoked(&self, payload: &SignedTokenPayload) -> bool {
        let revoked = self.revoked_token_ids.read().unwrap();
        revoked.contains(&payload.jti)
    }

    fn is_legacy_token_revoked(&self, token: &str) -> bool {
        let revoked = self.revoked_token_ids.read().unwrap();
        revoked.contains(&format!("legacy:{token}"))
    }

    pub fn touch(&self) {
        self.last_active_at.store(now_unix_ts(), Ordering::SeqCst);
    }

    pub fn add_client(&self, client_id: String, name: String, color: String, role: ClientRole) {
        let mut clients = self.clients.write().unwrap();
        let id = client_id.clone();
        clients.insert(
            client_id,
            ClientInfo {
                id: id.clone(),
                name,
                color,
                role: role.clone(),
            },
        );
        let mut roles = self.client_roles.write().unwrap();
        roles.insert(id.clone(), role);
        let mut maps = self.client_id_maps.write().unwrap();
        maps.entry(id).or_default();
        self.touch();
    }

    pub fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.write().unwrap();
        clients.remove(client_id);
        let mut roles = self.client_roles.write().unwrap();
        roles.remove(client_id);
        let mut maps = self.client_id_maps.write().unwrap();
        maps.remove(client_id);
        self.touch();
    }

    pub fn get_clients(&self) -> Vec<ClientInfo> {
        let clients = self.clients.read().unwrap();
        clients.values().cloned().collect()
    }

    pub fn next_operation_seq(&self) -> u64 {
        self.operation_seq.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn map_client_local_id(&self, client_id: &str, local_id: u64, canonical_id: u64) {
        let mut maps = self.client_id_maps.write().unwrap();
        let entry = maps.entry(client_id.to_string()).or_default();
        entry.insert(local_id, canonical_id);
        self.touch();
    }

    pub fn resolve_client_id(&self, client_id: &str, incoming_id: u64) -> u64 {
        let maps = self.client_id_maps.read().unwrap();
        maps.get(client_id)
            .and_then(|m| m.get(&incoming_id).copied())
            .unwrap_or(incoming_id)
    }

    pub fn validate_token_for_role(&self, token: &str, role: ClientRole) -> bool {
        if let Some(payload) = self.parse_signed_token(token) {
            if payload.sid != self.id
                || payload.exp < now_unix_ts()
                || self.is_token_revoked(&payload)
            {
                return false;
            }
            return match role {
                ClientRole::Editor => payload.role == ClientRole::Editor,
                ClientRole::Viewer => {
                    payload.role == ClientRole::Viewer || payload.role == ClientRole::Editor
                }
            };
        }

        if self.allow_legacy_tokens {
            if self.is_legacy_token_revoked(token) {
                return false;
            }
            return match role {
                ClientRole::Editor => token == self.editor_token,
                ClientRole::Viewer => token == self.viewer_token || token == self.editor_token,
            };
        }

        false
    }

    pub fn validate_any_token(&self, token: &str) -> bool {
        self.validate_token_for_role(token, ClientRole::Viewer)
    }

    pub fn issue_token_for_role(&self, role: ClientRole, ttl_secs: Option<u64>) -> String {
        let payload = SignedTokenPayload {
            sid: self.id.clone(),
            role,
            exp: now_unix_ts().saturating_add(ttl_secs.unwrap_or(self.token_ttl_secs)),
            jti: Uuid::new_v4().to_string(),
            version: SESSION_STORE_SCHEMA_VERSION,
        };

        let payload_json = serde_json::to_vec(&payload).unwrap_or_default();
        let encoded_payload = URL_SAFE_NO_PAD.encode(payload_json);
        let signature = self
            .sign_payload(encoded_payload.as_bytes())
            .unwrap_or_default();
        let encoded_signature = URL_SAFE_NO_PAD.encode(signature);

        format!("{encoded_payload}.{encoded_signature}")
    }

    pub fn rotate_viewer_token(&mut self) -> String {
        let previous = self.viewer_token.clone();
        let _ = self.revoke_token(&previous);
        let next = self.issue_token_for_role(ClientRole::Viewer, None);
        self.viewer_token = next.clone();
        next
    }

    pub fn rotate_editor_token(&mut self) -> String {
        let previous = self.editor_token.clone();
        let _ = self.revoke_token(&previous);
        let next = self.issue_token_for_role(ClientRole::Editor, None);
        self.editor_token = next.clone();
        next
    }

    pub fn revoke_token(&self, token: &str) -> bool {
        if let Some(payload) = self.parse_signed_token(token) {
            if payload.sid != self.id {
                return false;
            }
            let mut revoked = self.revoked_token_ids.write().unwrap();
            return revoked.insert(payload.jti);
        }

        if self.allow_legacy_tokens && (token == self.editor_token || token == self.viewer_token) {
            let mut revoked = self.revoked_token_ids.write().unwrap();
            return revoked.insert(format!("legacy:{token}"));
        }

        false
    }

    pub fn can_client_edit(&self, client_id: &str) -> bool {
        let roles = self.client_roles.read().unwrap();
        matches!(roles.get(client_id), Some(ClientRole::Editor))
    }

    fn to_persisted(&self) -> PersistedSession {
        let document = {
            let doc = self.document.read().unwrap();
            doc.serialize()
        };
        PersistedSession {
            schema_version: Some(SESSION_STORE_SCHEMA_VERSION),
            id: self.id.clone(),
            editor_token: self.editor_token.clone(),
            viewer_token: self.viewer_token.clone(),
            token_secret: Some(self.token_secret.clone()),
            token_ttl_secs: Some(self.token_ttl_secs),
            revoked_token_ids: Some(
                self.revoked_token_ids
                    .read()
                    .unwrap()
                    .iter()
                    .cloned()
                    .collect(),
            ),
            allow_legacy_tokens: Some(self.allow_legacy_tokens),
            document_schema_version: Some(SESSION_DOCUMENT_SCHEMA_VERSION),
            document,
            created_at: self.created_at.load(Ordering::SeqCst),
            last_active_at: self.last_active_at.load(Ordering::SeqCst),
        }
    }

    fn from_persisted(snapshot: PersistedSession) -> Self {
        let mut document = Document::new();
        let _ = document.deserialize(&snapshot.document);

        let has_token_secret = snapshot.token_secret.is_some();
        let token_secret = snapshot
            .token_secret
            .unwrap_or_else(|| format!("{}{}", Uuid::new_v4(), Uuid::new_v4()));
        let token_ttl_secs = snapshot.token_ttl_secs.unwrap_or(DEFAULT_TOKEN_TTL_SECS);
        let allow_legacy_tokens = snapshot.allow_legacy_tokens.unwrap_or(!has_token_secret);

        Session::new_with_timestamps(
            snapshot.id,
            document,
            token_secret,
            token_ttl_secs,
            snapshot.editor_token,
            snapshot.viewer_token,
            allow_legacy_tokens,
            snapshot
                .revoked_token_ids
                .unwrap_or_default()
                .into_iter()
                .collect(),
            snapshot.created_at,
            snapshot.last_active_at,
        )
    }
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    store_path: PathBuf,
    ttl_secs: u64,
    token_ttl_secs: u64,
}

impl SessionManager {
    pub fn new_with_persistence(store_path: PathBuf, ttl_secs: u64, token_ttl_secs: u64) -> Self {
        let mut manager = Self {
            sessions: HashMap::new(),
            store_path,
            ttl_secs,
            token_ttl_secs,
        };
        manager.load_from_disk();
        manager
    }

    pub fn create_new_session(&mut self, session_id: String, document: Document) -> Session {
        let session = Session::new(session_id, document, self.token_ttl_secs);
        self.create_session(session.clone());
        session
    }

    pub fn create_session(&mut self, session: Session) {
        self.sessions.insert(session.id.clone(), session);
        let _ = self.persist_all();
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.get(session_id).cloned()
    }

    pub fn mark_session_active(&self, session_id: &str) {
        if let Some(session) = self.sessions.get(session_id) {
            session.touch();
        }
    }

    pub fn persist_all(&self) -> Result<(), String> {
        let parent_dir = self
            .store_path
            .parent()
            .ok_or_else(|| "Invalid session store path".to_string())?;

        std::fs::create_dir_all(parent_dir)
            .map_err(|e| format!("Failed to create session store directory: {e}"))?;

        let snapshots: Vec<PersistedSession> =
            self.sessions.values().map(Session::to_persisted).collect();

        let payload = PersistedSessionStore {
            version: Some(SESSION_STORE_SCHEMA_VERSION),
            sessions: snapshots,
        };
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|e| format!("Failed to serialize session store: {e}"))?;

        let temp_path = self.store_path.with_extension("tmp");
        std::fs::write(&temp_path, json)
            .map_err(|e| format!("Failed to write temp session store: {e}"))?;
        std::fs::rename(&temp_path, &self.store_path)
            .map_err(|e| format!("Failed to atomically replace session store: {e}"))
    }

    pub fn cleanup_expired_sessions(&mut self) -> usize {
        let now = now_unix_ts();
        let before = self.sessions.len();
        self.sessions.retain(|_, session| {
            let has_clients = !session.clients.read().unwrap().is_empty();
            let last_active_at = session.last_active_at.load(Ordering::SeqCst);
            has_clients || now.saturating_sub(last_active_at) < self.ttl_secs
        });
        before.saturating_sub(self.sessions.len())
    }

    fn load_from_disk(&mut self) {
        let raw = match std::fs::read_to_string(&self.store_path) {
            Ok(data) => data,
            Err(_) => return,
        };

        let mut parsed = match serde_json::from_str::<PersistedSessionStore>(&raw) {
            Ok(value) => value,
            Err(_) => return,
        };

        match parsed.version.unwrap_or(1) {
            1 => {
                parsed.version = Some(SESSION_STORE_SCHEMA_VERSION);
            }
            SESSION_STORE_SCHEMA_VERSION => {}
            _ => return,
        }

        for snapshot in parsed.sessions {
            let session = Session::from_persisted(snapshot);
            self.sessions.insert(session.id.clone(), session);
        }
    }

    pub fn revoke_token(&mut self, session_id: &str, token_to_revoke: &str) -> bool {
        let Some(session) = self.sessions.get(session_id) else {
            return false;
        };
        let revoked = session.revoke_token(token_to_revoke);
        if revoked {
            let _ = self.persist_all();
        }
        revoked
    }

    pub fn rotate_viewer_token(&mut self, session_id: &str) -> Option<String> {
        let session = self.sessions.get_mut(session_id)?;
        let next = session.rotate_viewer_token();
        let _ = self.persist_all();
        Some(next)
    }

    pub fn rotate_editor_token(&mut self, session_id: &str) -> Option<String> {
        let session = self.sessions.get_mut(session_id)?;
        let next = session.rotate_editor_token();
        let _ = self.persist_all();
        Some(next)
    }

    pub fn issue_invite_token(
        &mut self,
        session_id: &str,
        role: ClientRole,
        ttl_secs: Option<u64>,
    ) -> Option<String> {
        let session = self.sessions.get_mut(session_id)?;
        Some(session.issue_token_for_role(role, ttl_secs))
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    #[allow(dead_code)]
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
        let _ = self.persist_all();
    }
}
