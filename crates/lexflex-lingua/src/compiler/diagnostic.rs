use crate::compiler::{CompileContextError, CompileTypeReferenceError};
use crate::types::TypeError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct CompileDiagnostic {
    pub message: String,
}

impl CompileDiagnostic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<String> for CompileDiagnostic {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for CompileDiagnostic {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompileError {
    #[error(transparent)]
    Diagnostic(#[from] CompileDiagnostic),
    #[error(transparent)]
    Type(#[from] TypeError),
    #[error(transparent)]
    Context(#[from] CompileContextError),
    #[error(transparent)]
    TypeReference(#[from] CompileTypeReferenceError),
    #[error("canonical hash: {0}")]
    CanonicalHash(#[from] lexflex_model::CanonicalHashError),
    #[error("compiled model context catalog does not match compiler catalog")]
    ModelContextMismatch {
        compiler_catalog: lexflex_model::CanonicalDigest,
        model_catalog: lexflex_model::CanonicalDigest,
    },
    #[error("entry-only evaluation received {count} declarations")]
    UnexpectedEntryDeclarations { count: usize },
}
