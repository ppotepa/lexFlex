use super::model::DocumentKnowledgeExtraction;

pub fn document_knowledge_hash(artifact: &DocumentKnowledgeExtraction) -> Result<String, serde_json::Error> {
    let value = serde_json::json!({
        "schema": artifact.schema,
        "source_graph_id": artifact.source_graph_id,
        "source_graph_sha256": artifact.source_graph_sha256,
        "source_resolution_id": artifact.source_resolution_id,
        "source_resolution_sha256": artifact.source_resolution_sha256,
        "source_temporal_discourse_id": artifact.source_temporal_discourse_id,
        "source_temporal_discourse_sha256": artifact.source_temporal_discourse_sha256,
        "source_sha256": artifact.source_sha256,
        "options_sha256": artifact.options_sha256,
        "summary": artifact.summary,
    });
    let bytes = crate::document::hash::canonical_json_bytes(&value)?;
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
