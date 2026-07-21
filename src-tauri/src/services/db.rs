use std::{
    fs,
    path::{Path, PathBuf},
};

use rusqlite::{
    auto_extension::{register_auto_extension, RawAutoExtension},
    Connection, OptionalExtension,
};

const ACTIVE_VECTOR_TABLE: &str = "clips_embeddings";
const ACTIVE_VECTOR_DIMENSIONS: usize = 384;

const SCHEMA: &str = r#"
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS clips (
  id TEXT PRIMARY KEY,
  content TEXT NOT NULL,
  content_type TEXT NOT NULL DEFAULT 'text',
  image_path TEXT,
  source_url TEXT,
  page_title TEXT,
  app_name TEXT,
  window_title TEXT,
  language TEXT,
  session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
  is_sensitive INTEGER NOT NULL DEFAULT 0,
  is_pinned INTEGER NOT NULL DEFAULT 0,
  copied_at INTEGER NOT NULL,
  ai_context TEXT,
  embedding_id TEXT,
  normalized_content TEXT,
  content_hash TEXT,
  is_duplicate INTEGER NOT NULL DEFAULT 0,
  duplicate_of TEXT REFERENCES clips(id),
  embedding_model TEXT,
  embedding_version TEXT,
  embedding_status TEXT NOT NULL DEFAULT 'pending',
  source_intent TEXT NOT NULL DEFAULT 'other',
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  label TEXT,
  summary TEXT,
  key_topics TEXT,
  source_apps TEXT,
  source_urls TEXT,
  clip_count INTEGER NOT NULL DEFAULT 0,
  started_at INTEGER NOT NULL,
  ended_at INTEGER NOT NULL,
  reconstructed INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS memory_edges (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  clip_a_id TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
  clip_b_id TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
  similarity REAL NOT NULL,
  temporal_weight REAL NOT NULL DEFAULT 1.0,
  edge_type TEXT NOT NULL DEFAULT 'semantic',
  detected_at INTEGER NOT NULL,
  UNIQUE(clip_a_id, clip_b_id)
);

CREATE TABLE IF NOT EXISTS memory_facts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  fact_type TEXT NOT NULL,
  fact_key TEXT NOT NULL,
  fact_value TEXT NOT NULL,
  clip_ids TEXT,
  first_seen INTEGER,
  last_seen INTEGER,
  occurrence_count INTEGER NOT NULL DEFAULT 1,
  UNIQUE(fact_type, fact_key)
);

CREATE TABLE IF NOT EXISTS memory_state (
  id INTEGER PRIMARY KEY DEFAULT 1,
  current_stage TEXT NOT NULL DEFAULT 'clippy',
  total_clips INTEGER NOT NULL DEFAULT 0,
  total_sessions INTEGER NOT NULL DEFAULT 0,
  total_edges INTEGER NOT NULL DEFAULT 0,
  total_facts INTEGER NOT NULL DEFAULT 0,
  last_analysis INTEGER
);

