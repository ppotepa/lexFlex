use crate::id::FunctionId;
use lexflex_model::{ConceptId, ParameterId, SemanticTypeReferenceError, VariableId};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileTypeReferenceLocation {
    ContextVariable {
        variable: VariableId,
    },
    ConceptSelfParameter {
        concept: ConceptId,
        parameter: ParameterId,
    },
    ConceptParameter {
        concept: ConceptId,
        parameter: ParameterId,
    },
    FunctionParameter {
        function: FunctionId,
        parameter: ParameterId,
    },
    FunctionResult {
        function: FunctionId,
    },
    LambdaParameter {
        parameter: ParameterId,
    },
    Quantifier {
        variable: VariableId,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("invalid semantic type reference at {location:?}: {source}")]
pub struct CompileTypeReferenceError {
    pub location: CompileTypeReferenceLocation,
    pub source: SemanticTypeReferenceError,
}
