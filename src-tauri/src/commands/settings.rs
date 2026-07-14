use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub fn clear_database(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;

    conn.execute("DELETE FROM clips", [])
        .map_err(|e| format!("Failed to delete clips: {}", e))?;
        
    conn.execute("DELETE FROM sessions", [])
        .map_err(|e| format!("Failed to delete sessions: {}", e))?;
        
    conn.execute("DELETE FROM filter_rules", [])
        .map_err(|e| format!("Failed to delete filter_rules: {}", e))?;

    // We can also drop FTS index and reset memory_state.
    conn.execute("UPDATE memory_state SET total_clips = 0, total_sessions = 0 WHERE id = 1", [])
        .map_err(|e| format!("Failed to reset memory state: {}", e))?;

    Ok(())
}
