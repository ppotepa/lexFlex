use super::model::DocumentTemporalDiscourse;

pub fn document_temporal_discourse_hash(
    artifact: &DocumentTemporalDiscourse,
) -> Result<String, serde_json::Error> {
    let mut canonical = artifact.clone();
    canonical.temporal_discourse_sha256.clear();
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(crate::document::hash::sha256_bytes(&bytes))
}
