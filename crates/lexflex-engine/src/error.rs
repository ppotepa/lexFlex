use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineErrorCode {
    InvalidProgram,
    InvalidGoal,
    ModelError,
    RuntimeBudget,
    StoreError,
    InternalInvariant,
}
