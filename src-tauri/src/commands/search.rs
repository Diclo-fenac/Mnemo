use std::collections::HashMap;

use fastembed::TextEmbedding;
use serde::Serialize;
use tauri::State;

use crate::models::clip::Clip;
use crate::services::source_intent::SourceIntent;
use crate::state::AppState;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MatchReason {
    pub reason_type: String,
    pub label: String,
    pub weight: f64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub clip: Clip,
    pub duplicate_count: i64,
    pub search_type: String,
    pub score: f64,
    pub match_reasons: Vec<MatchReason>,
}

#[derive(Default)]
struct Candidate {
    semantic: f64,
    exact: f64,
    rank: usize,
}

#[tauri::command]
pub fn hybrid_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let query = query.trim().to_string();
    if query.len() < 3 {
        return recent_results(&state);
    }

    let code_query = is_code_query(&query);
    let normalized_query = preprocess_query(&query);
    let mut candidates: HashMap<String, Candidate> = HashMap::new();

    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    let table: String = conn
        .query_row(
            "SELECT table_name FROM embedding_registry WHERE slot = 'active'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "clips_embeddings".to_string());
    if !valid_identifier(&table) {
        return Err("Invalid active vector table".to_string());
    }

    let fts_query = if code_query {
        quote_fts(&normalized_query)
    } else {
        normalized_query
            .split_whitespace()
            .filter(|word| word.len() >= 2)
            .map(|word| format!("\"{}\"", word.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" OR ")
    };
    let fts_table = if code_query {
        "clips_fts_code"
    } else {
        "clips_fts"
    };
    if !fts_query.is_empty() {
        let sql = format!("SELECT rowid FROM {fts_table} WHERE {fts_table} MATCH ?1 LIMIT 20");
        if let Ok(mut stmt) = conn.prepare(&sql) {
            if let Ok(rows) = stmt.query_map([fts_query], |row| row.get::<_, i64>(0)) {
                for (rank, row) in rows.flatten().enumerate() {
                    if let Ok(id) = conn.query_row(
                        "SELECT id FROM clips WHERE rowid = ?1 AND is_duplicate = 0",
                        [row],
                        |value| value.get::<_, String>(0),
                    ) {
                        candidates.entry(id).or_default().exact = 1.0 / (rank as f64 + 1.0);
                    }
                }
            }
        }
    }

    let semantic_candidates = if let Ok(embedder) = state.embedder.lock() {
        if let Some(model) = embedder.as_ref() {
            Some(vector_candidates(model, &conn, &table, &normalized_query)?)
        } else {
            None
        }
    } else {
        None
    };
    if let Some(rows) = semantic_candidates {
        for (rank, (id, distance)) in rows.into_iter().enumerate() {
            let candidate = candidates.entry(id).or_default();
            candidate.semantic = (1.0 - distance).clamp(0.0, 1.0);
            candidate.rank = rank;
        }
    }

    let now = chrono::Utc::now().timestamp_millis();
    let mut results = Vec::new();
    for (id, candidate) in candidates {
        let clip = match read_clip(&conn, &id) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let recency = recency_score(now, clip.copied_at);
        let score = 0.55 * candidate.semantic + 0.35 * candidate.exact + 0.10 * recency;
        let mut reasons = Vec::new();
        if candidate.semantic > 0.0 {
            reasons.push(MatchReason {
                reason_type: "semantic_similarity".into(),
                label: format!("Semantic similarity ({:.0}%)", candidate.semantic * 100.0),
                weight: 0.55 * candidate.semantic,
            });
        }
        if candidate.exact > 0.0 {
            reasons.push(MatchReason {
                reason_type: if code_query {
                    "fts_substring"
                } else {
                    "fts_exact"
                }
                .into(),
                label: if code_query {
                    "Contains exact code substring".into()
                } else {
                    "Keyword match in content".into()
                },
                weight: 0.35 * candidate.exact,
            });
        }
        if recency > 0.0 {
            reasons.push(MatchReason {
                reason_type: "recency".into(),
                label: "Recently copied".into(),
                weight: 0.10 * recency,
            });
        }
        let search_type = match (candidate.semantic > 0.0, candidate.exact > 0.0) {
            (true, true) => "hybrid",
            (true, false) => "semantic",
            _ => "keyword",
        };
        results.push(SearchResult {
            clip,
            duplicate_count: duplicate_count(&conn, &id),
            search_type: search_type.into(),
            score,
            match_reasons: reasons,
        });
    }
    results.sort_by(|a, b| b.score.total_cmp(&a.score));
    results.truncate(20);
    if results.len() > 1 {
        let top_count = results.len().min(5);
        let documents: Vec<String> = results[..top_count]
            .iter()
            .map(|result| result.clip.content.clone())
            .collect();
        if let Ok(mut reranker) = state.reranker.lock() {
            if let Some(model) = reranker.as_mut() {
                let document_refs = documents.iter().collect();
                if let Ok(ranked) = model.rerank(&query, document_refs, false, Some(32)) {
                    let top = results.drain(..top_count).collect::<Vec<_>>();
                    let mut reordered = Vec::with_capacity(top.len());
                    let mut seen = std::collections::HashSet::new();
                    for item in ranked {
                        if let Some(result) = top.get(item.index) {
                            seen.insert(result.clip.id.clone());
                            reordered.push(result.clone());
                        }
                    }
                    reordered.extend(
                        top.into_iter()
                            .filter(|result| !seen.contains(&result.clip.id)),
                    );
                    reordered.extend(results);
                    results = reordered;
                }
            }
        }
    }
    let query_type = if code_query { "keyword_code" } else { "hybrid" };
    if results.is_empty() {
        let _ = crate::services::search_feedback::log(
            &conn,
            &query,
            query_type,
            None,
            None,
            "no_results",
        );
    } else {
        for (rank, result) in results.iter().enumerate() {
            let _ = crate::services::search_feedback::log(
                &conn,
                &query,
                query_type,
                Some(&result.clip.id),
                Some((rank + 1) as i64),
                "impression",
            );
        }
    }
    Ok(results)
}

fn vector_candidates(
    model: &TextEmbedding,
    conn: &rusqlite::Connection,
    table: &str,
    query: &str,
) -> Result<Vec<(String, f64)>, String> {
    let query_embedding = model
        .embed(
            vec![crate::services::model_registry::prepare_text(
                &conn
                    .query_row(
                        "SELECT model FROM embedding_registry WHERE slot = 'active'",
                        [],
                        |row| row.get::<_, String>(0),
                    )
                    .unwrap_or_else(|_| "bge-small-en-v1.5".to_string()),
                query,
                true,
            )],
            Some(1),
        )
        .map_err(|e| format!("Embedding failed: {e}"))?
        .into_iter()
        .next()
        .ok_or_else(|| "No query embedding generated".to_string())?;
    let bytes: Vec<u8> = query_embedding
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    let sql = format!(
        "SELECT clip_id, distance FROM {table}
         WHERE embedding MATCH ?1 AND k = 20 ORDER BY distance"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![bytes], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.flatten().collect())
}

fn recent_results(state: &State<'_, AppState>) -> Result<Vec<SearchResult>, String> {
    let conn = state.db.lock().map_err(|_| "DB unavailable".to_string())?;
    let mut stmt = conn
        .prepare("SELECT id FROM clips ORDER BY copied_at DESC LIMIT 20")
        .map_err(|e| e.to_string())?;
    let ids: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    ids.into_iter()
        .filter_map(|id| read_clip(&conn, &id).ok())
        .map(|clip| SearchResult {
            duplicate_count: duplicate_count(&conn, &clip.id),
            clip,
            search_type: "recent".into(),
            score: 1.0,
            match_reasons: vec![MatchReason {
                reason_type: "recent".into(),
                label: "Recent clip".into(),
                weight: 1.0,
            }],
        })
        .collect::<Vec<_>>()
        .pipe(Ok)
}

fn duplicate_count(conn: &rusqlite::Connection, original_id: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM clips WHERE duplicate_of = ?1",
        [original_id],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

fn read_clip(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<Clip> {
    conn.query_row(
        "SELECT id, content, content_type, image_path, source_url, page_title,
                app_name, window_title, language, session_id, is_pinned,
                copied_at, ai_context, created_at
         FROM clips WHERE id = ?1",
        [id],
        |row| {
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
        },
    )
}

fn preprocess_query(query: &str) -> String {
    let aliases = [("k8s", "kubernetes"), ("react hooks", "useState useEffect")];
    let mut output = query.to_lowercase();
    for (from, to) in aliases {
        output = output.replace(from, to);
    }
    ["find", "that", "the", "i copied", "show me", "please"]
        .iter()
        .fold(output, |value, filler| value.replace(filler, " "))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_code_query(query: &str) -> bool {
    let symbols = query
        .chars()
        .filter(|c| "{}()[];=><:!".contains(*c))
        .count();
    let keywords = [
        "def ", "fn ", "func ", "class ", "import ", "require", "const ",
    ];
    symbols >= 2
        || keywords.iter().any(|keyword| query.contains(keyword))
        || (!query.contains(' ') && query.len() >= 3)
}

fn quote_fts(query: &str) -> String {
    format!("\"{}\"", query.replace('"', "\"\""))
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn recency_score(now: i64, copied_at: i64) -> f64 {
    let age_hours = ((now - copied_at).max(0) as f64) / 3_600_000.0;
    (-age_hours / 48.0).exp()
}

trait Pipe: Sized {
    fn pipe<T>(self, function: impl FnOnce(Self) -> T) -> T {
        function(self)
    }
}
impl<T> Pipe for T {}

#[allow(dead_code)]
fn _source_weight(intent: &str) -> f64 {
    match intent {
        "docs" => SourceIntent::Docs.trust_weight(),
        "github" => SourceIntent::Github.trust_weight(),
        "stackoverflow" => SourceIntent::StackOverflow.trust_weight(),
        "editor" => SourceIntent::Editor.trust_weight(),
        "terminal" => SourceIntent::Terminal.trust_weight(),
        _ => SourceIntent::Other.trust_weight(),
    }
}
