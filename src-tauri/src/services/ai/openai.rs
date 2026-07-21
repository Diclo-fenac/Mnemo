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
        "model": config.model,
        "store": false,
        "input": [{ "role": "user", "content": [{ "type": "input_text", "text": prompt }] }],
        "max_output_tokens": MAX_OUTPUT_TOKENS,
        "text": { "format": { "type": "json_object" } }
    });
    let key = config.api_key.as_deref().ok_or(AiError::NotConfigured)?;
    let response = request_agent()
        .post("https://api.openai.com/v1/responses")
        .set("Authorization", &format!("Bearer {key}"))
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|_| AiError::RequestFailed)?;
    let value: serde_json::Value = response.into_json().map_err(|_| AiError::RequestFailed)?;
    let raw = value
        .get("output_text")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned)
        .or_else(|| find_text(&value))
        .ok_or(AiError::InvalidAnswer)?;
    parse_answer(
        &raw,
        &evidence
            .iter()
            .map(|item| item.id.clone())
            .collect::<HashSet<_>>(),
        "openai",
    )
}

pub fn test(config: &ProviderConfig) -> Result<(), AiError> {
    let key = config.api_key.as_deref().ok_or(AiError::NotConfigured)?;
    let response = request_agent()
        .get("https://api.openai.com/v1/models")
        .set("Authorization", &format!("Bearer {key}"))
        .call()
        .map_err(|_| AiError::RequestFailed)?;
    if response.status() >= 400 {
        return Err(AiError::RequestFailed);
    }
    Ok(())
}

fn find_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => map.iter().find_map(|(key, value)| {
            if key == "text" {
                value.as_str().map(ToOwned::to_owned)
            } else {
                find_text(value)
            }
        }),
        serde_json::Value::Array(values) => values.iter().find_map(find_text),
        _ => None,
    }
}
