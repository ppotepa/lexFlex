use lexflex_model::SemanticExpression;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub value: SemanticExpression,
    pub trace: super::ExecutionTrace,
    pub steps: u64,
}

#[derive(Debug, Clone)]
pub struct TypedExecutionResult {
    pub execution: ExecutionResult,
    pub entry_type: crate::types::SemanticType,
}
