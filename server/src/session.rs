use rustboard_editor::Document;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use crate::websocket::ServerMessage;

#[derive(Clone)]
pub struct Session {
    pub id: String,
    pub document: Arc<RwLock<Document>>,
    pub clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
    pub client_id_maps: Arc<RwLock<HashMap<String, HashMap<u64, u64>>>>,
    pub operation_seq: Arc<AtomicU64>,
    pub broadcast_tx: Arc<broadcast::Sender<ServerMessage>>,
    pub _created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl Session {
    pub fn new(id: String, document: Document) -> Self {
        let (tx, _) = broadcast::channel::<ServerMessage>(10000);
        Self {
            id,
            document: Arc::new(RwLock::new(document)),
            clients: Arc::new(RwLock::new(HashMap::new())),
            client_id_maps: Arc::new(RwLock::new(HashMap::new())),
            operation_seq: Arc::new(AtomicU64::new(0)),
            broadcast_tx: Arc::new(tx),
            _created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn add_client(&self, client_id: String, name: String, color: String) {
        let mut clients = self.clients.write().unwrap();
        let id = client_id.clone();
        clients.insert(
            client_id,
            ClientInfo {
                id: id.clone(),
                name,
                color,
            },
        );
        let mut maps = self.client_id_maps.write().unwrap();
        maps.entry(id).or_default();
    }

    pub fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.write().unwrap();
        clients.remove(client_id);
        let mut maps = self.client_id_maps.write().unwrap();
        maps.remove(client_id);
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
    }

    pub fn resolve_client_id(&self, client_id: &str, incoming_id: u64) -> u64 {
        let maps = self.client_id_maps.read().unwrap();
        maps.get(client_id)
            .and_then(|m| m.get(&incoming_id).copied())
            .unwrap_or(incoming_id)
    }
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self, session: Session) {
        self.sessions.insert(session.id.clone(), session);
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.get(session_id).cloned()
    }

    pub fn session_exists(&self, session_id: &str) -> bool {
        self.sessions.contains_key(session_id)
    }

    #[allow(dead_code)]
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }
}
