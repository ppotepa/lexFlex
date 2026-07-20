use lexflex_model::{SemanticTypeReferenceError, VariableId};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompileContextError {
    #[error("invalid query variable type for {variable}: {source}")]
    InvalidQueryVariableType {
        variable: VariableId,
        source: SemanticTypeReferenceError,
    },
}
