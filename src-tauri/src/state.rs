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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct BrowserContext {
    pub url: String,
    pub title: String,
    pub timestamp: i64,
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub embedding_status: Mutex<EmbeddingStatus>,
    pub embedder: Arc<Mutex<Option<TextEmbedding>>>,
    pub browser_context: Arc<Mutex<Option<BrowserContext>>>,
}

impl AppState {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self {
            db,
            embedding_status: Mutex::new(EmbeddingStatus::Deferred),
            embedder: Arc::new(Mutex::new(None)),
            browser_context: Arc::new(Mutex::new(None)),
        }
    }
}
