use std::sync::Arc;
use std::thread;

use fastembed::TextEmbedding;
use rusqlite::{params, Connection};
use tauri::State;

use crate::services::model_registry;
use crate::state::AppState;

#[tauri::command]
pub fn get_supported_embedding_models() -> Vec<model_registry::ModelInfo> {
    model_registry::SUPPORTED_MODELS.to_vec()
}

#[tauri::command]
pub fn get_active_embedding_model(state: State<'_, AppState>) -> Result<String, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "Database unavailable".to_string())?;
    conn.query_row(
        "SELECT model FROM embedding_registry WHERE slot = 'active'",
        [],
        |row| row.get(0),
    )
    .map_err(|error| format!("Active model unavailable: {error}"))
}

#[tauri::command]
pub fn switch_embedding_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<bool, String> {
    let spec =
        model_registry::info(&model_id).ok_or_else(|| "Unsupported embedding model".to_string())?;
    let db = Arc::clone(&state.db);
    let embedder_state = Arc::clone(&state.embedder);
    let model_cache_dir = state.model_cache_dir.clone();

    let (next_version, table_name) = {
        let conn = db.lock().map_err(|_| "Database unavailable".to_string())?;
        let (active_model, active_version): (String, String) = conn
            .query_row(
                "SELECT model, version FROM embedding_registry WHERE slot = 'active'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        if active_model == model_id {
            return Ok(false);
        }
        let version = active_version.parse::<u64>().unwrap_or(1) + 1;
        let table = format!("clips_embeddings_migration_{version}");
        conn.execute("DELETE FROM embedding_registry WHERE slot = 'pending'", [])
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO embedding_registry
             (slot, model, version, dimensions, table_name, state, updated_at)
             VALUES ('pending', ?1, ?2, ?3, ?4, 'migrating', ?5)",
            params![
                model_id,
                version.to_string(),
                spec.dimensions as i64,
                table,
                chrono::Utc::now().timestamp_millis()
            ],
        )
        .map_err(|e| e.to_string())?;
        (version.to_string(), table)
    };

    thread::spawn(move || {
        if let Err(error) = migrate_model(
            db.clone(),
            embedder_state,
            model_id,
            &next_version,
            &table_name,
            model_cache_dir,
        ) {
            log::error!("[embedder] Model migration failed: {error}");
            if let Ok(conn) = db.lock() {
                let _ = conn.execute(
                    "UPDATE embedding_registry SET state = 'failed', updated_at = ?1
                     WHERE slot = 'pending'",
                    [chrono::Utc::now().timestamp_millis()],
                );
            }
        }
    });

    Ok(true)
}

fn migrate_model(
    db: Arc<std::sync::Mutex<Connection>>,
    embedder_state: Arc<std::sync::Mutex<Option<TextEmbedding>>>,
    model_id: String,
    version: &str,
    new_table: &str,
    model_cache_dir: std::path::PathBuf,
) -> Result<(), String> {
    let spec = model_registry::info(&model_id).ok_or_else(|| "Unsupported model".to_string())?;
    let model = crate::services::model_loader::load_text(&model_id, &model_cache_dir)
        .map_err(|e| e.to_string())?;

    {
        let conn = db.lock().map_err(|_| "Database unavailable".to_string())?;
        conn.execute_batch(&format!(
            "DROP TABLE IF EXISTS {new_table};
             CREATE VIRTUAL TABLE {new_table} USING vec0(
               clip_id TEXT PRIMARY KEY, embedding float[{}] distance_metric=cosine
             );",
            spec.dimensions
        ))
        .map_err(|e| e.to_string())?;
    }

    loop {
        let rows: Vec<(String, String)> = {
            let conn = db.lock().map_err(|_| "Database unavailable".to_string())?;
            let sql = format!(
                "SELECT c.id, c.content FROM clips c
                 WHERE c.is_duplicate = 0
                   AND NOT EXISTS (SELECT 1 FROM {new_table} v WHERE v.clip_id = c.id)
                 ORDER BY c.copied_at ASC LIMIT 32"
            );
            let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            rows
        };
        if rows.is_empty() {
            break;
        }

        let contents: Vec<String> = rows
            .iter()
            .map(|(_, content)| model_registry::prepare_text(&model_id, content, false))
            .collect();
        let vectors = model.embed(contents, None).map_err(|e| e.to_string())?;
        let conn = db.lock().map_err(|_| "Database unavailable".to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        let insert_sql = format!("INSERT INTO {new_table} (clip_id, embedding) VALUES (?1, ?2)");
        for ((clip_id, _), vector) in rows.iter().zip(vectors.iter()) {
            let bytes: Vec<u8> = vector
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect();
            tx.execute(&insert_sql, params![clip_id, bytes])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
    }

    let conn = db.lock().map_err(|_| "Database unavailable".to_string())?;
    let active_table: String = conn
        .query_row(
            "SELECT table_name FROM embedding_registry WHERE slot = 'active'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let old_table = format!("{active_table}_old_{version}");
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute_batch(&format!(
        "ALTER TABLE {active_table} RENAME TO {old_table};
         ALTER TABLE {new_table} RENAME TO {active_table};"
    ))
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE clips SET embedding_model = ?1, embedding_version = ?2,
         embedding_status = 'embedded', embedding_id = id WHERE is_duplicate = 0",
        params![model_id, version],
    )
    .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM embedding_registry WHERE slot = 'retired'", [])
        .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE embedding_registry SET slot = 'retired', state = 'retired' WHERE slot = 'active'",
        [],
    )
    .map_err(|e| e.to_string())?;
    tx.execute_batch(&format!("DROP TABLE IF EXISTS {old_table};"))
        .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE embedding_registry SET slot = 'active', state = 'ready', table_name = ?1,
         updated_at = ?2 WHERE slot = 'pending'",
        params![active_table, chrono::Utc::now().timestamp_millis()],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    let mut current = embedder_state
        .lock()
        .map_err(|_| "Embedder state unavailable".to_string())?;
    *current = Some(model);
    Ok(())
}
