use super::model::DocumentCompilation;
use sha2::{Digest, Sha256};

pub fn compilation_hash(
    compilation: &DocumentCompilation,
) -> Result<String, serde_json::Error> {
    let value = serde_json::json!({
        "document_id": compilation.document.id,
        "source_sha256": compilation.document.source_sha256,
        "sentence_count": compilation.document.sentences.len(),
        "summary": compilation.summary,
    });
    let bytes = crate::document::hash::canonical_json_bytes(&value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
