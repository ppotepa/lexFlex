use crate::{ConceptId, EntityId, ParameterId, QualifierId, SemanticType, VariableId};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExpressionTypeError {
    #[error("unknown concept: {0}")]
    UnknownConcept(ConceptId),

    #[error("unknown entity: {0}")]
    UnknownEntity(EntityId),

    #[error("unknown variable: {0}")]
    UnknownVariable(VariableId),

    #[error("concept {concept} requires parameter {parameter}")]
    MissingParameter {
        concept: ConceptId,
        parameter: ParameterId,
    },

    #[error("concept {concept} does not define parameter {parameter}")]
    UnknownParameter {
        concept: ConceptId,
        parameter: ParameterId,
    },

    #[error("concept {concept} parameter {parameter} expects {expected:?}, found {actual:?}")]
    ParameterTypeMismatch {
        concept: ConceptId,
        parameter: ParameterId,
        expected: SemanticType,
        actual: SemanticType,
    },

    #[error("satisfies expects predicate, found {0:?}")]
    ExpectedPredicate(SemanticType),

    #[error("predicate expects subject {expected:?}, found {actual:?}")]
    PredicateSubjectMismatch {
        expected: SemanticType,
        actual: SemanticType,
    },

    #[error("equality operands are incompatible: left={left:?}, right={right:?}")]
    EqualityMismatch {
        left: SemanticType,
        right: SemanticType,
    },

    #[error("logical operand must be Boolean, found {0:?}")]
    ExpectedBoolean(SemanticType),

    #[error("qualifier value is invalid for {qualifier}: {message}")]
    InvalidQualifier {
        qualifier: QualifierId,
        message: String,
    },

    #[error("bound variable scope underflow")]
    BoundScopeUnderflow,

    #[error("expression type-check node budget exceeded: max={max}")]
    NodeBudgetExceeded { max: usize },

    #[error("expression type-check depth budget exceeded: max={max}")]
    DepthBudgetExceeded { max: usize },
}