CREATE TABLE IF NOT EXISTS filter_rules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  rule_type TEXT NOT NULL,
  pattern TEXT NOT NULL,
  action TEXT NOT NULL DEFAULT 'block',
  enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS settings (
  id INTEGER PRIMARY KEY DEFAULT 1,
  hotkey TEXT NOT NULL DEFAULT 'CmdOrCtrl+Shift+V',
  storage_limit INTEGER DEFAULT 10000,
  auto_delete_days INTEGER,
  extension_enabled INTEGER DEFAULT 0,
  ollama_enabled INTEGER DEFAULT 0,
  ollama_url TEXT DEFAULT 'http://localhost:11434',
  ai_provider TEXT NOT NULL DEFAULT 'none',
  ai_model TEXT NOT NULL DEFAULT 'llama3.2:3b',
  ai_api_key TEXT,
  ai_ollama_url TEXT NOT NULL DEFAULT 'http://localhost:11434',
  ai_cloud_consent INTEGER NOT NULL DEFAULT 0,
  capture_enabled INTEGER NOT NULL DEFAULT 0,
  browser_context_enabled INTEGER NOT NULL DEFAULT 0,
  appearance TEXT NOT NULL DEFAULT 'dark',
  onboarding_completed INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS embedding_registry (
  slot TEXT PRIMARY KEY,
  model TEXT NOT NULL,
  version TEXT NOT NULL,
  dimensions INTEGER NOT NULL,
  table_name TEXT NOT NULL,
  state TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS search_feedback (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  query TEXT NOT NULL,
  query_type TEXT NOT NULL,
  result_clip_id TEXT REFERENCES clips(id) ON DELETE CASCADE,
  rank_position INTEGER,
  action TEXT NOT NULL,
  occurred_at INTEGER NOT NULL
);

INSERT OR IGNORE INTO settings (id) VALUES (1);
INSERT OR IGNORE INTO memory_state (id) VALUES (1);

INSERT OR IGNORE INTO filter_rules (rule_type, pattern, action) VALUES
  ('regex', '^\\d{4,8}$', 'block'),
  ('regex', '\\b\\d{4}[\\s-]?\\d{4}[\\s-]?\\d{4}[\\s-]?\\d{4}\\b', 'block'),
  ('regex', '(?i)[A-Z_]+=.{8,}', 'block'),
  ('regex', '-----BEGIN [A-Z ]*KEY-----', 'block'),
  ('regex', 'eyJ[A-Za-z0-9_-]{10,}\\.[A-Za-z0-9_-]{10,}', 'block'),
  ('regex', 'AKIA[0-9A-Z]{16}', 'block'),
  ('regex', '(?i)password\\s*[:=]\\s*\\S+', 'block');

CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
  content, ai_context, page_title, app_name,
  content='clips', content_rowid='rowid'
);

CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts_code USING fts5(
  content, page_title, app_name,
  content='clips', content_rowid='rowid', tokenize='trigram'
);

CREATE TRIGGER IF NOT EXISTS clips_ai AFTER INSERT ON clips BEGIN
  INSERT INTO clips_fts(rowid, content, ai_context, page_title, app_name)
  VALUES (new.rowid, new.content, new.ai_context, new.page_title, new.app_name);
  INSERT INTO clips_fts_code(rowid, content, page_title, app_name)
  VALUES (new.rowid, new.content, new.page_title, new.app_name);
END;

CREATE TRIGGER IF NOT EXISTS clips_ad AFTER DELETE ON clips BEGIN
  INSERT INTO clips_fts(clips_fts, rowid, content, ai_context, page_title, app_name)
  VALUES('delete', old.rowid, old.content, old.ai_context, old.page_title, old.app_name);
  INSERT INTO clips_fts_code(clips_fts_code, rowid, content, page_title, app_name)
  VALUES('delete', old.rowid, old.content, old.page_title, old.app_name);
END;

CREATE TRIGGER IF NOT EXISTS clips_au AFTER UPDATE ON clips BEGIN
  INSERT INTO clips_fts(clips_fts, rowid, content, ai_context, page_title, app_name)
  VALUES('delete', old.rowid, old.content, old.ai_context, old.page_title, old.app_name);
  INSERT INTO clips_fts(rowid, content, ai_context, page_title, app_name)
  VALUES (new.rowid, new.content, new.ai_context, new.page_title, new.app_name);
  INSERT INTO clips_fts_code(clips_fts_code, rowid, content, page_title, app_name)
  VALUES('delete', old.rowid, old.content, old.page_title, old.app_name);
  INSERT INTO clips_fts_code(rowid, content, page_title, app_name)
  VALUES (new.rowid, new.content, new.page_title, new.app_name);
END;
"#;

pub fn init_db(app_data_dir: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let mnemo_dir = app_data_dir.join("mnemo");
    fs::create_dir_all(mnemo_dir.join("images"))?;

    // sqlite-vec uses SQLite's auto-extension hook, which applies only to
    // connections opened after registration.
    let sqlite_vec_init = unsafe {
        std::mem::transmute::<*const (), RawAutoExtension>(
            sqlite_vec::sqlite3_vec_init as *const (),
        )
    };
    unsafe { register_auto_extension(sqlite_vec_init)? };
    let connection = Connection::open(database_path(&mnemo_dir))?;

    connection.execute_batch(SCHEMA)?;
    migrate_columns(&connection)?;
    let now = chrono::Utc::now().timestamp_millis();
    connection.execute(
        "INSERT OR IGNORE INTO embedding_registry
         (slot, model, version, dimensions, table_name, state, updated_at)
         VALUES ('active', 'bge-small-en-v1.5', '1', 384, 'clips_embeddings', 'ready', ?1)",
        [now],
    )?;
    if ensure_cosine_vector_index(&connection)? {
        rebuild_memory_edges(&connection)?;
    }
    Ok(connection)
}

fn ensure_cosine_vector_index(connection: &Connection) -> rusqlite::Result<bool> {
    let definition = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [ACTIVE_VECTOR_TABLE],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    let Some(definition) = definition else {
        create_cosine_vector_index(connection)?;
        return Ok(false);
    };
    if definition
        .to_ascii_lowercase()
        .contains("distance_metric=cosine")
    {
        return Ok(false);
    }

    // sqlite-vec fixes the distance metric when a vec0 table is created. Copy
    // through a distinct table before recreating the active index; renaming a
    // vec0 table does not rename its internal shadow-table names.
    let migration_table = "clips_embeddings_cosine_migration";
    connection.execute_batch(&format!("DROP TABLE IF EXISTS {migration_table};"))?;
    create_cosine_vector_index_named(connection, migration_table)?;
    connection.execute_batch(&format!(
        "INSERT INTO {migration_table} (clip_id, embedding)
         SELECT clip_id, embedding FROM {ACTIVE_VECTOR_TABLE};
         DROP TABLE {ACTIVE_VECTOR_TABLE};"
    ))?;
    create_cosine_vector_index(connection)?;
    connection.execute_batch(&format!(
        "INSERT INTO {ACTIVE_VECTOR_TABLE} (clip_id, embedding)
         SELECT clip_id, embedding FROM {migration_table};
         DROP TABLE {migration_table};"
    ))?;
    Ok(true)
}

fn create_cosine_vector_index(connection: &Connection) -> rusqlite::Result<()> {
    create_cosine_vector_index_named(connection, ACTIVE_VECTOR_TABLE)
}

fn create_cosine_vector_index_named(
    connection: &Connection,
    table_name: &str,
) -> rusqlite::Result<()> {
    connection.execute_batch(&format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS {table_name} USING vec0(
           clip_id TEXT PRIMARY KEY,
           embedding float[{ACTIVE_VECTOR_DIMENSIONS}] distance_metric=cosine
         );"
    ))
}

