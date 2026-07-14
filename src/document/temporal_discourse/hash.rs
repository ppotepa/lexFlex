use super::model::DocumentTemporalDiscourse;

pub fn document_temporal_discourse_hash(
    artifact: &DocumentTemporalDiscourse,
) -> Result<String, serde_json::Error> {
    let value = serde_json::json!({
        "schema": artifact.schema,
        "source_document_id": artifact.source_document_id,
        "source_graph_id": artifact.source_graph_id,
        "source_graph_sha256": artifact.source_graph_sha256,
        "source_resolution_id": artifact.source_resolution_id,
        "source_resolution_sha256": artifact.source_resolution_sha256,
        "source_sha256": artifact.source_sha256,
        "options_sha256": artifact.options_sha256,
        "summary": artifact.summary,
    });
    let bytes = crate::document::hash::canonical_json_bytes(&value)?;
    Ok(crate::document::hash::sha256_bytes(&bytes))
}
