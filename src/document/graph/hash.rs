use super::model::DocumentGraph;

pub fn document_graph_hash(graph: &DocumentGraph) -> Result<String, serde_json::Error> {
    let value = serde_json::json!({
        "schema": graph.schema,
        "source_document_id": graph.source_document_id,
        "source_language": graph.source_language,
        "source_sha256": graph.source_sha256,
        "compilation_sha256": graph.compilation_sha256,
        "options_sha256": graph.options_sha256,
        "summary": graph.summary,
    });
    let bytes = crate::document::hash::canonical_json_bytes(&value)?;
    Ok(crate::document::hash::sha256_bytes(&bytes))
}
