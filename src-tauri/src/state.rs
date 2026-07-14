use std::sync::{Arc, Mutex};
use fastembed::TextEmbedding;
use rusqlite::Connection;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmbeddingStatus {
    Deferred,
    Loading,
    Ready,
    Unavailable,
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub embedding_status: Mutex<EmbeddingStatus>,
    pub embedder: Arc<Mutex<Option<TextEmbedding>>>,
}

impl AppState {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self {
            db,
            embedding_status: Mutex::new(EmbeddingStatus::Deferred),
            embedder: Arc::new(Mutex::new(None)),
        }
    }
}
