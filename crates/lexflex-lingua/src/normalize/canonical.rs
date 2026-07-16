use lexflex_model::SemanticExpression;
use sha2::{Digest, Sha256};

pub fn expression_sha256(expression: &SemanticExpression) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(expression)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
