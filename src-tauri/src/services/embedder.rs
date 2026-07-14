use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use rusqlite::Connection;
use uuid::Uuid;

pub fn start_embedder(db: Arc<Mutex<Connection>>, embedder_state: Arc<Mutex<Option<TextEmbedding>>>) {
    thread::spawn(move || {
        log::info!("[embedder] Loading embedding model...");

        let model = match TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::BGESmallENV15,
            show_download_progress: false,
            ..Default::default()
        }) {
            Ok(m) => m,
            Err(e) => {
                log::error!("[embedder] Failed to load model: {e}");
                return;
            }
        };

        log::info!("[embedder] Model loaded. Starting background processor.");
        
        {
            let mut state = embedder_state.lock().unwrap();
            *state = Some(model);
        }

        loop {
            thread::sleep(Duration::from_secs(2));

            let un_embedded_clips: Vec<(String, String)> = {
                let conn = db.lock().unwrap();
                let mut stmt = match conn.prepare(
                    "SELECT c.id, c.content FROM clips c 
                     LEFT JOIN clips_embeddings ce ON c.id = ce.clip_id
                     WHERE ce.clip_id IS NULL AND c.content_type != 'url'
                     LIMIT 10"
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("[embedder] DB prepare error: {e}");
                        continue;
                    }
                };

                let rows = stmt.query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                });

                match rows {
                    Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
                    Err(_) => vec![],
                }
            };

            if un_embedded_clips.is_empty() {
                continue;
            }

            for (clip_id, content) in un_embedded_clips {
                let content_trimmed = if content.len() > 8000 {
                    content[..8000].to_string()
                } else {
                    content.clone()
                };

                let embeddings = {
                    let state = embedder_state.lock().unwrap();
                    if let Some(model) = state.as_ref() {
                        model.embed(vec![content_trimmed], None)
                    } else {
                        Err(anyhow::anyhow!("Model not ready"))
                    }
                };

                match embeddings {
                    Ok(embeddings) => {
                        if let Some(embedding) = embeddings.first() {
                            let conn = db.lock().unwrap();
                            
                            // Convert Vec<f32> to a format sqlite-vec understands
                            // sqlite-vec handles &[f32] using bytes.
                            let mut bytes = Vec::with_capacity(embedding.len() * 4);
                            for &f in embedding {
                                bytes.extend_from_slice(&f.to_le_bytes());
                            }

                            let res = conn.execute(
                                "INSERT INTO clips_embeddings (clip_id, embedding) VALUES (?1, ?2)",
                                rusqlite::params![clip_id, bytes],
                            );

                            if let Err(e) = res {
                                log::error!("[embedder] Failed to insert embedding for {}: {}", clip_id, e);
                            } else {
                                log::info!("[embedder] Generated embedding for {}", clip_id);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[embedder] Fastembed error for {}: {}", clip_id, e);
                    }
                }
            }
        }
    });
}
