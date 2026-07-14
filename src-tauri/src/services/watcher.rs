use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use chrono::Utc;
use rusqlite::Connection;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::models::clip::ClipAddedPayload;
use crate::services::{active_window, filter};

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const MAX_CONTENT_LENGTH: usize = 100_000;

pub fn start(app_handle: AppHandle, db: Arc<Mutex<Connection>>) {
    thread::spawn(move || {
        log::info!("[watcher] Clipboard watcher started");

        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(e) => {
                log::error!("[watcher] Failed to access clipboard: {e}");
                return;
            }
        };

        let mut last_content = String::new();

        let rules = {
            let conn = db.lock().unwrap();
            filter::load_rules(&conn)
        };

        loop {
            thread::sleep(POLL_INTERVAL);

            let text = match clipboard.get_text() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let trimmed = text.trim();
            if trimmed.is_empty() || trimmed == last_content || text.len() > MAX_CONTENT_LENGTH {
                continue;
            }

            last_content = trimmed.to_string();

            let filter_result = filter::evaluate(trimmed, &rules);
            if filter_result.blocked {
                log::info!(
                    "[watcher] Blocked sensitive content (rule: {:?})",
                    filter_result.matched_rule
                );
                continue;
            }

            let window_info = active_window::get_active_window();

            let clip_id = Uuid::new_v4().to_string();
            let now = Utc::now().timestamp();
            let content_type = detect_content_type(trimmed);
            let language = if content_type == "code" {
                detect_language(trimmed)
            } else {
                None
            };

            {
                let conn = db.lock().unwrap();
                let result = conn.execute(
                    "INSERT INTO clips (id, content, content_type, app_name, window_title, language, copied_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        clip_id,
                        trimmed,
                        content_type,
                        window_info.app_name,
                        window_info.window_title,
                        language,
                        now,
                    ],
                );

                match result {
                    Ok(_) => {
                        let _ = conn.execute(
                            "UPDATE memory_state SET total_clips = total_clips + 1 WHERE id = 1",
                            [],
                        );
                        log::info!("[watcher] Clip saved: {}", &clip_id[..8]);
                    }
                    Err(e) => {
                        log::error!("[watcher] Failed to insert clip: {e}");
                        continue;
                    }
                }
            }

            let preview = if trimmed.len() > 120 {
                format!("{}…", &trimmed[..120])
            } else {
                trimmed.to_string()
            };

            let payload = ClipAddedPayload {
                clip_id,
                content_preview: preview,
                content_type: content_type.to_string(),
                app_name: Some(window_info.app_name),
                copied_at: now,
            };

            if let Err(e) = app_handle.emit("clip-added", &payload) {
                log::error!("[watcher] Failed to emit clip-added: {e}");
            }
        }
    });
}

fn detect_content_type(content: &str) -> &'static str {
    let trimmed = content.trim();

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        if trimmed.lines().count() == 1 {
            return "url";
        }
    }

    let code_indicators = [
        "fn ", "pub ", "let ", "const ", "function ", "import ", "export ",
        "class ", "def ", "return ", "if (", "for (", "while (",
        "=>", "->", "::", "#{", "$(", "#!/",
    ];
    let bracket_heavy = trimmed.matches('{').count() + trimmed.matches('}').count();
    let has_semicolons = trimmed.matches(';').count() > 1;
    let has_code_kw = code_indicators.iter().any(|kw| trimmed.contains(kw));

    if (bracket_heavy >= 2 && has_semicolons) || (has_code_kw && trimmed.lines().count() > 2) {
        return "code";
    }

    "text"
}

fn detect_language(content: &str) -> Option<String> {
    let t = content.trim();
    if t.contains("fn ") && (t.contains("pub ") || t.contains("let ") || t.contains("::")) {
        Some("rust".into())
    } else if t.contains("function ") || t.contains("=>") || t.contains("const ") {
        if t.contains(": ") && (t.contains("interface ") || t.contains("type ")) {
            Some("typescript".into())
        } else {
            Some("javascript".into())
        }
    } else if t.contains("def ") && t.contains(":") {
        Some("python".into())
    } else if t.contains("package ") && t.contains("func ") {
        Some("go".into())
    } else {
        None
    }
}
