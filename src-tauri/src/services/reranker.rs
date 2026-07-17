/// Reranking is deliberately deferred. The current default ONNX artifact is
/// over 1 GB, so downloading it during every first-run startup would delay
/// capture readiness and consume disproportionate disk space. Search uses its
/// deterministic hybrid ranking until an explicit opt-in reranker setting is
/// introduced.
pub fn start() {
    log::info!("[reranker] Deferred; hybrid ranking is active");
}
