use serde::{Deserialize, Serialize};

use super::id::KnowledgeExtractionId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentKnowledgeExtractionOptions {
    pub include_entity_class_claims: bool,
    pub include_event_occurrence_claims: bool,
    pub include_discourse_claims: bool,
    pub include_temporal_scope: bool,
    pub include_attribution: bool,
    pub include_contradictions: bool,
}

impl Default for DocumentKnowledgeExtractionOptions {
    fn default() -> Self {
        Self {
            include_entity_class_claims: true,
            include_event_occurrence_claims: true,
            include_discourse_claims: true,
            include_temporal_scope: true,
            include_attribution: true,
            include_contradictions: true,
        }
    }
}

impl DocumentKnowledgeExtractionOptions {
    pub fn fingerprint(&self) -> String {
        super::id::options_fingerprint_bytes(self)
    }
}

impl From<&DocumentKnowledgeExtractionOptions> for KnowledgeExtractionId {
    fn from(_value: &DocumentKnowledgeExtractionOptions) -> Self {
        KnowledgeExtractionId("knowledge-options".into())
    }
}
