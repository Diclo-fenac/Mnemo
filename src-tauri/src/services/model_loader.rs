use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use fastembed::{InitOptionsUserDefined, TextEmbedding, TokenizerFiles, UserDefinedEmbeddingModel};

use crate::services::model_registry;

const MAX_REDIRECTS: usize = 8;

/// Load a model from Mnemo's cache. Downloads use an explicit redirect loop
/// because the hf-hub client bundled with fastembed 3 resolves relative 307
/// locations incorrectly on the current Hugging Face endpoint.
pub fn load_text(model_id: &str, cache_dir: &Path) -> Result<TextEmbedding> {
    let artifacts = model_registry::artifacts(model_id)
        .ok_or_else(|| anyhow!("Unsupported embedding model: {model_id}"))?;
    let root = cache_dir.join("artifacts").join(model_id);

    let onnx_file = ensure_file(cache_dir, artifacts.repository, artifacts.model_file, &root)?;
    let tokenizer_files = TokenizerFiles {
        tokenizer_file: fs::read(ensure_file(
            cache_dir,
            artifacts.repository,
            "tokenizer.json",
            &root,
        )?)
        .context("Read tokenizer.json")?,
        config_file: fs::read(ensure_file(
            cache_dir,
            artifacts.repository,
            "config.json",
            &root,
        )?)
        .context("Read config.json")?,
        special_tokens_map_file: fs::read(ensure_file(
            cache_dir,
            artifacts.repository,
            "special_tokens_map.json",
            &root,
        )?)
        .context("Read special_tokens_map.json")?,
        tokenizer_config_file: fs::read(ensure_file(
            cache_dir,
            artifacts.repository,
            "tokenizer_config.json",
            &root,
        )?)
        .context("Read tokenizer_config.json")?,
    };

    TextEmbedding::try_new_from_user_defined(
        UserDefinedEmbeddingModel {
            onnx_file: fs::read(onnx_file).context("Read ONNX model")?,
            tokenizer_files,
        },
        InitOptionsUserDefined::default(),
    )
    .context("Initialize local embedding model")
}

pub fn is_cached(model_id: &str, cache_dir: &Path) -> bool {
    let Some(artifacts) = model_registry::artifacts(model_id) else {
        return false;
    };
    let root = cache_dir.join("artifacts").join(model_id);
    [
        artifacts.model_file,
        "tokenizer.json",
        "config.json",
        "special_tokens_map.json",
        "tokenizer_config.json",
    ]
    .iter()
    .all(|remote_path| {
        let target = root.join(remote_path);
        (target.is_file()
            && target
                .metadata()
                .map(|meta| meta.len() > 0)
                .unwrap_or(false))
            || legacy_cached_file(cache_dir, artifacts.repository, remote_path).is_some()
    })
}

fn ensure_file(
    cache_dir: &Path,
    repository: &str,
    remote_path: &str,
    root: &Path,
) -> Result<PathBuf> {
    let target = root.join(remote_path);
    if target.is_file() && target.metadata()?.len() > 0 {
        return Ok(target);
    }

    let parent = target
        .parent()
        .ok_or_else(|| anyhow!("Invalid model destination"))?;
    fs::create_dir_all(parent).context("Create model cache directory")?;
    if let Some(legacy) = legacy_cached_file(cache_dir, repository, remote_path) {
        let legacy_source = legacy.canonicalize().unwrap_or(legacy);
        match fs::hard_link(&legacy_source, &target) {
            Ok(()) => {
                log::info!("[model] Reused cached {repository}/{remote_path}");
                return Ok(target);
            }
            Err(error) => {
                log::debug!("[model] Could not hard-link legacy cache for {remote_path}: {error}")
            }
        }
    }
    let partial = target.with_extension("part");
    if partial.exists() {
        fs::remove_file(&partial).context("Remove incomplete model download")?;
    }

    let url = format!("https://huggingface.co/{repository}/resolve/main/{remote_path}");
    log::info!("[model] Downloading {repository}/{remote_path}");
    let response = get_following_redirects(&url)?;
    let mut file = File::create(&partial).context("Create model download")?;
    io::copy(&mut response.into_reader(), &mut file).context("Write model download")?;
    file.sync_all().context("Finalize model download")?;
    if partial.metadata()?.len() == 0 {
        bail!("Downloaded model file is empty: {remote_path}");
    }
    fs::rename(&partial, &target).context("Finalize model cache entry")?;
    Ok(target)
}

