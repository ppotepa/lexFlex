use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ParseScore {
    pub lexical_priority: i64,
    pub application_count: u32,
    pub unresolved_types: u32,
}

impl ParseScore {
    pub fn lexical(priority: i64) -> Self {
        Self {
            lexical_priority: priority,
            application_count: 0,
            unresolved_types: 0,
        }
    }

    pub fn composed(left: Self, right: Self, unresolved_types: usize) -> Self {
        Self {
            lexical_priority: left.lexical_priority + right.lexical_priority,
            application_count: left.application_count + right.application_count + 1,
            unresolved_types: unresolved_types.min(u32::MAX as usize) as u32,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseMetrics {
    pub token_count: usize,
    pub lexical_candidate_count: usize,
    pub chart_item_count: usize,
    pub chart_replacement_count: usize,
    pub semantic_duplicate_count: usize,
    pub alternative_derivation_count: usize,
    pub rejected_application_count: usize,
    pub complete_semantic_count: usize,
    pub max_cell_size: usize,
    pub max_derivation_depth: usize,
    pub max_semantic_nodes: usize,
}
