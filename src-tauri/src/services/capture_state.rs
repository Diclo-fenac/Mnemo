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
    pub onboarding_completed: bool,
}

impl Default for CapturePreferences {
    fn default() -> Self {
        Self {
            capture_enabled: false,
            browser_context_enabled: false,
            auto_delete_days: None,
            appearance: "dark".to_string(),
            onboarding_completed: false,
        }
    }
}

pub fn load(conn: &Connection) -> rusqlite::Result<CapturePreferences> {
    conn.query_row(
        "SELECT capture_enabled, browser_context_enabled, auto_delete_days, appearance,
                onboarding_completed
         FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(CapturePreferences {
                capture_enabled: row.get::<_, i32>(0)? != 0,
                browser_context_enabled: row.get::<_, i32>(1)? != 0,
                auto_delete_days: row.get(2)?,
                appearance: row.get(3)?,
                onboarding_completed: row.get::<_, i32>(4)? != 0,
            })
        },
    )
}

pub fn persist(conn: &Connection, preferences: &CapturePreferences) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE settings
         SET capture_enabled = ?1, browser_context_enabled = ?2,
             auto_delete_days = ?3, appearance = ?4, onboarding_completed = ?5
         WHERE id = 1",
        params![
            i32::from(preferences.capture_enabled),
            i32::from(preferences.browser_context_enabled),
            preferences.auto_delete_days,
            preferences.appearance,
            i32::from(preferences.onboarding_completed),
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
                capture_enabled INTEGER NOT NULL DEFAULT 0,
                browser_context_enabled INTEGER NOT NULL DEFAULT 0,
                auto_delete_days INTEGER,
                appearance TEXT NOT NULL DEFAULT 'dark',
                onboarding_completed INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO settings (id) VALUES (1);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn fresh_preferences_keep_capture_paused_and_retention_disabled() {
        let preferences = load(&connection()).unwrap();
        assert!(!preferences.capture_enabled);
        assert!(!preferences.browser_context_enabled);
        assert_eq!(preferences.auto_delete_days, None);
        assert_eq!(preferences.appearance, "dark");
        assert!(!preferences.onboarding_completed);
    }

    #[test]
    fn persists_capture_toggle() {
        let conn = connection();
        set_capture_enabled(&conn, false).unwrap();
        assert!(!load(&conn).unwrap().capture_enabled);
    }
}
