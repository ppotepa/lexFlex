use super::id::options_fingerprint_bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityResolutionProfileKind {
    Conservative,
    Balanced,
    RecallOriented,
    DiagnosticOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentEntityResolutionOptions {
    pub profile: EntityResolutionProfileKind,
    pub max_sentence_distance: usize,
    pub max_paragraph_distance: usize,
    pub allow_cross_paragraph_proper_names: bool,
    pub allow_cross_paragraph_pronouns: bool,
    pub allow_same_sentence_cataphora: bool,
    pub resolve_definite_descriptions: bool,
    pub resolve_demonstratives: bool,
    pub resolve_reflexives: bool,
    pub detect_zero_subjects: bool,
    pub prefer_precision: bool,
    pub acceptance_threshold: i32,
    pub ambiguity_margin: i32,
    pub max_alternatives: usize,
    pub retain_rejected_candidates: bool,
}

impl Default for DocumentEntityResolutionOptions {
    fn default() -> Self {
        Self::conservative()
    }
}

impl DocumentEntityResolutionOptions {
    pub fn conservative() -> Self {
        Self {
            profile: EntityResolutionProfileKind::Conservative,
            max_sentence_distance: 6,
            max_paragraph_distance: 1,
            allow_cross_paragraph_proper_names: true,
            allow_cross_paragraph_pronouns: false,
            allow_same_sentence_cataphora: true,
            resolve_definite_descriptions: true,
            resolve_demonstratives: true,
            resolve_reflexives: true,
            detect_zero_subjects: true,
            prefer_precision: true,
            acceptance_threshold: 700,
            ambiguity_margin: 140,
            max_alternatives: 16,
            retain_rejected_candidates: true,
        }
    }

    pub fn fingerprint(&self) -> String {
        options_fingerprint_bytes(self)
    }
}
