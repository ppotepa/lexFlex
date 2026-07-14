use super::hash::document_knowledge_hash;
use super::model::DocumentKnowledgeExtraction;

pub struct DocumentKnowledgeValidator;

impl DocumentKnowledgeValidator {
    pub fn validate(artifact: &DocumentKnowledgeExtraction) -> Result<(), &'static str> {
        if !artifact.schema.is_supported() {
            return Err("unsupported schema");
        }
        if artifact.proposition_order.len() != artifact.proposition_occurrences.len() {
            return Err("proposition order mismatch");
        }
        if artifact.claim_order.len() != artifact.claims.len() {
            return Err("claim order mismatch");
        }
        if artifact.value_order.len() != artifact.values.len() {
            return Err("value order mismatch");
        }
        if artifact.contradiction_order.len() != artifact.contradiction_sets.len() {
            return Err("contradiction order mismatch");
        }
        if let Ok(expected) = document_knowledge_hash(artifact) {
            if expected != artifact.knowledge_sha256 {
                return Err("hash mismatch");
            }
        }
        Ok(())
    }
}
