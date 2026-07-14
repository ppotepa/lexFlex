use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum DocumentEntityResolutionError {
    #[error("invalid graph artifact")]
    InvalidGraph,
    #[error("missing mention {0}")]
    MissingMention(String),
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentEntityResolutionSerializationError {
    #[error("serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported resolution schema")]
    UnsupportedSchema,
    #[error("invalid intrinsic resolution")]
    InvalidIntrinsicResolution,
    #[error("hash mismatch")]
    HashMismatch,
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentEntityResolutionServiceError {
    #[error("graph build failed: {0}")]
    Graph(#[from] crate::document::graph::DocumentGraphServiceError),
    #[error("resolution failed: {0}")]
    Resolution(#[from] DocumentEntityResolutionError),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentEntityResolutionValidationError {
    SchemaVersionMismatch,
    AlgorithmVersionMismatch,
    ResolutionIdMismatch,
    SourceDocumentIdMismatch,
    SourceGraphIdMismatch,
    SourceGraphHashMismatch,
    SourceHashMismatch,
    OptionsMismatch,
    MissingDecision,
    DuplicateDecision,
    MissingCluster,
    DuplicateCluster,
    MissingMentionProfile,
    DuplicateMentionProfile,
    MissingSyntheticMention,
    DuplicateSyntheticMention,
    DecisionOrderMismatch,
    ClusterOrderMismatch,
    MentionOrderMismatch,
    SummaryMismatch,
    HashMismatch,
    DecisionSelectedClusterNotAllowed,
    DecisionHardAcceptedWithoutEvidence,
    DecisionSelectedClusterMissing,
    ClusterMissingRepresentative,
    ClusterDuplicateMention,
    ClusterMentionNotResolved,
    ClusterIncompatibleMentions,
    GraphMentionNotCovered,
}
