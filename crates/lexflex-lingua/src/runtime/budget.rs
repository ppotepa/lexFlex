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
    pub max_expansion_depth: usize,
    pub max_expansions: u64,
    pub max_environment_bindings: usize,
}

impl Default for ExecutionBudget {
    fn default() -> Self {
        Self {
            max_steps: 100_000,
            max_call_depth: 128,
            max_value_nodes: 100_000,
            max_trace_events: 10_000,
            max_expansion_depth: 64,
            max_expansions: 10_000,
            max_environment_bindings: 4_096,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BudgetState {
    pub steps: u64,
    pub call_depth: usize,
    pub value_nodes: usize,
    pub expansion_depth: usize,
    pub expansions: u64,
    pub max_observed_environment_bindings: usize,
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
    #[error("concept expansion cycle: {path:?}")]
    ExpansionCycle { path: Vec<ConceptId> },
    #[error("concept expansion depth exceeded: max={max}")]
    ExpansionDepthExceeded { max: usize },
    #[error("concept expansion count exceeded: max={max}")]
    ExpansionCountExceeded { max: u64 },
    #[error("runtime environment binding limit exceeded: max={max}")]
    EnvironmentBindingLimitExceeded { max: usize },
    #[error("concept {concept} requires a self subject")]
    MissingSelfSubject { concept: ConceptId },
    #[error("expected closure")]
    ExpectedClosure,
    #[error("expected semantic value")]
    ExpectedSemantic,
    #[error("semantic normalization failed: {0}")]
    Normalization(String),
}
