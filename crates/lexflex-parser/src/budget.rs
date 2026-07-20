#[derive(Debug, Clone)]
pub struct ParseBudget {
    pub max_tokens: usize,
    pub max_lexical_candidates_per_token: usize,
    pub max_items_per_cell: usize,
    pub max_total_items: usize,
    pub max_derivation_depth: usize,
    pub max_complete_parses: usize,
    pub max_semantic_nodes: usize,
    pub max_derivations_per_item: usize,
    pub max_total_derivations: usize,
}

impl Default for ParseBudget {
    fn default() -> Self {
        Self {
            max_tokens: 64,
            max_lexical_candidates_per_token: 32,
            max_items_per_cell: 128,
            max_total_items: 10_000,
            max_derivation_depth: 128,
            max_complete_parses: 16,
            max_semantic_nodes: 10_000,
            max_derivations_per_item: 64,
            max_total_derivations: 50_000,
        }
    }
}
