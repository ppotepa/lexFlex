use crate::canonical::canonicalize_document_resolution;
use crate::model::CanonicalDocumentResolutionArtifacts;
use lexflex::api::LexFlexAPI;
use lexflex::document::graph::DocumentGraph;
use lexflex::document::resolution::DocumentEntityResolution;

#[derive(Debug, Clone)]
pub struct ResolutionPipelineRun {
    pub resolution: Option<DocumentEntityResolution>,
    pub canonical: Option<CanonicalDocumentResolutionArtifacts>,
    pub deterministic: bool,
    pub error_categories: Vec<String>,
}

pub fn run_document_resolution_pipeline(
    api: &LexFlexAPI,
    graph: &DocumentGraph,
    repeat_count: usize,
) -> ResolutionPipelineRun {
    let mut error_categories = Vec::new();
    let mut runs = Vec::new();
    for _ in 0..repeat_count.max(1) {
        match api.resolve_document_graph(graph) {
            Ok(resolution) => runs.push(resolution),
            Err(error) => error_categories.push(error.to_string()),
        }
    }
    let deterministic = runs
        .windows(2)
        .all(|pair| pair[0].resolution_sha256 == pair[1].resolution_sha256 && pair[0] == pair[1]);
    let resolution = runs.first().cloned();
    let canonical = resolution
        .as_ref()
        .map(|resolution| canonicalize_document_resolution(resolution, deterministic));
    ResolutionPipelineRun {
        resolution,
        canonical,
        deterministic,
        error_categories,
    }
}
