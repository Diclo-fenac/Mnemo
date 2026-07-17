use arboard::Clipboard;
use rusqlite::params;
use tauri::State;

use crate::models::clip::Clip;
use crate::state::AppState;

#[tauri::command]
pub fn list_clips(
    state: State<'_, AppState>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<Clip>, String> {
    let limit = page_size.unwrap_or(50).clamp(1, 200) as i64;
    let offset = ((page.unwrap_or(1).max(1) - 1) as i64) * limit;

    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, content, content_type, image_path, source_url, page_title,
                    app_name, window_title, language, session_id, is_pinned,
                    copied_at, ai_context, created_at
             FROM clips
             ORDER BY copied_at DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| format!("Query prepare failed: {e}"))?;

    let clips = stmt
        .query_map(rusqlite::params![limit, offset], |row| {
            Ok(Clip {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                image_path: row.get(3)?,
                source_url: row.get(4)?,
                page_title: row.get(5)?,
                app_name: row.get(6)?,
                window_title: row.get(7)?,
                language: row.get(8)?,
                session_id: row.get(9)?,
                is_pinned: row.get::<_, i32>(10)? == 1,
                copied_at: row.get(11)?,
                ai_context: row.get(12)?,
                created_at: row.get(13)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Query row failed: {e}"))?;

    Ok(clips)
}

#[tauri::command]
pub fn get_clip(state: State<'_, AppState>, id: String) -> Result<Clip, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;

    conn.query_row(
        "SELECT id, content, content_type, image_path, source_url, page_title,
                app_name, window_title, language, session_id, is_pinned,
                copied_at, ai_context, created_at
         FROM clips WHERE id = ?1",
        [&id],
        |row| {
            Ok(Clip {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                image_path: row.get(3)?,
                source_url: row.get(4)?,
                page_title: row.get(5)?,
                app_name: row.get(6)?,
                window_title: row.get(7)?,
                language: row.get(8)?,
                session_id: row.get(9)?,
                is_pinned: row.get::<_, i32>(10)? == 1,
                copied_at: row.get(11)?,
                ai_context: row.get(12)?,
                created_at: row.get(13)?,
            })
        },
    )
    .map_err(|e| format!("Clip not found: {e}"))
}

#[tauri::command]
pub fn delete_clip(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    let table = active_embedding_table(&conn)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Delete transaction failed: {e}"))?;
    tx.execute(&format!("DELETE FROM {table} WHERE clip_id = ?1"), [&id])
        .map_err(|e| format!("Embedding delete failed: {e}"))?;
    let deleted = tx
        .execute("DELETE FROM clips WHERE id = ?1", [&id])
        .map_err(|e| format!("Delete failed: {e}"))?;
    if deleted > 0 {
        let clips: i64 = tx
            .query_row("SELECT COUNT(*) FROM clips", [], |row| row.get(0))
            .map_err(|e| format!("State count failed: {e}"))?;
        let sessions: i64 = tx
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
            .map_err(|e| format!("State count failed: {e}"))?;
        let edges: i64 = tx
            .query_row("SELECT COUNT(*) FROM memory_edges", [], |row| row.get(0))
            .map_err(|e| format!("State count failed: {e}"))?;
        let facts: i64 = tx
            .query_row("SELECT COUNT(*) FROM memory_facts", [], |row| row.get(0))
            .map_err(|e| format!("State count failed: {e}"))?;
        let stage = if clips >= 200 && sessions >= 5 {
            "archivor"
        } else if clips >= 50 {
            "bindor"
        } else {
            "clippy"
        };
        tx.execute(
            "UPDATE memory_state SET current_stage = ?1, total_clips = ?2, total_sessions = ?3,
             total_edges = ?4, total_facts = ?5, last_analysis = ?6 WHERE id = 1",
            params![
                stage,
                clips,
                sessions,
                edges,
                facts,
                chrono::Utc::now().timestamp_millis()
            ],
        )
        .map_err(|e| format!("State update failed: {e}"))?;
    }
    tx.commit()
        .map_err(|e| format!("Delete commit failed: {e}"))?;
    Ok(deleted > 0)
}

#[tauri::command]
pub fn toggle_pin(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;

    let current: bool = conn
        .query_row("SELECT is_pinned FROM clips WHERE id = ?1", [&id], |row| {
            row.get::<_, i32>(0).map(|v| v == 1)
        })
        .map_err(|e| format!("Clip not found: {e}"))?;

    let new_state = !current;
    conn.execute(
        "UPDATE clips SET is_pinned = ?1 WHERE id = ?2",
        rusqlite::params![new_state as i32, id],
    )
    .map_err(|e| format!("Pin update failed: {e}"))?;

    Ok(new_state)
}

#[tauri::command]
pub fn copy_clip(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;

    let content: String = conn
        .query_row("SELECT content FROM clips WHERE id = ?1", [&id], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Clip not found: {e}"))?;
    drop(conn);

    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard unavailable: {e}"))?;
    clipboard
        .set_text(&content)
        .map_err(|e| format!("Failed to copy: {e}"))?;

    Ok(true)
}

#[tauri::command]
pub fn get_session_clips(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Clip>, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, content, content_type, image_path, source_url, page_title,
                    app_name, window_title, language, session_id, is_pinned,
                    copied_at, ai_context, created_at
             FROM clips
             WHERE session_id = ?1
             ORDER BY copied_at ASC",
        )
        .map_err(|e| format!("Query prepare failed: {e}"))?;

    let clips = stmt
        .query_map(rusqlite::params![session_id], |row| {
            Ok(Clip {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                image_path: row.get(3)?,
                source_url: row.get(4)?,
                page_title: row.get(5)?,
                app_name: row.get(6)?,
                window_title: row.get(7)?,
                language: row.get(8)?,
                session_id: row.get(9)?,
                is_pinned: row.get::<_, i32>(10)? == 1,
                copied_at: row.get(11)?,
                ai_context: row.get(12)?,
                created_at: row.get(13)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Query row failed: {e}"))?;

    Ok(clips)
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
