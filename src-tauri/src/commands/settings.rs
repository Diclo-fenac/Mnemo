use crate::{
    services::capture_state::{self, CapturePreferences},
    state::AppState,
};
use serde::Deserialize;
use tauri::{Emitter, State};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturePreferencesInput {
    pub capture_enabled: bool,
    pub browser_context_enabled: bool,
    pub auto_delete_days: Option<i64>,
    pub appearance: String,
    pub onboarding_completed: bool,
}

#[tauri::command]
pub fn get_capture_preferences(state: State<'_, AppState>) -> Result<CapturePreferences, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    capture_state::load(&conn).map_err(|error| format!("Unable to load preferences: {error}"))
}

#[tauri::command]
pub fn update_capture_preferences(
    state: State<'_, AppState>,
    preferences: CapturePreferencesInput,
) -> Result<CapturePreferences, String> {
    if !matches!(preferences.appearance.as_str(), "dark" | "light" | "system") {
        return Err("Appearance must be dark, light, or system".to_string());
    }
    if let Some(days) = preferences.auto_delete_days {
        if !(1..=3650).contains(&days) {
            return Err("Auto-delete days must be between 1 and 3650".to_string());
        }
    }
    let next = CapturePreferences {
        capture_enabled: preferences.capture_enabled,
        browser_context_enabled: preferences.browser_context_enabled,
        auto_delete_days: preferences.auto_delete_days,
        appearance: preferences.appearance,
        onboarding_completed: preferences.onboarding_completed,
    };
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    capture_state::persist(&conn, &next)
        .map_err(|error| format!("Unable to save preferences: {error}"))?;
    capture_state::set_enabled(&state.capture_enabled, next.capture_enabled);
    capture_state::set_enabled(&state.browser_context_enabled, next.browser_context_enabled);
    if !next.browser_context_enabled {
        if let Ok(mut context) = state.browser_context.lock() {
            *context = None;
        }
    }
    Ok(next)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteOnboardingInput {
    pub capture_enabled: bool,
}

#[tauri::command]
pub fn complete_onboarding(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    input: CompleteOnboardingInput,
) -> Result<CapturePreferences, String> {
    let next = {
        let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
        let mut preferences = capture_state::load(&conn)
            .map_err(|error| format!("Unable to load preferences: {error}"))?;
        preferences.onboarding_completed = true;
        preferences.capture_enabled = input.capture_enabled;
        capture_state::persist(&conn, &preferences)
            .map_err(|error| format!("Unable to save onboarding: {error}"))?;
        preferences
    };

    capture_state::set_enabled(&state.capture_enabled, next.capture_enabled);
    if crate::services::embedder::start_embedder(
        state.db.clone(),
        state.embedder.clone(),
        state.embedding_status.clone(),
        state.model_cache_dir.clone(),
        state.model_start_requested.clone(),
    ) {
        let _ = app.emit("embedding-state-changed", "loading");
    }
    Ok(next)
}

#[tauri::command]
pub fn retry_embedding_model(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    state.reset_model_start_request();
    if crate::services::embedder::start_embedder(
        state.db.clone(),
        state.embedder.clone(),
        state.embedding_status.clone(),
        state.model_cache_dir.clone(),
        state.model_start_requested.clone(),
    ) {
        let _ = app.emit("embedding-state-changed", "loading");
        return Ok(true);
    }
    Ok(false)
}

#[tauri::command]
pub fn clear_database(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    let table = active_embedding_table(&conn)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Database transaction failed: {e}"))?;
    tx.execute(&format!("DELETE FROM {table}"), [])
        .map_err(|e| format!("Failed to clear embeddings: {e}"))?;
    tx.execute("DELETE FROM clips", [])
        .map_err(|e| format!("Failed to delete clips: {e}"))?;
    tx.execute("DELETE FROM sessions", [])
        .map_err(|e| format!("Failed to delete sessions: {e}"))?;
    tx.execute("DELETE FROM memory_edges", [])
        .map_err(|e| format!("Failed to delete memory edges: {e}"))?;
    tx.execute("DELETE FROM memory_facts", [])
        .map_err(|e| format!("Failed to delete memory facts: {e}"))?;
    tx.execute("DELETE FROM search_feedback", [])
        .map_err(|e| format!("Failed to delete search feedback: {e}"))?;
    tx.execute(
        "UPDATE memory_state SET current_stage = 'clippy', total_clips = 0,
         total_sessions = 0, total_edges = 0, total_facts = 0, last_analysis = NULL WHERE id = 1",
        [],
    )
    .map_err(|e| format!("Failed to reset memory state: {e}"))?;
    tx.commit()
        .map_err(|e| format!("Database reset commit failed: {e}"))?;

    Ok(())
}

fn active_embedding_table(conn: &rusqlite::Connection) -> Result<String, String> {
    let table = conn
        .query_row(
            "SELECT table_name FROM embedding_registry WHERE slot = 'active'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "clips_embeddings".to_string());
    if table.is_empty() || !table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Invalid active embedding table".to_string());
    }
    Ok(table)
}