fn legacy_cached_file(cache_dir: &Path, repository: &str, remote_path: &str) -> Option<PathBuf> {
    let repo_dir = cache_dir.join(format!("models--{}", repository.replace('/', "--")));
    let revision = fs::read_to_string(repo_dir.join("refs/main")).ok()?;
    let file = repo_dir
        .join("snapshots")
        .join(revision.trim())
        .join(remote_path);
    file.is_file().then_some(file)
}

fn get_following_redirects(initial_url: &str) -> Result<ureq::Response> {
    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let mut url = initial_url.to_string();

    for _ in 0..MAX_REDIRECTS {
        match agent.get(&url).set("User-Agent", "Mnemo/0.1").call() {
            Ok(response) if (300..400).contains(&response.status()) => {
                url = next_redirect_url(&url, &response)?;
            }
            Ok(response) => return Ok(response),
            Err(ureq::Error::Status(status, response)) if (300..400).contains(&status) => {
                url = next_redirect_url(&url, &response)?;
            }
            Err(error) => return Err(anyhow!("Model download request failed: {error}")),
        }
    }

    bail!("Model download exceeded {MAX_REDIRECTS} redirects")
}

fn next_redirect_url(current_url: &str, response: &ureq::Response) -> Result<String> {
    let location = response
        .header("Location")
        .ok_or_else(|| anyhow!("Model download redirect has no Location header"))?;
    resolve_redirect(current_url, location)
}

fn resolve_redirect(current_url: &str, location: &str) -> Result<String> {
    if location.starts_with("https://") || location.starts_with("http://") {
        return Ok(location.to_string());
    }
    if location.starts_with('/') {
        return Ok(format!("https://huggingface.co{location}"));
    }
    let parent = current_url
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .ok_or_else(|| anyhow!("Invalid redirect base URL"))?;
    Ok(format!("{parent}/{location}"))
}

#[cfg(test)]
mod tests {
    use super::{get_following_redirects, legacy_cached_file, resolve_redirect};
    use std::fs;

    #[test]
    fn resolves_hugging_face_relative_redirects() {
        assert_eq!(
            resolve_redirect(
                "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/tokenizer.json",
                "/api/resolve-cache/models/Xenova/bge-small-en-v1.5/revision/tokenizer.json",
            )
            .unwrap(),
            "https://huggingface.co/api/resolve-cache/models/Xenova/bge-small-en-v1.5/revision/tokenizer.json",
        );
    }

    #[test]
    fn finds_legacy_fastembed_cache_entries() {
        let root = std::env::temp_dir().join(format!("mnemo-model-test-{}", std::process::id()));
        let repo = root.join("models--Xenova--bge-small-en-v1.5");
        fs::create_dir_all(repo.join("snapshots/revision/onnx")).unwrap();
        fs::create_dir_all(repo.join("refs")).unwrap();
        fs::write(repo.join("refs/main"), "revision").unwrap();
        fs::write(repo.join("snapshots/revision/onnx/model.onnx"), [1_u8]).unwrap();

        assert_eq!(
            legacy_cached_file(&root, "Xenova/bge-small-en-v1.5", "onnx/model.onnx"),
            Some(repo.join("snapshots/revision/onnx/model.onnx")),
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "contacts Hugging Face to verify redirect compatibility"]
    fn follows_the_current_hugging_face_relative_redirect() {
        let response = get_following_redirects(
            "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/tokenizer.json",
        )
        .unwrap();
        assert_eq!(response.status(), 200);
    }
}
