use tauri::State;
use crate::models::clip::Clip;
use crate::state::AppState;
use fastembed::TextEmbedding;

#[tauri::command]
pub fn hybrid_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Clip>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    // 1. Get embedding for the query
    let query_embedding = {
        let embedder_state = state.embedder.lock().map_err(|_| "Embedder lock failed")?;
        if let Some(model) = embedder_state.as_ref() {
            let embeddings = model
                .embed(vec![query.clone()], None)
                .map_err(|e| format!("Embedding failed: {}", e))?;
            embeddings.into_iter().next().ok_or("No embedding generated")?
        } else {
            return Err("Model not ready".to_string());
        }
    };

    let mut query_bytes = Vec::with_capacity(query_embedding.len() * 4);
    for &f in &query_embedding {
        query_bytes.extend_from_slice(&f.to_le_bytes());
    }

    let conn = state.db.lock().map_err(|_| "DB unavailable")?;

    // We do a reciprocal rank fusion or just a simple union.
    // For simplicity, we just do a FTS5 search AND a vector search, then combine.
    // sqlite-vec allows vector search. FTS5 allows text search.

    let mut stmt = conn.prepare(
        "
        WITH fts_matches AS (
            SELECT rowid, rank as fts_rank 
            FROM clips_fts 
            WHERE clips_fts MATCH ?1 
            LIMIT 50
        ),
        vec_matches AS (
            SELECT clip_id, vec_distance_cosine(embedding, ?2) as vec_dist
            FROM clips_embeddings 
            WHERE embedding MATCH ?2 AND k = 50
        )
        SELECT c.id, c.content, c.content_type, c.image_path, c.source_url, 
               c.page_title, c.app_name, c.window_title, c.language, c.session_id, 
               c.is_pinned, c.copied_at, c.ai_context, c.created_at,
               COALESCE(v.vec_dist, 1.0) as distance
        FROM clips c
        LEFT JOIN vec_matches v ON c.id = v.clip_id
        LEFT JOIN fts_matches f ON c.rowid = f.rowid
        WHERE v.clip_id IS NOT NULL OR f.rowid IS NOT NULL
        ORDER BY distance ASC
        LIMIT 50
        "
    ).map_err(|e| format!("Prepare failed: {e}"))?;

    let clips = stmt
        .query_map(rusqlite::params![query, query_bytes], |row| {
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

    Ok(clips)
}
