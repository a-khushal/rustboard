use rustboard_editor::Document;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use crate::websocket::ServerMessage;

fn now_unix_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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
    id: String,
    editor_token: String,
    viewer_token: String,
    document: String,
    created_at: u64,
    last_active_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedSessionStore {
    sessions: Vec<PersistedSession>,
}

impl Session {
    pub fn new(id: String, document: Document, editor_token: String, viewer_token: String) -> Self {
        let now = now_unix_ts();
        Self::new_with_timestamps(id, document, editor_token, viewer_token, now, now)
    }

    pub fn new_with_timestamps(
        id: String,
        document: Document,
        editor_token: String,
        viewer_token: String,
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
            created_at: Arc::new(AtomicU64::new(created_at)),
            last_active_at: Arc::new(AtomicU64::new(last_active_at)),
        }
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
        match role {
            ClientRole::Editor => token == self.editor_token,
            ClientRole::Viewer => token == self.viewer_token || token == self.editor_token,
        }
    }

    pub fn validate_any_token(&self, token: &str) -> bool {
        token == self.editor_token || token == self.viewer_token
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
            id: self.id.clone(),
            editor_token: self.editor_token.clone(),
            viewer_token: self.viewer_token.clone(),
            document,
            created_at: self.created_at.load(Ordering::SeqCst),
            last_active_at: self.last_active_at.load(Ordering::SeqCst),
        }
    }

    fn from_persisted(snapshot: PersistedSession) -> Self {
        let mut document = Document::new();
        let _ = document.deserialize(&snapshot.document);
        Session::new_with_timestamps(
            snapshot.id,
            document,
            snapshot.editor_token,
            snapshot.viewer_token,
            snapshot.created_at,
            snapshot.last_active_at,
        )
    }
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    store_path: PathBuf,
    ttl_secs: u64,
}

impl SessionManager {
    pub fn new_with_persistence(store_path: PathBuf, ttl_secs: u64) -> Self {
        let mut manager = Self {
            sessions: HashMap::new(),
            store_path,
            ttl_secs,
        };
        manager.load_from_disk();
        manager
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

        let parsed = match serde_json::from_str::<PersistedSessionStore>(&raw) {
            Ok(value) => value,
            Err(_) => return,
        };

        for snapshot in parsed.sessions {
            let session = Session::from_persisted(snapshot);
            self.sessions.insert(session.id.clone(), session);
        }
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
