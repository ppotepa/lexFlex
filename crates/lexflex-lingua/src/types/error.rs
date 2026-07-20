use crate::types::SemanticType;
use lexflex_model::{ConceptId, EntityId, ParameterId, VariableId};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TypeError {
    #[error("unknown concept: {0}")]
    UnknownConcept(ConceptId),
    #[error("unknown entity: {0}")]
    UnknownEntity(EntityId),
    #[error("unknown local symbol: {0}")]
    UnknownLocal(crate::id::SymbolId),
    #[error("unknown function: {0}")]
    UnknownFunction(crate::id::FunctionId),
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
    #[error("parameter {parameter} of {concept} expects {expected:?}, found {actual:?}")]
    ParameterTypeMismatch {
        concept: ConceptId,
        parameter: ParameterId,
        expected: SemanticType,
        actual: SemanticType,
    },
    #[error("expected predicate, found {found:?}")]
    ExpectedPredicate { found: SemanticType },
    #[error("equality operands are incompatible: {left:?} and {right:?}")]
    EqualityTypeMismatch {
        left: SemanticType,
        right: SemanticType,
    },
    #[error("logical expression expects Boolean, found {0:?}")]
    ExpectedBoolean(SemanticType),
    #[error("expected function, found {found:?}")]
    ExpectedFunction { found: SemanticType },
    #[error("unknown function argument {0}")]
    UnknownFunctionArgument(ParameterId),
}
