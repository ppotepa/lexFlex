use super::model::DocumentEntityResolution;
use super::DocumentEntityResolutionSerializationError;

pub fn entity_resolution_to_canonical_json(
    resolution: &DocumentEntityResolution,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(resolution)
}

pub fn entity_resolution_to_pretty_json(
    resolution: &DocumentEntityResolution,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(resolution)
}

pub fn entity_resolution_from_json(
    text: &str,
) -> Result<DocumentEntityResolution, DocumentEntityResolutionSerializationError> {
    let resolution: DocumentEntityResolution = serde_json::from_str(text)?;
    if !resolution.schema.is_supported() {
        return Err(DocumentEntityResolutionSerializationError::UnsupportedSchema);
    }
    super::validation::DocumentEntityResolutionValidator::validate_intrinsic(&resolution)
        .map_err(|_| DocumentEntityResolutionSerializationError::InvalidIntrinsicResolution)?;
    let expected = crate::document::resolution::document_entity_resolution_hash(&resolution)?;
    if expected != resolution.resolution_sha256 {
        return Err(DocumentEntityResolutionSerializationError::HashMismatch);
    }
    Ok(resolution)
}
