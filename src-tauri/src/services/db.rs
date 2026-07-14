use std::{fs, path::{Path, PathBuf}};

use rusqlite::Connection;

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
  auto_delete_days INTEGER DEFAULT 30,
  extension_enabled INTEGER DEFAULT 0,
  ollama_enabled INTEGER DEFAULT 0,
  ollama_url TEXT DEFAULT 'http://localhost:11434'
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

CREATE TRIGGER IF NOT EXISTS clips_ai AFTER INSERT ON clips BEGIN
  INSERT INTO clips_fts(rowid, content, ai_context, page_title, app_name)
  VALUES (new.rowid, new.content, new.ai_context, new.page_title, new.app_name);
END;

CREATE TRIGGER IF NOT EXISTS clips_ad AFTER DELETE ON clips BEGIN
  INSERT INTO clips_fts(clips_fts, rowid, content, ai_context, page_title, app_name)
  VALUES('delete', old.rowid, old.content, old.ai_context, old.page_title, old.app_name);
END;

CREATE TRIGGER IF NOT EXISTS clips_au AFTER UPDATE ON clips BEGIN
  INSERT INTO clips_fts(clips_fts, rowid, content, ai_context, page_title, app_name)
  VALUES('delete', old.rowid, old.content, old.ai_context, old.page_title, old.app_name);
  INSERT INTO clips_fts(rowid, content, ai_context, page_title, app_name)
  VALUES (new.rowid, new.content, new.ai_context, new.page_title, new.app_name);
END;
"#;

pub fn init_db(app_data_dir: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let mnemo_dir = app_data_dir.join("mnemo");
    fs::create_dir_all(mnemo_dir.join("images"))?;

    let connection = Connection::open(database_path(&mnemo_dir))?;
    sqlite_vec::sqlite3_vec_init(&connection)?;

    connection.execute_batch(SCHEMA)?;
    connection.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS clips_embeddings USING vec0(
          clip_id TEXT PRIMARY KEY,
          embedding float[384]
        );"
    )?;
    Ok(connection)
}

fn database_path(mnemo_dir: &Path) -> PathBuf {
    mnemo_dir.join("clips.db")
}
