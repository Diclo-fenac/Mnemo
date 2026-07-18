use fastembed::{TextEmbedding, TextRerank};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

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
    pub embedding_status: Arc<Mutex<EmbeddingStatus>>,
    pub model_start_requested: Arc<AtomicBool>,
    pub embedder: Arc<Mutex<Option<TextEmbedding>>>,
    pub reranker: Arc<Mutex<Option<TextRerank>>>,
    pub browser_context: Arc<Mutex<Option<BrowserContext>>>,
    pub capture_enabled: Arc<AtomicBool>,
    pub browser_context_enabled: Arc<AtomicBool>,
    pub model_cache_dir: PathBuf,
}

impl AppState {
    pub fn new(
        db: Arc<Mutex<Connection>>,
        model_cache_dir: PathBuf,
        capture_enabled: bool,
        browser_context_enabled: bool,
    ) -> Self {
        Self {
            db,
            embedding_status: Arc::new(Mutex::new(EmbeddingStatus::Deferred)),
            model_start_requested: Arc::new(AtomicBool::new(false)),
            embedder: Arc::new(Mutex::new(None)),
            reranker: Arc::new(Mutex::new(None)),
            browser_context: Arc::new(Mutex::new(None)),
            capture_enabled: Arc::new(AtomicBool::new(capture_enabled)),
            browser_context_enabled: Arc::new(AtomicBool::new(browser_context_enabled)),
            model_cache_dir,
        }
    }

    pub fn reset_model_start_request(&self) {
        self.model_start_requested.store(false, Ordering::Release);
    }
}
