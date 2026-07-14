use std::sync::{Arc, Mutex};
use axum::{
    routing::post,
    extract::State,
    Json, Router,
};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use chrono::Utc;
use crate::state::BrowserContext;

#[derive(Deserialize)]
pub struct ContextPayload {
    pub url: String,
    pub title: String,
}

type ServerState = Arc<Mutex<Option<BrowserContext>>>;

pub fn start_server(context_state: ServerState) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);

            let app = Router::new()
                .route("/context", post(handle_context))
                .layer(cors)
                .with_state(context_state);

            let listener = tokio::net::TcpListener::bind("127.0.0.1:17531").await.unwrap();
            log::info!("[http] Listening on 127.0.0.1:17531");
            axum::serve(listener, app).await.unwrap();
        });
    });
}

async fn handle_context(
    State(state): State<ServerState>,
    Json(payload): Json<ContextPayload>,
) -> &'static str {
    if payload.url.is_empty() {
        return "empty";
    }

    let mut lock = state.lock().unwrap();
    *lock = Some(BrowserContext {
        url: payload.url,
        title: payload.title,
        timestamp: Utc::now().timestamp(),
    });

    "ok"
}
