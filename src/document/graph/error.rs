use crate::document::compilation::DocumentCompilationValidationError;
use crate::document::{GraphEdgeId, GraphNodeId, SentenceId};

use super::validation::DocumentGraphValidationError;

#[derive(Debug, thiserror::Error)]
pub enum DocumentGraphBuildError {
    #[error("invalid compilation artifact")]
    InvalidCompilation {
        errors: Vec<DocumentCompilationValidationError>,
    },
    #[error("duplicate node {id}")]
    DuplicateNode { id: GraphNodeId },
    #[error("duplicate edge {id}")]
    DuplicateEdge { id: GraphEdgeId },
    #[error("missing sentence result {sentence_id}")]
    MissingSentenceResult { sentence_id: SentenceId },
    #[error("invalid graph artifact")]
    InvalidFinalGraph { errors: Vec<DocumentGraphValidationError> },
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentGraphSerializationError {
    #[error("serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported graph schema")]
    UnsupportedSchema,
    #[error("invalid intrinsic graph")]
    InvalidIntrinsicGraph,
    #[error("hash mismatch")]
    HashMismatch,
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentGraphServiceError {
    #[error("document compilation failed: {0}")]
    Compilation(#[from] crate::document::service::DocumentServiceError),
    #[error("graph build failed: {0}")]
    Build(#[from] DocumentGraphBuildError),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}
