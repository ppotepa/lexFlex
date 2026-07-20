use crate::solve::goal_validation::GoalValidationError;
use crate::solve::{GoalCanonicalizationError, UnifyError};
use lexflex_model::{AssertionCatalogError, CanonicalHashError};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SolveError {
    #[error("{0}")]
    Goal(#[from] GoalValidationError),
    #[error("{0}")]
    Unify(#[from] UnifyError),
    #[error("{0}")]
    CanonicalHash(#[from] CanonicalHashError),
    #[error("{0}")]
    GoalCanonicalization(#[from] GoalCanonicalizationError),
    #[error("candidate assertion integrity failed: {0}")]
    AssertionIntegrity(#[from] AssertionCatalogError),
}
