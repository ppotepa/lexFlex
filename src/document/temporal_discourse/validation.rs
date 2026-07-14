use super::hash::document_temporal_discourse_hash;
use super::model::DocumentTemporalDiscourse;

pub struct DocumentTemporalDiscourseValidator;

impl DocumentTemporalDiscourseValidator {
    pub fn validate(artifact: &DocumentTemporalDiscourse) -> Result<(), &'static str> {
        if !artifact.schema.is_supported() {
            return Err("unsupported schema");
        }
        if let Ok(expected) = document_temporal_discourse_hash(artifact) {
            if expected != artifact.temporal_discourse_sha256 {
                return Err("hash mismatch");
            }
        }
        Ok(())
    }
}
