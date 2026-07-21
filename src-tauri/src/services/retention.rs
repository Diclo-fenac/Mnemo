use rusqlite::Connection;

const MAX_RETENTION_DAYS: i64 = 3650;

pub fn purge_expired(conn: &mut Connection, now_ms: i64) -> rusqlite::Result<usize> {
    let days = conn.query_row(
        "SELECT auto_delete_days FROM settings WHERE id = 1",
        [],
        |row| row.get::<_, Option<i64>>(0),
    )?;
    let Some(days) = days else {
        return Ok(0);
    };
    if !(1..=MAX_RETENTION_DAYS).contains(&days) {
        return Ok(0);
    }
    let cutoff = now_ms.saturating_sub(days.saturating_mul(86_400_000));
    let ids: Vec<String> = {
        let mut statement =
            conn.prepare("SELECT id FROM clips WHERE copied_at < ?1 AND is_pinned = 0")?;
        let rows = statement
            .query_map([cutoff], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    if ids.is_empty() {
        return Ok(0);
    }

    let table = active_embedding_table(conn)?;
    let tx = conn.unchecked_transaction()?;
    for id in &ids {
        tx.execute(&format!("DELETE FROM {table} WHERE clip_id = ?1"), [id])?;
        tx.execute("DELETE FROM clips WHERE id = ?1", [id])?;
    }
    tx.execute(
        "UPDATE memory_state SET total_clips = (SELECT COUNT(*) FROM clips),
         total_sessions = (SELECT COUNT(*) FROM sessions),
         total_edges = (SELECT COUNT(*) FROM memory_edges),
         total_facts = (SELECT COUNT(*) FROM memory_facts) WHERE id = 1",
        [],
    )?;
    tx.commit()?;
    Ok(ids.len())
}

fn active_embedding_table(conn: &Connection) -> rusqlite::Result<String> {
    let table = conn
        .query_row(
            "SELECT table_name FROM embedding_registry WHERE slot = 'active'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "clips_embeddings".to_string());
    if table.is_empty()
        || !table
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || char == '_')
    {
        return Err(rusqlite::Error::InvalidParameterName(
            "invalid embedding table".to_string(),
        ));
    }
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    fn connection(days: Option<i64>) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE settings (id INTEGER PRIMARY KEY, auto_delete_days INTEGER);
             CREATE TABLE clips (id TEXT PRIMARY KEY, copied_at INTEGER, is_pinned INTEGER);
             CREATE TABLE sessions (id TEXT PRIMARY KEY);
             CREATE TABLE memory_edges (id INTEGER PRIMARY KEY);
             CREATE TABLE memory_facts (id INTEGER PRIMARY KEY);
             CREATE TABLE memory_state (id INTEGER PRIMARY KEY, total_clips INTEGER, total_sessions INTEGER, total_edges INTEGER, total_facts INTEGER);
             CREATE TABLE embedding_registry (slot TEXT PRIMARY KEY, table_name TEXT);
             CREATE TABLE clips_embeddings (clip_id TEXT PRIMARY KEY);",
        ).unwrap();
        conn.execute(
            "INSERT INTO settings (id, auto_delete_days) VALUES (1, ?1)",
            [days],
        )
        .unwrap();
        conn.execute("INSERT INTO memory_state VALUES (1, 0, 0, 0, 0)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO embedding_registry VALUES ('active', 'clips_embeddings')",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn removes_old_unpinned_clips_but_preserves_pins() {
        let mut conn = connection(Some(30));
        let now = 4_000_000_000_i64;
        conn.execute(
            "INSERT INTO clips VALUES ('old', ?1, 0), ('pinned', ?1, 1), ('new', ?2, 0)",
            params![now - 31 * 86_400_000, now - 1],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO clips_embeddings VALUES ('old'), ('pinned'), ('new')",
            [],
        )
        .unwrap();
        assert_eq!(purge_expired(&mut conn, now).unwrap(), 1);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM clips", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM clips_embeddings WHERE clip_id = 'old'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
    }

    #[test]
    fn disabled_retention_keeps_clips() {
        let mut conn = connection(None);
        conn.execute("INSERT INTO clips VALUES ('old', 0, 0)", [])
            .unwrap();
        assert_eq!(purge_expired(&mut conn, 4_000_000_000).unwrap(), 0);
    }

    #[test]
    fn invalid_retention_values_fail_safe() {
        for days in [Some(0), Some(-1), Some(MAX_RETENTION_DAYS + 1)] {
            let mut conn = connection(days);
            conn.execute("INSERT INTO clips VALUES ('old', 0, 0)", [])
                .unwrap();
            assert_eq!(purge_expired(&mut conn, 4_000_000_000).unwrap(), 0);
            assert_eq!(
                conn.query_row("SELECT COUNT(*) FROM clips", [], |row| row.get::<_, i64>(0))
                    .unwrap(),
                1
            );
        }
    }
}
