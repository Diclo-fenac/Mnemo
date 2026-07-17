#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub dimensions: usize,
    pub size_mb: usize,
    pub description: &'static str,
    pub recommended_for: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelArtifacts {
    pub repository: &'static str,
    pub model_file: &'static str,
}

pub const SUPPORTED_MODELS: &[ModelInfo] = &[
    ModelInfo {
        id: "bge-small-en-v1.5",
        display_name: "BGE Small (Fast)",
        dimensions: 384,
        size_mb: 130,
        description: "Default retrieval model with the best speed-quality balance.",
        recommended_for: "general",
    },
    ModelInfo {
        id: "bge-base-en-v1.5",
        display_name: "BGE Base (Quality)",
        dimensions: 768,
        size_mb: 420,
        description: "Higher recall with larger vectors and slower embedding.",
        recommended_for: "quality",
    },
    ModelInfo {
        id: "nomic-embed-text-v1.5",
        display_name: "Nomic v1.5 (Long context)",
        dimensions: 768,
        size_mb: 270,
        description: "Long-context model for larger copied documents.",
        recommended_for: "long_form",
    },
];

pub fn info(id: &str) -> Option<&'static ModelInfo> {
    SUPPORTED_MODELS.iter().find(|model| model.id == id)
}

pub fn artifacts(id: &str) -> Option<ModelArtifacts> {
    Some(match id {
        "bge-small-en-v1.5" => ModelArtifacts {
            repository: "Xenova/bge-small-en-v1.5",
            model_file: "onnx/model.onnx",
        },
        "bge-base-en-v1.5" => ModelArtifacts {
            repository: "Xenova/bge-base-en-v1.5",
            model_file: "onnx/model.onnx",
        },
        "nomic-embed-text-v1.5" => ModelArtifacts {
            repository: "nomic-ai/nomic-embed-text-v1.5",
            model_file: "onnx/model.onnx",
        },
        _ => return None,
    })
}

pub fn prepare_text(model_id: &str, text: &str, query: bool) -> String {
    if model_id == "nomic-embed-text-v1.5" {
        let prefix = if query {
            "search_query: "
        } else {
            "search_document: "
        };
        format!("{prefix}{text}")
    } else {
        text.to_string()
    }
}
