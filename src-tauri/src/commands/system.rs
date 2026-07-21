use tauri::State;

use crate::{
    models::BootstrapState,
    state::{AppState, EmbeddingStatus},
};

#[tauri::command]
pub fn healthcheck(state: State<'_, AppState>) -> Result<bool, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "Database state is unavailable".to_string())?;
    connection
        .query_row("SELECT 1", [], |row| row.get::<_, i32>(0))
        .map(|value| value == 1)
        .map_err(|error| format!("Database healthcheck failed: {error}"))
}

#[tauri::command]
pub fn test_context_bridge(state: State<'_, AppState>) -> Result<bool, String> {
    if !crate::services::capture_state::is_enabled(&state.browser_context_enabled) {
        return Err("Enable Browser Context first".to_string());
    }
    ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_millis(500))
        .timeout_read(std::time::Duration::from_millis(500))
        .build()
        .get("http://127.0.0.1:17531/health")
        .call()
        .map(|response| response.status() == 200)
        .map_err(|error| format!("Context bridge is unavailable: {error}"))
}

#[tauri::command]
pub fn get_bootstrap_state(state: State<'_, AppState>) -> Result<BootstrapState, String> {
    let connection = state
        .db
        .lock()
        .map_err(|_| "Database state is unavailable".to_string())?;
    let stage = connection
        .query_row(
            "SELECT current_stage FROM memory_state WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "clippy".to_string());
    let onboarding_completed = connection
        .query_row(
            "SELECT onboarding_completed FROM settings WHERE id = 1",
            [],
            |row| row.get::<_, i32>(0),
        )
        .map(|value| value != 0)
        .unwrap_or(false);
    drop(connection);

    let embedding_status = match *state
        .embedding_status
        .lock()
        .map_err(|_| "Embedder state is unavailable".to_string())?
    {
        EmbeddingStatus::Deferred => "deferred",
        EmbeddingStatus::Loading => "loading",
        EmbeddingStatus::Ready => "ready",
        EmbeddingStatus::Unavailable => "unavailable",
    };

    Ok(BootstrapState {
        database_ready: true,
        onboarding_completed,
        embedding_status: embedding_status.to_string(),
        stage,
    })
}
