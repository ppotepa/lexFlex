use super::model::DocumentCompilation;
use sha2::{Digest, Sha256};

pub fn compilation_hash(
    compilation: &DocumentCompilation,
) -> Result<String, serde_json::Error> {
    let mut canonical = compilation.clone();
    canonical.compilation_sha256.clear();
    let bytes = serde_json::to_vec(&canonical)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
