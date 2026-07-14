use super::model::DocumentKnowledgeExtraction;

pub fn document_knowledge_hash(artifact: &DocumentKnowledgeExtraction) -> Result<String, serde_json::Error> {
    let mut clone = artifact.clone();
    clone.knowledge_sha256.clear();
    let bytes = serde_json::to_vec(&clone)?;
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
