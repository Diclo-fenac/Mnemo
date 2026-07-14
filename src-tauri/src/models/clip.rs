use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub content: String,
    pub content_type: String,
    pub image_path: Option<String>,
    pub source_url: Option<String>,
    pub page_title: Option<String>,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub language: Option<String>,
    pub session_id: Option<String>,
    pub is_pinned: bool,
    pub copied_at: i64,
    pub ai_context: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClipAddedPayload {
    pub clip_id: String,
    pub content_preview: String,
    pub content_type: String,
    pub app_name: Option<String>,
    pub copied_at: i64,
}
