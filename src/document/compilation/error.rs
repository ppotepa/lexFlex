use super::validation::DocumentCompilationValidationError;
use crate::document::validation::DocumentStructureError;

#[derive(Debug, thiserror::Error)]
pub enum DocumentCompilationError {
    #[error("input document is structurally invalid")]
    InvalidDocument {
        errors: Vec<DocumentStructureError>,
    },
    #[error("input document reconstruction failed")]
    Reconstruction {
        errors: Vec<DocumentStructureError>,
    },
    #[error("compilation artifact is invalid")]
    InvalidCompilation {
        errors: Vec<DocumentCompilationValidationError>,
    },
    #[error("failed to serialize compilation artifact: {0}")]
    Serialization(#[from] serde_json::Error),
}
