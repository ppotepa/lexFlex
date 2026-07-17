use crate::solve::goal_validation::GoalValidationError;
use crate::solve::{SolveTypeError, UnifyError};
use lexflex_model::CanonicalHashError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SolveError {
    #[error("{0}")]
    Goal(#[from] GoalValidationError),
    #[error("{0}")]
    Type(#[from] SolveTypeError),
    #[error("{0}")]
    Unify(#[from] UnifyError),
    #[error("{0}")]
    CanonicalHash(#[from] CanonicalHashError),
}
