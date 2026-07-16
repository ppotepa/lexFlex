use crate::solve::goal_validation::GoalValidationError;
use crate::solve::unify::UnifyError;
use crate::solve::SolveTypeError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SolveError {
    #[error("{0}")]
    Goal(#[from] GoalValidationError),
    #[error("{0}")]
    Type(#[from] SolveTypeError),
    #[error("{0}")]
    Unify(#[from] UnifyError),
}
