use lexflex_language::{AtomicCategoryKind, CategoryTypeVariableId, FeatureName, FeatureValue, SlashDirection};
use lexflex_model::{ParameterId, SemanticType, TypeRelationError};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CategoryUnifyError {
    #[error("category shape mismatch")]
    ShapeMismatch,
    #[error("atomic category kind mismatch: expected={expected:?}, actual={actual:?}")]
    AtomicKindMismatch {
        expected: AtomicCategoryKind,
        actual: AtomicCategoryKind,
    },
    #[error("slash direction mismatch: expected={expected:?}, actual={actual:?}")]
    DirectionMismatch {
        expected: SlashDirection,
        actual: SlashDirection,
    },
    #[error("semantic parameter mismatch: expected={expected}, actual={actual}")]
    SemanticParameterMismatch {
        expected: ParameterId,
        actual: ParameterId,
    },
    #[error("required feature is missing: {0}")]
    MissingFeature(FeatureName),
    #[error("feature mismatch for {name}: expected={expected}, actual={actual}")]
    FeatureMismatch {
        name: FeatureName,
        expected: FeatureValue,
        actual: FeatureValue,
    },
    #[error("semantic type mismatch: expected={expected:?}, actual={actual:?}")]
    SemanticTypeMismatch {
        expected: SemanticType,
        actual: SemanticType,
    },
    #[error("category type conflict: {0}")]
    TypeRelation(#[from] TypeRelationError),
    #[error("unresolved category variable: {0}")]
    UnresolvedVariable(CategoryTypeVariableId),
    #[error("category alias cycle: {variables:?}")]
    AliasCycle {
        variables: Vec<CategoryTypeVariableId>,
    },
    #[error("category alias invariant failed for {0}")]
    AliasInvariant(CategoryTypeVariableId),
}

#[allow(dead_code)]
impl CategoryUnifyError {
    pub fn is_non_applicable(&self) -> bool {
        matches!(
            self,
            CategoryUnifyError::ShapeMismatch
                | CategoryUnifyError::AtomicKindMismatch { .. }
                | CategoryUnifyError::DirectionMismatch { .. }
                | CategoryUnifyError::MissingFeature(_)
                | CategoryUnifyError::FeatureMismatch { .. }
                | CategoryUnifyError::SemanticTypeMismatch { .. }
        )
    }
}