use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub max_text_bytes: usize,
    pub max_ron_bytes: usize,
    pub max_json_bytes: usize,
    pub max_document_bytes: usize,
    pub max_document_segments: usize,
    pub max_expression_nodes: usize,
    pub max_expression_depth: usize,
    pub max_generation_bytes: usize,
    pub max_provider_request_bytes: usize,
    pub max_provider_response_bytes: usize,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self {
            max_text_bytes: 8 * 1024 * 1024,
            max_ron_bytes: 32 * 1024 * 1024,
            max_json_bytes: 32 * 1024 * 1024,
            max_document_bytes: 8 * 1024 * 1024,
            max_document_segments: 100_000,
            max_expression_nodes: 100_000,
            max_expression_depth: 512,
            max_generation_bytes: 1_000_000,
            max_provider_request_bytes: 1024 * 1024,
            max_provider_response_bytes: 8 * 1024 * 1024,
        }
    }
}

impl ResourceBudget {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.max_text_bytes == 0
            || self.max_ron_bytes == 0
            || self.max_json_bytes == 0
            || self.max_document_bytes == 0
            || self.max_document_segments == 0
            || self.max_expression_nodes == 0
            || self.max_expression_depth == 0
            || self.max_generation_bytes == 0
            || self.max_provider_request_bytes == 0
            || self.max_provider_response_bytes == 0
        {
            return Err("resource budget limits must be non-zero");
        }
        Ok(())
    }
}
