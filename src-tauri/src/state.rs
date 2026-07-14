use std::sync::{Arc, Mutex};

use rusqlite::Connection;

#[derive(Debug, Clone, Copy)]
pub enum EmbeddingStatus {
    Deferred,
    Loading,
    Ready,
    Unavailable,
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub embedding_status: Mutex<EmbeddingStatus>,
}

impl AppState {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self {
            db,
            // FastEmbed is added in Milestone 3. The UI can already represent this state.
            embedding_status: Mutex::new(EmbeddingStatus::Deferred),
        }
    }
}
