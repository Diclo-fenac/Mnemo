use crate::models::clip::Clip;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphData {
    pub nodes: Vec<Clip>,
    pub links: Vec<GraphLink>,
    pub state: String,
    pub unconnected_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    pub similarity: f64,
    pub edge_type: String,
    pub temporal_weight: f64,
}

#[tauri::command]
pub fn get_graph_data(state: State<'_, AppState>, limit: Option<i64>) -> Result<GraphData, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable")?;
    let limit = limit.unwrap_or(100).clamp(1, 200);

    // For Milestone 3, we'll fetch the last 100 clips as nodes.
    let mut stmt = conn
        .prepare(
            "SELECT id, content, content_type, image_path, source_url, page_title,
                    app_name, window_title, language, session_id, is_pinned,
                    copied_at, ai_context, created_at
             FROM clips
             ORDER BY copied_at DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("Query prepare failed: {e}"))?;

    let nodes: Vec<Clip> = stmt
        .query_map([limit], |row| {
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
        })
        .map_err(|e| format!("Query failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Node row decode failed: {error}"))?;

    let mut edge_stmt = conn
        .prepare(
            "SELECT clip_a_id, clip_b_id, similarity, edge_type, temporal_weight FROM memory_edges
             WHERE clip_a_id IN (SELECT id FROM clips ORDER BY copied_at DESC LIMIT ?1)
               AND clip_b_id IN (SELECT id FROM clips ORDER BY copied_at DESC LIMIT ?1)
             ORDER BY similarity DESC",
        )
        .map_err(|e| format!("Edge query failed: {e}"))?;
    let links = edge_stmt
        .query_map([limit], |row| {
            Ok(GraphLink {
                source: row.get(0)?,
                target: row.get(1)?,
                similarity: row.get(2)?,
                edge_type: row.get(3)?,
                temporal_weight: row.get(4)?,
            })
        })
        .map_err(|e| format!("Edge query failed: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Edge row decode failed: {error}"))?;

    let connected_ids: std::collections::HashSet<&str> = links
        .iter()
        .flat_map(|link| [link.source.as_str(), link.target.as_str()])
        .collect();
    let unconnected_count = nodes
        .iter()
        .filter(|node| !connected_ids.contains(node.id.as_str()))
        .count();
    let pending_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM clips
             WHERE is_duplicate = 0 AND embedding_status IN ('pending', 'failed')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let state = if links.is_empty() && pending_count > 0 {
        "building"
    } else if links.is_empty() {
        "edge_free"
    } else {
        "ready"
    };

    Ok(GraphData {
        nodes,
        links,
        state: state.to_string(),
        unconnected_count,
    })
}
