use crate::normalize::NormalizationError;
use crate::solve::GoalValidationError;
use lexflex_model::CanonicalHashError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GoalCanonicalizationError {
    #[error("goal validation failed: {0}")]
    Validation(#[from] GoalValidationError),

    #[error("goal normalization failed: {0}")]
    Normalization(#[from] NormalizationError),

    #[error("goal canonical hash failed: {0}")]
    CanonicalHash(#[from] CanonicalHashError),
}
