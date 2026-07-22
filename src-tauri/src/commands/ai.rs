use std::collections::HashSet;

use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;
use tauri::State;

use crate::{
    services::{ai, filter},
    state::AppState,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsInput {
    pub provider: String,
    pub model: String,
    pub ollama_url: String,
    pub api_key: Option<String>,
    pub clear_api_key: bool,
    pub cloud_consent: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundedAnswerResponse {
    pub answer: String,
    pub citations: Vec<String>,
    pub confidence: String,
    pub source: String,
    pub fallback_reason: Option<String>,
}

#[tauri::command]
pub fn get_ai_settings(state: State<'_, AppState>) -> Result<ai::AiSettings, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "Database unavailable".to_string())?;
    read_settings(&conn)
}

#[tauri::command]
pub fn update_ai_settings(
    state: State<'_, AppState>,
    input: AiSettingsInput,
) -> Result<ai::AiSettings, String> {
    let provider = ai::Provider::parse(&input.provider)
        .ok_or_else(|| "Unsupported AI provider".to_string())?;
    let model = if input.model.trim().is_empty() {
        default_model(provider).to_string()
    } else {
        input.model.trim().to_string()
    };
    if model.chars().count() > 120 {
        return Err("Model name is too long".to_string());
    }
    if input
        .api_key
        .as_deref()
        .is_some_and(|key| key.chars().count() > 512)
    {
        return Err("API key is too long".to_string());
    }
    if provider != ai::Provider::None
        && matches!(provider, ai::Provider::Openai | ai::Provider::Gemini)
        && !input.cloud_consent
    {
        return Err("Cloud consent is required before enabling this provider".to_string());
    }
    let url = input.ollama_url.trim().trim_end_matches('/');
    if provider == ai::Provider::Ollama && !is_loopback_url(url) {
        return Err("Ollama URL must use localhost, 127.0.0.1, or ::1".to_string());
    }
    let conn = state
        .db
        .lock()
        .map_err(|_| "Database unavailable".to_string())?;
    let existing_key: Option<String> = conn
        .query_row("SELECT ai_api_key FROM settings WHERE id = 1", [], |row| {
            row.get(0)
        })
        .unwrap_or(None);
    let key = if input.clear_api_key {
        None
    } else {
        input
            .api_key
            .map(|key| key.trim().to_string())
            .filter(|key| !key.is_empty())
            .or(existing_key)
    };
    if matches!(provider, ai::Provider::Openai | ai::Provider::Gemini) && key.is_none() {
        return Err("An API key is required for this provider".to_string());
    }
    conn.execute("UPDATE settings SET ai_provider = ?1, ai_model = ?2, ai_api_key = ?3, ai_ollama_url = ?4, ai_cloud_consent = ?5 WHERE id = 1", rusqlite::params![provider.as_str(), model, key, if url.is_empty() { "http://localhost:11434" } else { url }, i32::from(input.cloud_consent)]).map_err(|error| error.to_string())?;
    read_settings(&conn)
}

#[tauri::command]
pub fn test_ai_provider(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "Database unavailable".to_string())?;
    let config = read_config(&conn)?;
    drop(conn);
    if matches!(config.provider, ai::Provider::Openai | ai::Provider::Gemini)
        && !config.cloud_consent
    {
        return Err("Cloud consent is required before testing this provider".to_string());
    }
    match config.provider {
        ai::Provider::None => Ok(true),
        ai::Provider::Ollama => crate::services::ai::ollama::test(&config)
            .map(|_| true)
            .map_err(|error| error.to_string()),
        ai::Provider::Openai => crate::services::ai::openai::test(&config)
            .map(|_| true)
            .map_err(|error| error.to_string()),
        ai::Provider::Gemini => crate::services::ai::gemini::test(&config)
            .map(|_| true)
            .map_err(|error| error.to_string()),
    }
}

#[tauri::command]
pub fn generate_grounded_answer(
    state: State<'_, AppState>,
    query: String,
    clip_ids: Vec<String>,
    local_only: Option<bool>,
    allow_cloud: Option<bool>,
    allow_local_fallback: Option<bool>,
) -> Result<GroundedAnswerResponse, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "Database unavailable".to_string())?;
    let config = read_config(&conn)?;
    let evidence = load_evidence(&conn, &clip_ids)?;
    let strict_provider = !allow_local_fallback.unwrap_or(true);
    let local = local_only.unwrap_or(false);
    if local {
        return Ok(ai::local_answer(&query, &evidence, None).into());
    }
    if config.provider == ai::Provider::None {
        if strict_provider {
            return Err("No AI answer provider is configured. Select an available provider in Settings.".to_string());
        }
        return Ok(ai::local_answer(&query, &evidence, None).into());
    }
    let cloud_provider = matches!(config.provider, ai::Provider::Openai | ai::Provider::Gemini);
    if cloud_provider && (!config.cloud_consent || !allow_cloud.unwrap_or(false)) {
        if strict_provider {
            return Err(if !config.cloud_consent {
                "Cloud consent is required before using the configured AI provider.".to_string()
            } else {
                "Cloud answer permission is required before using the configured AI provider.".to_string()
            });
        }
        return Ok(ai::local_answer(
            &query,
            &evidence,
            Some(if !config.cloud_consent {
                "Cloud consent was not granted".to_string()
            } else {
                "Cloud answer requires confirmation for this search".to_string()
            }),
        )
        .into());
    }
    let _request_guard =
        ai::RequestGuard::try_start(&state.ai_request_active).map_err(|error| error.to_string())?;
    drop(conn);
    let result = match config.provider {
        ai::Provider::Ollama => ai::ollama::generate(&config, &query, &evidence),
        ai::Provider::Openai => ai::openai::generate(&config, &query, &evidence),
        ai::Provider::Gemini => ai::gemini::generate(&config, &query, &evidence),
        ai::Provider::None => unreachable!(),
    };
    match result {
        Ok(answer) => Ok(answer.into()),
        Err(error) if strict_provider => Err(error.to_string()),
        Err(error) => Ok(ai::local_answer(&query, &evidence, Some(error.to_string())).into()),
    }
}