fn rebuild_memory_edges(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute("DELETE FROM memory_edges", [])?;
    let embeddings: Vec<(String, Vec<u8>)> = {
        let mut statement = connection.prepare(&format!(
            "SELECT v.clip_id, v.embedding
             FROM {ACTIVE_VECTOR_TABLE} v
             JOIN clips c ON c.id = v.clip_id
             WHERE c.is_duplicate = 0
             ORDER BY c.copied_at ASC"
        ))?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<_, _>>()?;
        rows
    };

    for (clip_id, bytes) in embeddings {
        let Some(vector) = decode_embedding(&bytes) else {
            log::warn!("[db] Skipping malformed embedding for clip {clip_id}");
            continue;
        };
        crate::services::memory_graph::process_new_clip(connection, &clip_id, &vector)?;
    }
    crate::services::intelligence::refresh_state(connection)?;
    Ok(())
}

fn decode_embedding(bytes: &[u8]) -> Option<Vec<f32>> {
    if !bytes.len().is_multiple_of(std::mem::size_of::<f32>()) {
        return None;
    }
    Some(
        bytes
            .chunks_exact(std::mem::size_of::<f32>())
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect(),
    )
}

fn migrate_columns(connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let mut stmt = connection.prepare("PRAGMA table_info(clips)")?;
    let columns: std::collections::HashSet<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<_, _>>()?;

    let additions = [
        ("normalized_content", "TEXT"),
        ("content_hash", "TEXT"),
        ("is_duplicate", "INTEGER NOT NULL DEFAULT 0"),
        ("duplicate_of", "TEXT REFERENCES clips(id)"),
        ("embedding_model", "TEXT"),
        ("embedding_version", "TEXT"),
        ("embedding_status", "TEXT NOT NULL DEFAULT 'pending'"),
        ("source_intent", "TEXT NOT NULL DEFAULT 'other'"),
    ];

    for (name, definition) in additions {
        if !columns.contains(name) {
            connection
                .execute_batch(&format!("ALTER TABLE clips ADD COLUMN {name} {definition}"))?;
        }
    }
    let mut edge_columns = connection.prepare("PRAGMA table_info(memory_edges)")?;
    let edge_column_names: std::collections::HashSet<String> = edge_columns
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<_, _>>()?;
    if !edge_column_names.contains("temporal_weight") {
        connection.execute_batch(
            "ALTER TABLE memory_edges ADD COLUMN temporal_weight REAL NOT NULL DEFAULT 1.0",
        )?;
    }

    let mut settings_stmt = connection.prepare("PRAGMA table_info(settings)")?;
    let settings_columns: std::collections::HashSet<String> = settings_stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<_, _>>()?;
    let settings_additions = [
        ("capture_enabled", "INTEGER NOT NULL DEFAULT 1"),
        ("browser_context_enabled", "INTEGER NOT NULL DEFAULT 0"),
        ("appearance", "TEXT NOT NULL DEFAULT 'dark'"),
        ("onboarding_completed", "INTEGER NOT NULL DEFAULT 0"),
        ("ai_provider", "TEXT NOT NULL DEFAULT 'none'"),
        ("ai_model", "TEXT NOT NULL DEFAULT 'llama3.2:3b'"),
        ("ai_api_key", "TEXT"),
        (
            "ai_ollama_url",
            "TEXT NOT NULL DEFAULT 'http://localhost:11434'",
        ),
        ("ai_cloud_consent", "INTEGER NOT NULL DEFAULT 0"),
    ];
    for (name, definition) in settings_additions {
        if !settings_columns.contains(name) {
            connection.execute_batch(&format!(
                "ALTER TABLE settings ADD COLUMN {name} {definition}"
            ))?;
        }
    }

    // Preserve the legacy Ollama opt-in when introducing the provider selector.
    connection.execute(
        "UPDATE settings SET ai_provider = 'ollama', ai_ollama_url = ollama_url
         WHERE id = 1 AND ai_provider = 'none' AND ollama_enabled = 1",
        [],
    )?;

    connection.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_clips_content_hash ON clips(content_hash);
         CREATE INDEX IF NOT EXISTS idx_clips_embedding_status ON clips(embedding_status);
         CREATE INDEX IF NOT EXISTS idx_feedback_query ON search_feedback(query);
         CREATE TRIGGER IF NOT EXISTS clips_code_ai AFTER INSERT ON clips BEGIN
           INSERT INTO clips_fts_code(rowid, content, page_title, app_name)
           VALUES (new.rowid, new.content, new.page_title, new.app_name);
         END;
         CREATE TRIGGER IF NOT EXISTS clips_code_ad AFTER DELETE ON clips BEGIN
           INSERT INTO clips_fts_code(clips_fts_code, rowid, content, page_title, app_name)
           VALUES('delete', old.rowid, old.content, old.page_title, old.app_name);
         END;
         CREATE TRIGGER IF NOT EXISTS clips_code_au AFTER UPDATE ON clips BEGIN
           INSERT INTO clips_fts_code(clips_fts_code, rowid, content, page_title, app_name)
           VALUES('delete', old.rowid, old.content, old.page_title, old.app_name);
           INSERT INTO clips_fts_code(rowid, content, page_title, app_name)
           VALUES (new.rowid, new.content, new.page_title, new.app_name);
         END;
         INSERT OR IGNORE INTO clips_fts_code(rowid, content, page_title, app_name)
           SELECT rowid, content, page_title, app_name FROM clips;",
    )?;

    let existing: Vec<(String, String)> = {
        let mut stmt = connection.prepare(
            "SELECT id, content FROM clips WHERE content_hash IS NULL OR normalized_content IS NULL",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<_, _>>()?;
        rows
    };
    for (id, content) in existing {
        let normalized = crate::services::content_dedup::normalize_content(&content);
        let hash = crate::services::content_dedup::content_hash(&normalized);
        connection.execute(
            "UPDATE clips SET normalized_content = ?1, content_hash = ?2 WHERE id = ?3",
            rusqlite::params![normalized, hash, id],
        )?;
    }

    Ok(())
}

