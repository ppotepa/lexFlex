use lexflex_model::ConceptId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct VerificationLimits {
    pub max_expression_depth: usize,
    pub max_expression_nodes: usize,
    pub max_declarations: usize,
    pub max_parameters_per_function: usize,
    pub allow_recursive_concepts: bool,
}

impl Default for VerificationLimits {
    fn default() -> Self {
        Self {
            max_expression_depth: 256,
            max_expression_nodes: 100_000,
            max_declarations: 10_000,
            max_parameters_per_function: 64,
            allow_recursive_concepts: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub declaration_count: usize,
    pub expression_node_count: usize,
    pub max_expression_depth: usize,
    pub concept_dependencies: BTreeMap<ConceptId, Vec<ConceptId>>,
}
