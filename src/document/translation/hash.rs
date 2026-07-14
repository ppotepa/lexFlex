use super::model::DocumentTranslation;
use sha2::{Digest, Sha256};

pub fn translation_output_hash(output: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(output.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn translation_hash(
    translation: &DocumentTranslation,
) -> Result<String, serde_json::Error> {
    let mut canonical = translation.clone();
    canonical.translation_sha256.clear();
    let bytes = crate::document::hash::canonical_json_bytes(&canonical)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
