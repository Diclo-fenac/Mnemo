use rusqlite::Connection;

pub fn refresh_state(conn: &Connection) -> rusqlite::Result<()> {
    let clips: i64 = conn.query_row("SELECT COUNT(*) FROM clips", [], |row| row.get(0))?;
    let sessions: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
    let edges: i64 = conn.query_row("SELECT COUNT(*) FROM memory_edges", [], |row| row.get(0))?;
    let facts: i64 = conn.query_row("SELECT COUNT(*) FROM memory_facts", [], |row| row.get(0))?;
    let stage = if clips >= 200 && sessions >= 5 {
        "archivor"
    } else if clips >= 50 {
        "bindor"
    } else {
        "clippy"
    };
    conn.execute(
        "UPDATE memory_state SET current_stage = ?1, total_clips = ?2, total_sessions = ?3,
         total_edges = ?4, total_facts = ?5, last_analysis = ?6 WHERE id = 1",
        rusqlite::params![
            stage,
            clips,
            sessions,
            edges,
            facts,
            chrono::Utc::now().timestamp_millis()
        ],
    )?;
    Ok(())
}
