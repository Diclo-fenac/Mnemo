use crate::state::BrowserContext;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use chrono::Utc;
use serde::Deserialize;
use std::sync::{atomic::AtomicBool, Arc, Mutex};

#[derive(Deserialize)]
pub struct ContextPayload {
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub selected_text: Option<String>,
    #[serde(default)]
    pub favicon_url: Option<String>,
}

#[derive(Clone)]
struct ServerState {
    context: Arc<Mutex<Option<BrowserContext>>>,
    enabled: Arc<AtomicBool>,
}

pub fn start_server(context_state: Arc<Mutex<Option<BrowserContext>>>, enabled: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                log::error!("[http] Runtime initialization failed: {error}");
                return;
            }
        };
        rt.block_on(async {
            let app = Router::new()
                .route("/context", post(handle_context))
                .with_state(ServerState {
                    context: context_state,
                    enabled,
                });

            let listener = match tokio::net::TcpListener::bind("127.0.0.1:17531").await {
                Ok(listener) => listener,
                Err(error) => {
                    log::error!("[http] Listener unavailable: {error}");
                    return;
                }
            };
            log::info!("[http] Listening on 127.0.0.1:17531");
            if let Err(error) = axum::serve(listener, app).await {
                log::error!("[http] Server failed: {error}");
            }
        });
    });
}

async fn handle_context(
    State(state): State<ServerState>,
    Json(payload): Json<ContextPayload>,
) -> Result<&'static str, StatusCode> {
    if !crate::services::capture_state::is_enabled(&state.enabled) {
        return Err(StatusCode::NO_CONTENT);
    }
    if payload.url.trim().is_empty()
        || payload.url.len() > 8_192
        || payload.title.len() > 2_048
        || payload
            .selected_text
            .as_ref()
            .is_some_and(|text| text.len() > 100_000)
        || payload
            .favicon_url
            .as_ref()
            .is_some_and(|url| url.len() > 8_192)
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut lock = state
        .context
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    *lock = Some(BrowserContext {
        url: payload.url,
        title: payload.title,
        timestamp: Utc::now().timestamp_millis(),
    });

    Ok("ok")
}