impl From<ai::GroundedAnswer> for GroundedAnswerResponse {
    fn from(answer: ai::GroundedAnswer) -> Self {
        Self {
            answer: answer.answer,
            citations: answer.citations,
            confidence: answer.confidence,
            source: answer.source,
            fallback_reason: answer.fallback_reason,
        }
    }
}

fn read_settings(conn: &Connection) -> Result<ai::AiSettings, String> {
    conn.query_row("SELECT ai_provider, ai_model, ai_ollama_url, ai_api_key, ai_cloud_consent FROM settings WHERE id = 1", [], |row| Ok(ai::AiSettings { provider: row.get(0)?, model: row.get(1)?, ollama_url: row.get(2)?, has_api_key: row.get::<_, Option<String>>(3)?.is_some_and(|key| !key.is_empty()), cloud_consent: row.get::<_, i32>(4)? != 0 })).map_err(|error| error.to_string())
}

fn read_config(conn: &Connection) -> Result<ai::ProviderConfig, String> {
    conn.query_row("SELECT ai_provider, ai_model, ai_api_key, ai_ollama_url, ai_cloud_consent FROM settings WHERE id = 1", [], |row| Ok(ai::ProviderConfig { provider: ai::Provider::parse(&row.get::<_, String>(0)?).unwrap_or(ai::Provider::None), model: row.get(1)?, api_key: row.get(2)?, ollama_url: row.get(3)?, cloud_consent: row.get::<_, i32>(4)? != 0 })).map_err(|error| error.to_string())
}

fn load_evidence(conn: &Connection, ids: &[String]) -> Result<Vec<ai::EvidenceClip>, String> {
    let rules = filter::load_rules(conn);
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for id in ids.iter().take(ai::MAX_CLIPS) {
        if !seen.insert(id) {
            continue;
        }
        let row = conn.query_row("SELECT content, page_title, app_name, source_url, copied_at, ai_context, is_sensitive FROM clips WHERE id = ?1", [id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, i64>(4)?, row.get::<_, Option<String>>(5)?, row.get::<_, i32>(6)?))).optional().map_err(|error| error.to_string())?;
        let Some((content, page_title, app_name, source_url, copied_at, context, is_sensitive)) =
            row
        else {
            continue;
        };
        if is_sensitive != 0 || filter::evaluate(&content, &rules).blocked {
            continue;
        }
        let topics = context
            .as_deref()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
            .and_then(|value| {
                value
                    .get("topic_tags")
                    .or_else(|| value.get("topicTags"))
                    .and_then(|tags| tags.as_array())
                    .map(|tags| {
                        tags.iter()
                            .filter_map(|tag| tag.as_str().map(ToOwned::to_owned))
                            .take(6)
                            .collect()
                    })
            })
            .unwrap_or_default();
        result.push(ai::EvidenceClip {
            id: id.clone(),
            content,
            source: source_url
                .or(page_title)
                .or(app_name)
                .unwrap_or_else(|| "Source unavailable".to_string()),
            copied_at,
            topics,
        });
    }
    Ok(result)
}

fn default_model(provider: ai::Provider) -> &'static str {
    match provider {
        ai::Provider::Ollama => "llama3.2:3b",
        ai::Provider::Openai => "gpt-4o-mini",
        ai::Provider::Gemini => "gemini-2.0-flash",
        ai::Provider::None => "llama3.2:3b",
    }
}
fn is_loopback_url(url: &str) -> bool {
    url.starts_with("http://localhost")
        || url.starts_with("http://127.0.0.1")
        || url.starts_with("http://[::1]")
        || url.starts_with("https://localhost")
        || url.starts_with("https://127.0.0.1")
        || url.starts_with("https://[::1]")
}
