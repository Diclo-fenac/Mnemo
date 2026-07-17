use std::sync::{atomic::AtomicBool, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use arboard::Clipboard;
use chrono::Utc;
use rusqlite::Connection;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::models::clip::ClipAddedPayload;
use crate::services::{active_window, content_dedup, filter, source_intent};

use crate::state::BrowserContext;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const MAX_CONTENT_LENGTH: usize = 100_000;
const SESSION_TIMEOUT: i64 = 15 * 60 * 1000; // 15 minutes in milliseconds
const BROWSER_CONTEXT_TTL_MS: i64 = 5_000;

pub fn start(
    app_handle: AppHandle,
    db: Arc<Mutex<Connection>>,
    context_state: Arc<Mutex<Option<BrowserContext>>>,
    capture_enabled: Arc<AtomicBool>,
    browser_context_enabled: Arc<AtomicBool>,
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
        let mut last_retention_check = Instant::now() - Duration::from_secs(301);

        loop {
            thread::sleep(POLL_INTERVAL);

            if last_retention_check.elapsed() >= Duration::from_secs(300) {
                if let Ok(mut conn) = db.lock() {
                    if let Err(error) = crate::services::retention::purge_expired(
                        &mut conn,
                        Utc::now().timestamp_millis(),
                    ) {
                        log::warn!("[watcher] Retention cleanup failed: {error}");
                    }
                }
                last_retention_check = Instant::now();
            }

            // Capture off means we do not read the clipboard at all.
            if !crate::services::capture_state::is_enabled(&capture_enabled) {
                continue;
            }

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

            let now = Utc::now().timestamp_millis();

            // Session logic
            if now - last_copied_at > SESSION_TIMEOUT {
                let sid = Uuid::new_v4().to_string();
                current_session_id = Some(sid.clone());

                let conn = db.lock().unwrap();
                let _ = conn.execute(
                    "INSERT INTO sessions (id, started_at, ended_at, label, clip_count) VALUES (?1, ?2, ?3, 'New Session', 1)",
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
            if crate::services::capture_state::is_enabled(&browser_context_enabled) {
                let mut ctx_lock = context_state.lock().unwrap();
                if let Some(ctx) = ctx_lock.take() {
                    // The extension and clipboard event happen separately, so
                    // retain only context from the preceding five seconds.
                    if browser_context_is_fresh(now, ctx.timestamp) {
                        source_url = Some(ctx.url);
                        page_title = Some(ctx.title);
                    }
                }
            }

            let clip_id = Uuid::new_v4().to_string();
            let content_type = detect_content_type(trimmed);
            let normalized_content = content_dedup::normalize_content(trimmed);
            let content_hash = content_dedup::content_hash(&normalized_content);
            let source = source_intent::detect(
                Some(&window_info.app_name),
                source_url.as_deref(),
                page_title.as_deref(),
            );
            let duplicate_of = {
                let conn = db.lock().unwrap();
                content_dedup::find_original(&conn, &content_hash)
                    .ok()
                    .flatten()
            };
            let language = if content_type == "code" {
                detect_language(trimmed)
            } else {
                None
            };

            {
                let conn = db.lock().unwrap();
                let result = conn.execute(
                    "INSERT INTO clips
                     (id, content, content_type, app_name, window_title, language,
                      copied_at, session_id, source_url, page_title, normalized_content,
                      content_hash, is_duplicate, duplicate_of, embedding_status, source_intent)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                             ?13, ?14, ?15, ?16)",
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
                        page_title,
                        normalized_content,
                        content_hash,
                        i32::from(duplicate_of.is_some()),
                        duplicate_of.clone(),
                        if duplicate_of.is_some() {
                            "skipped"
                        } else {
                            "pending"
                        },
                        source.as_str(),
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

            if let Some(session_id) = current_session_id.as_deref() {
                if let Ok(conn) = db.lock() {
                    if let Err(error) =
                        crate::services::session_builder::refresh_session(&conn, session_id)
                    {
                        log::warn!("[watcher] Session labeling failed: {error}");
                    }
                }
            }
            if let Ok(conn) = db.lock() {
                if let Err(error) = crate::services::intelligence::refresh_state(&conn) {
                    log::warn!("[watcher] Intelligence state update failed: {error}");
                }
            }

            if let Some(original_id) = duplicate_of {
                log::info!(
                    "[watcher] Duplicate event {} of {}",
                    &clip_id[..8],
                    &original_id[..8]
                );
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

    if (trimmed.starts_with("http://") || trimmed.starts_with("https://"))
        && trimmed.lines().count() == 1
    {
        return "url";
    }

    let code_indicators = [
        "fn ",
        "pub ",
        "let ",
        "const ",
        "function ",
        "import ",
        "export ",
        "class ",
        "def ",
        "return ",
        "if (",
        "for (",
        "while (",
        "=>",
        "->",
        "::",
        "#{",
        "$(",
        "#!/",
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

fn browser_context_is_fresh(now: i64, context_timestamp: i64) -> bool {
    now.saturating_sub(context_timestamp) <= BROWSER_CONTEXT_TTL_MS
}

#[cfg(test)]
mod tests {
    use super::browser_context_is_fresh;

    #[test]
    fn keeps_only_recent_browser_context() {
        assert!(browser_context_is_fresh(10_000, 5_000));
        assert!(!browser_context_is_fresh(10_001, 5_000));
    }
}
