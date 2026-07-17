use rusqlite::{params, Connection};

const MAX_EDGES_PER_CLIP: i64 = 15;
const MAX_EDGES_PER_SESSION: i64 = 5;

pub fn process_new_clip(
    conn: &Connection,
    clip_id: &str,
    embedding: &[f32],
) -> rusqlite::Result<usize> {
    let table: String = conn.query_row(
        "SELECT table_name FROM embedding_registry WHERE slot = 'active'",
        [],
        |row| row.get(0),
    )?;
    if !table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Ok(0);
    }
    let bytes: Vec<u8> = embedding
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    let sql = format!(
        "SELECT clip_id, distance FROM {table}
         WHERE embedding MATCH ?1 AND k = 20 ORDER BY distance"
    );
    let mut stmt = conn.prepare(&sql)?;
    let candidates: Vec<(String, f64)> = stmt
        .query_map(params![bytes], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|row| row.ok())
        .filter(|(id, distance)| id != clip_id && 1.0 - distance > 0.65)
        .collect();

    let new_time: i64 = conn.query_row(
        "SELECT copied_at FROM clips WHERE id = ?1",
        [clip_id],
        |row| row.get(0),
    )?;
    let mut created = 0;
    for (other_id, distance) in candidates {
        if !can_add_edge(conn, clip_id, &other_id)? {
            continue;
        }
        let other_time: i64 = conn.query_row(
            "SELECT copied_at FROM clips WHERE id = ?1",
            [&other_id],
            |row| row.get(0),
        )?;
        let proximity = (-((new_time - other_time).unsigned_abs() as f64) / 604_800_000.0).exp();
        let temporal_weight = 0.7 + 0.3 * proximity;
        let similarity = (1.0 - distance).clamp(0.0, 1.0) * temporal_weight;
        let (a, b) = if clip_id < other_id.as_str() {
            (clip_id, other_id.as_str())
        } else {
            (other_id.as_str(), clip_id)
        };
        created += conn.execute(
            "INSERT OR IGNORE INTO memory_edges
             (clip_a_id, clip_b_id, similarity, temporal_weight, edge_type, detected_at)
             VALUES (?1, ?2, ?3, ?4, 'semantic_temporal', ?5)",
            params![
                a,
                b,
                similarity,
                temporal_weight,
                chrono::Utc::now().timestamp_millis()
            ],
        )?;
    }
    Ok(created)
}

fn can_add_edge(conn: &Connection, a: &str, b: &str) -> rusqlite::Result<bool> {
    let total_a: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_edges WHERE clip_a_id = ?1 OR clip_b_id = ?1",
        [a],
        |row| row.get(0),
    )?;
    let total_b: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_edges WHERE clip_a_id = ?1 OR clip_b_id = ?1",
        [b],
        |row| row.get(0),
    )?;
    if total_a >= MAX_EDGES_PER_CLIP || total_b >= MAX_EDGES_PER_CLIP {
        return Ok(false);
    }
    let same_session: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_edges e
         JOIN clips ca ON ca.id = CASE WHEN e.clip_a_id = ?1 THEN e.clip_b_id ELSE e.clip_a_id END
         JOIN clips cb ON cb.id = ?2
         WHERE (e.clip_a_id = ?1 OR e.clip_b_id = ?1)
           AND ca.session_id IS NOT NULL AND ca.session_id = cb.session_id",
        params![a, b],
        |row| row.get(0),
    )?;
    Ok(same_session < MAX_EDGES_PER_SESSION)
}
