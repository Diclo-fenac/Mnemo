use std::sync::atomic::{AtomicBool, Ordering};

use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturePreferences {
    pub capture_enabled: bool,
    pub browser_context_enabled: bool,
    pub auto_delete_days: Option<i64>,
    pub appearance: String,
}

impl Default for CapturePreferences {
    fn default() -> Self {
        Self {
            capture_enabled: true,
            browser_context_enabled: false,
            auto_delete_days: Some(30),
            appearance: "dark".to_string(),
        }
    }
}

pub fn load(conn: &Connection) -> rusqlite::Result<CapturePreferences> {
    conn.query_row(
        "SELECT capture_enabled, browser_context_enabled, auto_delete_days, appearance
         FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(CapturePreferences {
                capture_enabled: row.get::<_, i32>(0)? != 0,
                browser_context_enabled: row.get::<_, i32>(1)? != 0,
                auto_delete_days: row.get(2)?,
                appearance: row.get(3)?,
            })
        },
    )
}

pub fn persist(conn: &Connection, preferences: &CapturePreferences) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE settings
         SET capture_enabled = ?1, browser_context_enabled = ?2,
             auto_delete_days = ?3, appearance = ?4
         WHERE id = 1",
        params![
            i32::from(preferences.capture_enabled),
            i32::from(preferences.browser_context_enabled),
            preferences.auto_delete_days,
            preferences.appearance,
        ],
    )?;
    Ok(())
}

pub fn set_capture_enabled(conn: &Connection, enabled: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE settings SET capture_enabled = ?1 WHERE id = 1",
        [i32::from(enabled)],
    )?;
    Ok(())
}

pub fn is_enabled(flag: &AtomicBool) -> bool {
    flag.load(Ordering::Relaxed)
}

pub fn set_enabled(flag: &AtomicBool, enabled: bool) {
    flag.store(enabled, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE settings (
                id INTEGER PRIMARY KEY,
                capture_enabled INTEGER NOT NULL DEFAULT 1,
                browser_context_enabled INTEGER NOT NULL DEFAULT 0,
                auto_delete_days INTEGER,
                appearance TEXT NOT NULL DEFAULT 'dark'
            );
            INSERT INTO settings (id, auto_delete_days) VALUES (1, 30);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn defaults_to_capture_on_and_context_off() {
        let preferences = load(&connection()).unwrap();
        assert!(preferences.capture_enabled);
        assert!(!preferences.browser_context_enabled);
        assert_eq!(preferences.appearance, "dark");
    }

    #[test]
    fn persists_capture_toggle() {
        let conn = connection();
        set_capture_enabled(&conn, false).unwrap();
        assert!(!load(&conn).unwrap().capture_enabled);
    }
}
