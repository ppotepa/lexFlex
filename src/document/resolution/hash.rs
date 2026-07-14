use super::model::DocumentEntityResolution;

pub fn document_entity_resolution_hash(
    resolution: &DocumentEntityResolution,
) -> Result<String, serde_json::Error> {
    // Decisions contain diagnostic alternatives and map-shaped indexes whose
    // construction order is intentionally non-semantic. The artifact ID is
    // derived from the stable resolution contract and ordered graph inputs;
    // the full decision payload remains persisted and validated separately.
    let value = serde_json::json!({
        "schema": resolution.schema,
        "source_document_id": resolution.source_document_id,
        "source_graph_id": resolution.source_graph_id,
        "source_graph_sha256": resolution.source_graph_sha256,
        "source_sha256": resolution.source_sha256,
        "options_sha256": resolution.options_sha256,
    });
    let bytes = crate::document::hash::canonical_json_bytes(&value)?;
    Ok(crate::document::hash::sha256_bytes(&bytes))
}
