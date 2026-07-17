use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fastembed::TextEmbedding;
use rusqlite::Connection;

use crate::state::EmbeddingStatus;

pub const CURRENT_MODEL: &str = "bge-small-en-v1.5";
pub const CURRENT_VERSION: &str = "1";

pub fn start_embedder(
    db: Arc<Mutex<Connection>>,
    embedder_state: Arc<Mutex<Option<TextEmbedding>>>,
    embedding_status: Arc<Mutex<EmbeddingStatus>>,
    model_cache_dir: PathBuf,
) {
    thread::spawn(move || {
        log::info!("[embedder] Loading embedding model...");
        if let Ok(mut status) = embedding_status.lock() {
            *status = EmbeddingStatus::Loading;
        }

        let active_model = db
            .lock()
            .ok()
            .and_then(|conn| {
                conn.query_row(
                    "SELECT model FROM embedding_registry WHERE slot = 'active'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .ok()
            })
            .unwrap_or_else(|| CURRENT_MODEL.to_string());
        let model = match crate::services::model_loader::load_text(&active_model, &model_cache_dir)
        {
            Ok(m) => m,
            Err(e) => {
                log::error!("[embedder] Failed to load model: {e}");
                if let Ok(mut status) = embedding_status.lock() {
                    *status = EmbeddingStatus::Unavailable;
                }
                return;
            }
        };

        log::info!("[embedder] Model loaded. Starting background processor.");

        {
            let mut state = embedder_state.lock().unwrap();
            *state = Some(model);
        }
        if let Ok(mut status) = embedding_status.lock() {
            *status = EmbeddingStatus::Ready;
        }

        loop {
            thread::sleep(Duration::from_secs(2));

            let un_embedded_clips: Vec<(String, String)> = {
                let conn = db.lock().unwrap();
                let mut stmt = match conn.prepare(
                    "SELECT c.id, c.content FROM clips c 
                     LEFT JOIN clips_embeddings ce ON c.id = ce.clip_id
                     WHERE ce.clip_id IS NULL
                       AND c.is_duplicate = 0
                       AND c.embedding_status IN ('pending', 'failed')
                     LIMIT 10",
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("[embedder] DB prepare error: {e}");
                        continue;
                    }
                };

                let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)));

                match rows {
                    Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
                    Err(_) => vec![],
                }
            };

            if un_embedded_clips.is_empty() {
                continue;
            }

            let active_model_name = {
                let conn = db.lock().unwrap();
                conn.query_row(
                    "SELECT model FROM embedding_registry WHERE slot = 'active'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap_or_else(|_| CURRENT_MODEL.to_string())
            };
            let texts: Vec<String> = un_embedded_clips
                .iter()
                .map(|(_, content)| {
                    let trimmed = if content.chars().count() > 8000 {
                        content.chars().take(8000).collect::<String>()
                    } else {
                        content.clone()
                    };
                    crate::services::model_registry::prepare_text(
                        &active_model_name,
                        &trimmed,
                        false,
                    )
                })
                .collect();
            let embeddings = {
                let state = embedder_state.lock().unwrap();
                if let Some(model) = state.as_ref() {
                    model.embed(texts, Some(32))
                } else {
                    Err(anyhow::anyhow!("Model not ready"))
                }
            };

            match embeddings {
                Ok(embeddings) => {
                    let conn = db.lock().unwrap();
                    let active_version: String = conn
                        .query_row(
                            "SELECT version FROM embedding_registry WHERE slot = 'active'",
                            [],
                            |row| row.get(0),
                        )
                        .unwrap_or_else(|_| CURRENT_VERSION.to_string());
                    for ((clip_id, _), embedding) in un_embedded_clips.iter().zip(embeddings.iter())
                    {
                        let exists: bool = conn
                            .query_row(
                                "SELECT EXISTS(SELECT 1 FROM clips WHERE id = ?1)",
                                [clip_id],
                                |row| row.get(0),
                            )
                            .unwrap_or(false);
                        if !exists {
                            continue;
                        }
                        let bytes: Vec<u8> = embedding
                            .iter()
                            .flat_map(|value| value.to_le_bytes())
                            .collect();
                        let res = conn.execute(
                            "INSERT OR REPLACE INTO clips_embeddings (clip_id, embedding) VALUES (?1, ?2)",
                            rusqlite::params![clip_id, bytes],
                        );
                        if res.is_ok() {
                            let _ = conn.execute(
                                "UPDATE clips SET embedding_model = ?1,
                                 embedding_version = ?2, embedding_status = 'embedded',
                                 embedding_id = ?3 WHERE id = ?3",
                                rusqlite::params![active_model_name, active_version, clip_id],
                            );
                            let _ = crate::services::memory_graph::process_new_clip(
                                &conn, clip_id, embedding,
                            );
                            if let Err(error) = crate::services::intelligence::refresh_state(&conn)
                            {
                                log::warn!("[embedder] Intelligence state update failed: {error}");
                            }
                        }
                    }
                }
                Err(error) => {
                    let conn = db.lock().unwrap();
                    for (clip_id, _) in un_embedded_clips {
                        let _ = conn.execute(
                            "UPDATE clips SET embedding_status = 'failed' WHERE id = ?1",
                            [clip_id],
                        );
                    }
                    log::error!("[embedder] Batch embedding failed: {error}");
                }
            }
        }
    });
}