fn database_path(mnemo_dir: &Path) -> PathBuf {
    mnemo_dir.join("clips.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_app_dir() -> PathBuf {
        std::env::temp_dir().join(format!("mnemo-db-test-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn creates_a_cosine_vector_index() {
        let root = temporary_app_dir();
        let connection = init_db(&root).unwrap();
        let definition: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [ACTIVE_VECTOR_TABLE],
                |row| row.get(0),
            )
            .unwrap();

        assert!(definition
            .to_ascii_lowercase()
            .contains("distance_metric=cosine"));
        drop(connection);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fresh_settings_require_capture_consent_and_keep_everything() {
        let root = temporary_app_dir();
        let connection = init_db(&root).unwrap();
        let (capture_enabled, browser_context_enabled, auto_delete_days): (i32, i32, Option<i64>) =
            connection
                .query_row(
                    "SELECT capture_enabled, browser_context_enabled, auto_delete_days
                     FROM settings WHERE id = 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .unwrap();

        assert_eq!(capture_enabled, 0);
        assert_eq!(browser_context_enabled, 0);
        assert_eq!(auto_delete_days, None);
        drop(connection);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn migrates_a_legacy_l2_index_without_losing_vectors() {
        let root = temporary_app_dir();
        let connection = init_db(&root).unwrap();
        connection
            .execute_batch(&format!(
                "DROP TABLE {ACTIVE_VECTOR_TABLE};
                 CREATE VIRTUAL TABLE {ACTIVE_VECTOR_TABLE} USING vec0(
                   clip_id TEXT PRIMARY KEY,
                   embedding float[{ACTIVE_VECTOR_DIMENSIONS}]
                 );"
            ))
            .unwrap();
        let bytes = vec![0.25_f32; ACTIVE_VECTOR_DIMENSIONS]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        connection
            .execute(
                &format!("INSERT INTO {ACTIVE_VECTOR_TABLE} (clip_id, embedding) VALUES (?1, ?2)"),
                rusqlite::params!["legacy-clip", bytes],
            )
            .unwrap();

        assert!(ensure_cosine_vector_index(&connection).unwrap());
        let definition: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [ACTIVE_VECTOR_TABLE],
                |row| row.get(0),
            )
            .unwrap();
        let count: i64 = connection
            .query_row(
                &format!("SELECT COUNT(*) FROM {ACTIVE_VECTOR_TABLE}"),
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(definition
            .to_ascii_lowercase()
            .contains("distance_metric=cosine"));
        assert_eq!(count, 1);
        drop(connection);
        std::fs::remove_dir_all(root).unwrap();
    }
}
