use crate::solve::goal_validation::GoalValidationError;
use crate::solve::UnifyError;
use lexflex_model::{CanonicalHashError, ExpressionTypeError};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SolveError {
    #[error("{0}")]
    Goal(#[from] GoalValidationError),
    #[error("{0}")]
    Type(#[from] ExpressionTypeError),
    #[error("{0}")]
    Unify(#[from] UnifyError),
    #[error("{0}")]
    CanonicalHash(#[from] CanonicalHashError),
}
