use tauri::State;
use serde::Serialize;
use crate::models::clip::Clip;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphData {
    pub nodes: Vec<Clip>,
    pub links: Vec<GraphLink>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    pub similarity: f64,
}

#[tauri::command]
pub fn get_graph_data(state: State<'_, AppState>) -> Result<GraphData, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable")?;

    // For Milestone 3, we'll fetch the last 100 clips as nodes.
    let mut stmt = conn
        .prepare(
            "SELECT id, content, content_type, image_path, source_url, page_title,
                    app_name, window_title, language, session_id, is_pinned,
                    copied_at, ai_context, created_at
             FROM clips
             ORDER BY copied_at DESC
             LIMIT 100",
        )
        .map_err(|e| format!("Query prepare failed: {e}"))?;

    let nodes: Vec<Clip> = stmt
        .query_map([], |row| {
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
        .filter_map(|r| r.ok())
        .collect();

    // Since O(N^2) similarity is expensive, we can use the `memory_edges` table 
    // or just generate dummy links for the canvas visualization to prove it works.
    let mut links = Vec::new();
    
    // Minimal mock linking for testing the d3-force graph
    for i in 0..nodes.len() {
        if i > 0 {
            // Link to the previous node randomly to create a connected graph
            links.push(GraphLink {
                source: nodes[i].id.clone(),
                target: nodes[i - 1].id.clone(),
                similarity: 0.8,
            });
        }
    }

    Ok(GraphData { nodes, links })
}
