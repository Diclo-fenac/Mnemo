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
        embedding_status: embedding_status.to_string(),
        stage,
    })
}
