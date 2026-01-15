use rustboard_editor::Document;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use crate::websocket::ServerMessage;

#[derive(Clone)]
pub struct Session {
    pub id: String,
    pub document: Arc<RwLock<Document>>,
    pub clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
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
        let (tx, _) = broadcast::channel::<ServerMessage>(100);
        Self {
            id,
            document: Arc::new(RwLock::new(document)),
            clients: Arc::new(RwLock::new(HashMap::new())),
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
                id,
                name,
                color,
            },
        );
    }

    pub fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.write().unwrap();
        clients.remove(client_id);
    }

    pub fn get_clients(&self) -> Vec<ClientInfo> {
        let clients = self.clients.read().unwrap();
        clients.values().cloned().collect()
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

