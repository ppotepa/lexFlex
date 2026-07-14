use super::model::DocumentGraph;

pub fn document_graph_hash(graph: &DocumentGraph) -> Result<String, serde_json::Error> {
    let mut canonical = graph.clone();
    canonical.graph_sha256.clear();
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(crate::document::hash::sha256_bytes(&bytes))
}
