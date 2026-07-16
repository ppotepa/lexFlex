use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceOperation {
    EnterExpression,
    ExitExpression,
    LoadLocal,
    CreateClosure,
    CallClosure,
    ApplyConcept,
    BuildSatisfies,
    BuildEquals,
    Normalize,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionTraceEvent {
    pub sequence: u64,
    pub operation: TraceOperation,
    pub expression_kind: String,
    pub details: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub events: Vec<ExecutionTraceEvent>,
}
