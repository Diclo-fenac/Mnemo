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

use crate::state::BrowserContext;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const MAX_CONTENT_LENGTH: usize = 100_000;
const SESSION_TIMEOUT: i64 = 15 * 60; // 15 mins

pub fn start(
    app_handle: AppHandle, 
    db: Arc<Mutex<Connection>>, 
    context_state: Arc<Mutex<Option<BrowserContext>>>
) {
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

        let mut last_copied_at: i64 = 0;
        let mut current_session_id: Option<String> = None;

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

            let now = Utc::now().timestamp();
            
            // Session logic
            if now - last_copied_at > SESSION_TIMEOUT {
                let sid = Uuid::new_v4().to_string();
                current_session_id = Some(sid.clone());
                
                let conn = db.lock().unwrap();
                let _ = conn.execute(
                    "INSERT INTO sessions (id, started_at, ended_at, label) VALUES (?1, ?2, ?3, 'New Session')",
                    rusqlite::params![sid, now, now],
                );
                let _ = conn.execute(
                    "UPDATE memory_state SET total_sessions = total_sessions + 1 WHERE id = 1",
                    [],
                );
                log::info!("[watcher] Started new session: {}", &sid[..8]);
            } else if let Some(sid) = &current_session_id {
                let conn = db.lock().unwrap();
                let _ = conn.execute(
                    "UPDATE sessions SET ended_at = ?1, clip_count = clip_count + 1 WHERE id = ?2",
                    rusqlite::params![now, sid],
                );
            }
            
            last_copied_at = now;

            let window_info = active_window::get_active_window();
            
            // Context logic
            let (mut source_url, mut page_title) = (None, None);
            {
                let mut ctx_lock = context_state.lock().unwrap();
                if let Some(ctx) = ctx_lock.take() {
                    // Check if context is fresh (within 5 seconds)
                    if now - ctx.timestamp < 5 {
                        source_url = Some(ctx.url);
                        page_title = Some(ctx.title);
                    }
                }
            }

            let clip_id = Uuid::new_v4().to_string();
            let content_type = detect_content_type(trimmed);
            let language = if content_type == "code" {
                detect_language(trimmed)
            } else {
                None
            };

            {
                let conn = db.lock().unwrap();
                let result = conn.execute(
                    "INSERT INTO clips (id, content, content_type, app_name, window_title, language, copied_at, session_id, source_url, page_title)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    rusqlite::params![
                        clip_id,
                        trimmed,
                        content_type,
                        window_info.app_name.clone(),
                        window_info.window_title,
                        language,
                        now,
                        current_session_id,
                        source_url,
                        page_title
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
