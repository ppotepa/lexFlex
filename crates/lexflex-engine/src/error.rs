use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineErrorCode {
    InvalidProgram,
    InvalidGoal,
    InvalidAssertion,
    InvalidEvidence,
    Canonicalization,
    Integrity,
    ModelError,
    RuntimeBudget,
    StoreError,
    InternalInvariant,
}
