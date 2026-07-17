use crate::types::SemanticType;
use lexflex_model::{ConceptId, EntityId, SemanticExpression, VariableId};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UnifyError {
    #[error("shape mismatch")]
    ShapeMismatch,
    #[error("missing binding {0}")]
    MissingBinding(lexflex_model::ParameterId),
    #[error("conflicting binding for {variable}")]
    ConflictingBinding {
        variable: VariableId,
        existing: SemanticExpression,
        incoming: SemanticExpression,
    },
    #[error("value mismatch")]
    ValueMismatch {
        pattern: SemanticExpression,
        candidate: SemanticExpression,
    },
    #[error("occurs check failed for {variable}")]
    OccursCheck {
        variable: VariableId,
        candidate: SemanticExpression,
    },
    #[error("bound scope underflow")]
    BoundScopeUnderflow,
    #[error("unknown variable type {0}")]
    UnknownVariableType(VariableId),
    #[error("type mismatch for {variable}: expected {expected:?}, found {actual:?}")]
    VariableTypeMismatch {
        variable: VariableId,
        expected: SemanticType,
        actual: SemanticType,
    },
    #[error("unification depth limit exceeded (max {depth})")]
    DepthLimitExceeded { depth: usize },
    #[error("unknown concept {0}")]
    UnknownConcept(ConceptId),
    #[error("unknown entity {0}")]
    UnknownEntity(EntityId),
}

impl UnifyError {
    pub fn is_candidate_mismatch(&self) -> bool {
        matches!(
            self,
            Self::ShapeMismatch
                | Self::MissingBinding(_)
                | Self::ConflictingBinding { .. }
                | Self::ValueMismatch { .. }
                | Self::OccursCheck { .. }
                | Self::VariableTypeMismatch { .. }
        )
    }

    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::DepthLimitExceeded { .. }
                | Self::UnknownConcept(_)
                | Self::UnknownEntity(_)
        )
    }
}
