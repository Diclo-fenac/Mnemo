use std::collections::BTreeMap;

use rusqlite::{params, Connection, Row};
use serde::Serialize;
use tauri::State;

use crate::{models::clip::Clip, state::AppState};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub key_topics: Vec<String>,
    pub source_apps: Vec<String>,
    pub source_urls: Vec<String>,
    pub clip_count: i64,
    pub started_at: i64,
    pub ended_at: i64,
    pub duration_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStat {
    pub label: String,
    pub count: i64,
    pub source_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConnection {
    pub clip_id: String,
    pub content_preview: String,
    pub similarity: f64,
    pub copied_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionReconstruction {
    pub session: SessionSummary,
    pub clips: Vec<Clip>,
    pub source_breakdown: Vec<SourceStat>,
    pub connections: Vec<SessionConnection>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedClip {
    pub id: String,
    pub content: String,
    pub source_url: Option<String>,
    pub page_title: Option<String>,
    pub app_name: Option<String>,
    pub copied_at: i64,
    pub similarity: f64,
    pub edge_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipContext {
    pub source: String,
    pub likely_purpose: String,
    pub topic_tags: Vec<String>,
}

#[tauri::command]
pub fn list_sessions(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<SessionSummary>, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    let session_ids: Vec<String> = {
        let mut ids = conn
            .prepare("SELECT id FROM sessions")
            .map_err(|error| error.to_string())?;
        let rows = ids.query_map([], |row| row.get(0))
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        rows
    };
    for session_id in session_ids {
        crate::services::session_builder::refresh_session(&conn, &session_id)
            .map_err(|error| format!("Unable to refresh session: {error}"))?;
    }
    let mut statement = conn
        .prepare(
            "SELECT id, label, summary, key_topics, source_apps, source_urls, clip_count, started_at, ended_at
             FROM sessions ORDER BY started_at DESC LIMIT ?1",
        )
        .map_err(|error| error.to_string())?;
    let sessions = statement
        .query_map([limit.unwrap_or(100).clamp(1, 500)], session_from_row)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(sessions)
}

#[tauri::command]
pub fn get_session_reconstruction(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<SessionReconstruction, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    crate::services::session_builder::refresh_session(&conn, &session_id)
        .map_err(|error| format!("Unable to refresh session: {error}"))?;
    let session = conn
        .query_row(
            "SELECT id, label, summary, key_topics, source_apps, source_urls, clip_count, started_at, ended_at
             FROM sessions WHERE id = ?1",
            [&session_id],
            session_from_row,
        )
        .map_err(|error| format!("Session not found: {error}"))?;
    let clips = session_clips(&conn, &session_id)?;
    let source_breakdown = source_breakdown(&clips);
    let connections = cross_session_connections(&conn, &session_id)?;

    Ok(SessionReconstruction {
        session,
        clips,
        source_breakdown,
        connections,
    })
}

#[tauri::command]
pub fn get_related_clips(
    state: State<'_, AppState>,
    clip_id: String,
    limit: Option<i64>,
) -> Result<Vec<RelatedClip>, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    let mut statement = conn
        .prepare(
            "SELECT c.id, c.content, c.source_url, c.page_title, c.app_name, c.copied_at, e.similarity, e.edge_type
             FROM memory_edges e
             JOIN clips c ON c.id = CASE WHEN e.clip_a_id = ?1 THEN e.clip_b_id ELSE e.clip_a_id END
             WHERE e.clip_a_id = ?1 OR e.clip_b_id = ?1
             ORDER BY e.similarity DESC LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;
    let related = statement
        .query_map(params![clip_id, limit.unwrap_or(5).clamp(1, 20)], |row| {
            Ok(RelatedClip {
                id: row.get(0)?,
                content: row.get(1)?,
                source_url: row.get(2)?,
                page_title: row.get(3)?,
                app_name: row.get(4)?,
                copied_at: row.get(5)?,
                similarity: row.get(6)?,
                edge_type: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(related)
}

#[tauri::command]
pub fn get_clip_context(
    state: State<'_, AppState>,
    clip_id: String,
) -> Result<ClipContext, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    let (content, page_title, app_name, source_url, ai_context): (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT content, page_title, app_name, source_url, ai_context FROM clips WHERE id = ?1",
            [&clip_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|error| format!("Clip not found: {error}"))?;

    if let Some(raw) = ai_context {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
            let tags = value
                .get("topic_tags")
                .or_else(|| value.get("topicTags"))
                .and_then(serde_json::Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_else(|| infer_tags(&content));
            return Ok(ClipContext {
                source: value
                    .get("source")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| {
                        source_label(
                            page_title.as_deref(),
                            app_name.as_deref(),
                            source_url.as_deref(),
                        )
                    }),
                likely_purpose: value
                    .get("likely_purpose")
                    .or_else(|| value.get("likelyPurpose"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| infer_purpose(&content).to_string()),
                topic_tags: tags,
            });
        }
    }

    Ok(ClipContext {
        source: source_label(
            page_title.as_deref(),
            app_name.as_deref(),
            source_url.as_deref(),
        ),
        likely_purpose: infer_purpose(&content).to_string(),
        topic_tags: infer_tags(&content),
    })
}

fn session_from_row(row: &Row<'_>) -> rusqlite::Result<SessionSummary> {
    let started_at: i64 = row.get(7)?;
    let ended_at: i64 = row.get(8)?;
    let key_topics = json_strings(row.get::<_, Option<String>>(3)?.as_deref());
    let source_apps = json_strings(row.get::<_, Option<String>>(4)?.as_deref());
    let source_urls = json_strings(row.get::<_, Option<String>>(5)?.as_deref());
    let clip_count: i64 = row.get(6)?;
    let raw_label = row.get::<_, Option<String>>(1)?.unwrap_or_default();
    let label = if raw_label.is_empty() || raw_label == "New Session" {
        key_topics
            .first()
            .map(|topic| format!("{} research", title_case(topic)))
            .unwrap_or_else(|| "Research session".to_string())
    } else {
        raw_label
    };
    let raw_summary = row.get::<_, Option<String>>(2)?.unwrap_or_default();
    Ok(SessionSummary {
        id: row.get(0)?,
        label,
        summary: if raw_summary.is_empty() {
            format!("{clip_count} captured clips")
        } else {
            raw_summary
        },
        key_topics,
        source_apps,
        source_urls,
        clip_count,
        started_at,
        ended_at,
        duration_ms: (ended_at - started_at).max(0),
    })
}

fn session_clips(conn: &Connection, session_id: &str) -> Result<Vec<Clip>, String> {
    let mut statement = conn.prepare(
        "SELECT id, content, content_type, image_path, source_url, page_title, app_name, window_title,
                language, session_id, is_pinned, copied_at, ai_context, created_at
         FROM clips WHERE session_id = ?1 ORDER BY copied_at ASC",
    ).map_err(|error| error.to_string())?;
    let clips = statement
        .query_map([session_id], clip_from_row)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(clips)
}

fn clip_from_row(row: &Row<'_>) -> rusqlite::Result<Clip> {
    Ok(Clip {
        id: row.get(0)?,
        content: row.get(1)?,
        content_type: row.get(2)?,
        image_path: row.get(3)?,
        source_url: row.get(4)?,
        page_title: row.get(5)?,
        app_name: row.get(6)?,
        window_title: row.get(7)?,
        language: row.get(8)?,
        session_id: row.get(9)?,
        is_pinned: row.get::<_, i32>(10)? == 1,
        copied_at: row.get(11)?,
        ai_context: row.get(12)?,
        created_at: row.get(13)?,
    })
}

fn source_breakdown(clips: &[Clip]) -> Vec<SourceStat> {
    let mut sources = BTreeMap::<(String, String), i64>::new();
    for clip in clips {
        let source_url = clip
            .source_url
            .as_deref()
            .map(str::trim)
            .filter(|url| !url.is_empty());
        let app_name = clip
            .app_name
            .as_deref()
            .map(str::trim)
            .filter(|app| !app.is_empty() && !app.eq_ignore_ascii_case("unknown"));
        let (label, source_type) = if let Some(url) = source_url {
            (domain(url), "web".to_string())
        } else if let Some(app) = app_name {
            (app.to_string(), "app".to_string())
        } else {
            ("Source unavailable".to_string(), "unavailable".to_string())
        };
        *sources.entry((label, source_type)).or_default() += 1;
    }
    sources
        .into_iter()
        .map(|((label, source_type), count)| SourceStat {
            label,
            count,
            source_type,
        })
        .collect()
}

fn cross_session_connections(
    conn: &Connection,
    session_id: &str,
) -> Result<Vec<SessionConnection>, String> {
    let mut statement = conn.prepare(
        "SELECT other.id, other.content, e.similarity, other.copied_at
         FROM memory_edges e
         JOIN clips own ON own.id = CASE WHEN e.clip_a_id IN (SELECT id FROM clips WHERE session_id = ?1) THEN e.clip_a_id ELSE e.clip_b_id END
         JOIN clips other ON other.id = CASE WHEN own.id = e.clip_a_id THEN e.clip_b_id ELSE e.clip_a_id END
         WHERE own.session_id = ?1 AND (other.session_id IS NULL OR other.session_id != ?1)
         ORDER BY e.similarity DESC LIMIT 5",
    ).map_err(|error| error.to_string())?;
    let connections = statement
        .query_map([session_id], |row| {
            let content: String = row.get(1)?;
            Ok(SessionConnection {
                clip_id: row.get(0)?,
                content_preview: truncate(&content, 100),
                similarity: row.get(2)?,
                copied_at: row.get(3)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(connections)
}

fn json_strings(value: Option<&str>) -> Vec<String> {
    value
        .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
        .unwrap_or_default()
}

fn infer_tags(content: &str) -> Vec<String> {
    let lower = content.to_lowercase();
    let tags = [
        ("react", ["usestate", "useeffect", "react"].as_slice()),
        ("async", ["async", "await", "asyncio"].as_slice()),
        ("docker", ["docker", "compose", "kubernetes"].as_slice()),
        ("sql", ["select ", "insert ", " join "].as_slice()),
        ("auth", ["oauth", "jwt", "bearer"].as_slice()),
        ("python", ["def ", "import ", "class "].as_slice()),
    ];
    tags.into_iter()
        .filter_map(|(tag, matches)| {
            matches
                .iter()
                .any(|needle| lower.contains(needle))
                .then_some(tag.to_string())
        })
        .take(6)
        .collect()
}

fn infer_purpose(content: &str) -> &'static str {
    let tags = infer_tags(content);
    if tags.iter().any(|tag| tag == "auth") {
        "Implementing authentication flow"
    } else if tags.iter().any(|tag| tag == "docker") {
        "Configuring container infrastructure"
    } else if tags.iter().any(|tag| tag == "sql") {
        "Writing database queries"
    } else if tags.iter().any(|tag| tag == "react") {
        "Building React component logic"
    } else if tags.iter().any(|tag| tag == "async") {
        "Researching async patterns"
    } else {
        "General reference"
    }
}

fn source_label(
    page_title: Option<&str>,
    app_name: Option<&str>,
    source_url: Option<&str>,
) -> String {
    page_title
        .map(str::to_owned)
        .or_else(|| source_url.map(domain))
        .or_else(|| app_name.map(str::to_owned))
        .unwrap_or_else(|| "Unknown source".to_string())
}

fn domain(url: &str) -> String {
    url.split("//")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url)
        .trim_start_matches("www.")
        .to_string()
}
fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() > max {
        format!("{}...", value.chars().take(max).collect::<String>())
    } else {
        value.to_string()
    }
}
