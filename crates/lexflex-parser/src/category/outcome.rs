use crate::diagnostic::ParseError;
use lexflex_language::{AtomicCategoryKind, CategoryType, CategoryTypeVariableId, FeatureName, FeatureValue, SlashDirection};
use lexflex_model::{CanonicalHashError, ParameterId, SemanticType};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoryMismatch {
    Shape,
    AtomicKind {
        expected: AtomicCategoryKind,
        actual: AtomicCategoryKind,
    },
    Direction {
        expected: SlashDirection,
        actual: SlashDirection,
    },
    SemanticParameter {
        expected: ParameterId,
        actual: ParameterId,
    },
    SemanticType {
        expected: SemanticType,
        actual: SemanticType,
    },
    SubstitutionConflict {
        existing: CategoryType,
        incoming: CategoryType,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CategoryInvariantError {
    #[error("category alias cycle: {variables:?}")]
    AliasCycle {
        variables: Vec<CategoryTypeVariableId>,
    },
    #[error("category alias invariant: {0}")]
    AliasInvariant(CategoryTypeVariableId),
    #[error("canonical hash failed: {0}")]
    CanonicalHash(#[from] CanonicalHashError),
}

impl From<CategoryInvariantError> for ParseError {
    fn from(value: CategoryInvariantError) -> Self {
        match value {
            CategoryInvariantError::CanonicalHash(error) => Self::CanonicalHash(error.to_string()),
            other => Self::Category(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoryOutcome<T> {
    Applied(T),
    NotApplicable(CategoryMismatch),
}
