use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityMetrics {
    pub total_clips: i64,
    pub embedded_clips: i64,
    pub pending_clips: i64,
    pub failed_clips: i64,
    pub skipped_clips: i64,
    pub embedding_coverage: f64,
    pub duplicate_count: i64,
    pub edge_count: i64,
    pub empty_search_rate: f64,
    pub ctr_by_position: Vec<PositionCtr>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionCtr {
    pub band: String,
    pub impressions: i64,
    pub clicks: i64,
    pub ctr: f64,
}

#[tauri::command]
pub fn get_quality_metrics(state: State<'_, AppState>) -> Result<QualityMetrics, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    let count = |sql: &str| -> Result<i64, String> {
        conn.query_row(sql, [], |row| row.get(0))
            .map_err(|e| e.to_string())
    };
    let total = count("SELECT COUNT(*) FROM clips")?;
    let embedded = count("SELECT COUNT(*) FROM clips WHERE embedding_status = 'embedded'")?;
    let pending = count("SELECT COUNT(*) FROM clips WHERE embedding_status = 'pending'")?;
    let failed = count("SELECT COUNT(*) FROM clips WHERE embedding_status = 'failed'")?;
    let skipped = count("SELECT COUNT(*) FROM clips WHERE embedding_status = 'skipped'")?;
    let duplicate_count = count("SELECT COUNT(*) FROM clips WHERE is_duplicate = 1")?;
    let edge_count = count("SELECT COUNT(*) FROM memory_edges")?;
    let searches =
        count("SELECT COUNT(DISTINCT query) FROM search_feedback WHERE action = 'no_results'")?;
    let all_queries = count("SELECT COUNT(DISTINCT query) FROM search_feedback")?;
    let mut bands = Vec::new();
    for (name, low, high) in [
        ("1-3", 1, 3),
        ("4-5", 4, 5),
        ("6-10", 6, 10),
        ("11+", 11, i64::MAX),
    ] {
        let impressions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM search_feedback WHERE action = 'impression'
                 AND rank_position >= ?1 AND rank_position <= ?2",
                rusqlite::params![low, high],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let clicks: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM search_feedback
                 WHERE action IN ('view', 'copy_again') AND rank_position >= ?1 AND rank_position <= ?2",
                rusqlite::params![low, high],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        bands.push(PositionCtr {
            band: name.into(),
            impressions,
            clicks,
            ctr: if impressions == 0 {
                0.0
            } else {
                clicks as f64 / impressions as f64
            },
        });
    }
    Ok(QualityMetrics {
        total_clips: total,
        embedded_clips: embedded,
        pending_clips: pending,
        failed_clips: failed,
        skipped_clips: skipped,
        embedding_coverage: if total == 0 {
            0.0
        } else {
            embedded as f64 / total as f64
        },
        duplicate_count,
        edge_count,
        empty_search_rate: if all_queries == 0 {
            0.0
        } else {
            searches as f64 / all_queries as f64
        },
        ctr_by_position: bands,
    })
}

#[tauri::command]
pub fn log_search_feedback(
    state: State<'_, AppState>,
    query: String,
    query_type: String,
    result_clip_id: Option<String>,
    rank_position: Option<i64>,
    action: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    crate::services::search_feedback::log(
        &conn,
        &query,
        &query_type,
        result_clip_id.as_deref(),
        rank_position,
        &action,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn log_copy_again(
    state: State<'_, AppState>,
    query: String,
    result_clip_id: String,
    rank_position: Option<i64>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    crate::services::search_feedback::log(
        &conn,
        &query,
        "hybrid",
        Some(&result_clip_id),
        rank_position,
        "copy_again",
    )
    .map_err(|e| e.to_string())
}
