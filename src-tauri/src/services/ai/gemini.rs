use super::{
    compact_prompt, parse_answer, request_agent, AiError, EvidenceClip, GroundedAnswer,
    ProviderConfig, MAX_OUTPUT_TOKENS,
};
use serde_json::json;
use std::collections::HashSet;

pub fn generate(
    config: &ProviderConfig,
    query: &str,
    evidence: &[EvidenceClip],
) -> Result<GroundedAnswer, AiError> {
    let prompt = compact_prompt(query, evidence)?;
    let body = json!({
        "contents": [{ "parts": [{ "text": prompt }] }],
        "generationConfig": { "responseMimeType": "application/json", "maxOutputTokens": MAX_OUTPUT_TOKENS }
    });
    let key = config.api_key.as_deref().ok_or(AiError::NotConfigured)?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        config.model
    );
    let response = request_agent()
        .post(&url)
        .set("x-goog-api-key", key)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|_| AiError::RequestFailed)?;
    let value: serde_json::Value = response.into_json().map_err(|_| AiError::RequestFailed)?;
    let raw = value
        .get("candidates")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("content"))
        .and_then(|v| v.get("parts"))
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("text"))
        .and_then(|v| v.as_str())
        .ok_or(AiError::InvalidAnswer)?;
    parse_answer(
        raw,
        &evidence
            .iter()
            .map(|item| item.id.clone())
            .collect::<HashSet<_>>(),
        "gemini",
    )
}

pub fn test(config: &ProviderConfig) -> Result<(), AiError> {
    let key = config.api_key.as_deref().ok_or(AiError::NotConfigured)?;
    let response = request_agent()
        .get(&format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}",
            config.model
        ))
        .set("x-goog-api-key", key)
        .call()
        .map_err(|_| AiError::RequestFailed)?;
    if response.status() >= 400 {
        return Err(AiError::RequestFailed);
    }
    Ok(())
}
