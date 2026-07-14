use super::model::DocumentEntityResolution;

pub fn document_entity_resolution_hash(
    resolution: &DocumentEntityResolution,
) -> Result<String, serde_json::Error> {
    let mut canonical = resolution.clone();
    canonical.resolution_sha256.clear();
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(crate::document::hash::sha256_bytes(&bytes))
}
