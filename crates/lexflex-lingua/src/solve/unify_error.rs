use lexflex_model::{ExpressionTypeError, VariableId};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UnifyError {
    #[error("bound scope underflow")]
    BoundScopeUnderflow,
    #[error("unknown variable type {0}")]
    UnknownVariableType(VariableId),
    #[error("unification depth limit exceeded (max {depth})")]
    DepthLimitExceeded { depth: usize },
    #[error("candidate type error: {0}")]
    CandidateType(ExpressionTypeError),
}
