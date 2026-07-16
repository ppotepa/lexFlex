use crate::id::{FunctionId, SymbolId};
use lexflex_model::{ConceptId, ParameterId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ExecutionBudget {
    pub max_steps: u64,
    pub max_call_depth: usize,
    pub max_value_nodes: usize,
    pub max_trace_events: usize,
}

impl Default for ExecutionBudget {
    fn default() -> Self {
        Self {
            max_steps: 100_000,
            max_call_depth: 128,
            max_value_nodes: 100_000,
            max_trace_events: 10_000,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BudgetState {
    pub steps: u64,
    pub call_depth: usize,
    pub value_nodes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpansionMode {
    PreserveApplications,
    ExpandTransparent,
    ExpandAllDefined,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub expansion: ExpansionMode,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            expansion: ExpansionMode::PreserveApplications,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    #[error("budget exceeded")]
    BudgetExceeded,
    #[error("unknown local symbol: {0}")]
    UnknownLocal(SymbolId),
    #[error("unknown function: {0}")]
    UnknownFunction(FunctionId),
    #[error("unknown concept: {0}")]
    UnknownConcept(ConceptId),
    #[error("missing concept definition: {0}")]
    MissingConceptDefinition(ConceptId),
    #[error("missing argument: {0}")]
    MissingArgument(ParameterId),
    #[error("expected closure")]
    ExpectedClosure,
    #[error("expected semantic value")]
    ExpectedSemantic,
}
