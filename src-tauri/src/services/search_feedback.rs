use rusqlite::{params, Connection};

pub fn log(
    conn: &Connection,
    query: &str,
    query_type: &str,
    result_clip_id: Option<&str>,
    rank_position: Option<i64>,
    action: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO search_feedback
         (query, query_type, result_clip_id, rank_position, action, occurred_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            query,
            query_type,
            result_clip_id,
            rank_position,
            action,
            chrono::Utc::now().timestamp_millis()
        ],
    )?;
    Ok(())
}
