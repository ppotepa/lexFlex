use crate::knowledge::snapshot::KnowledgeSnapshotError;
use lexflex_model::CanonicalDigest;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SessionIntegrityError {
    #[error("session schema mismatch: stored={stored}, expected={expected}")]
    SchemaMismatch { stored: u32, expected: u32 },
    #[error("session id is empty")]
    EmptySessionId,
    #[error("session model mismatch: stored={stored}, current={current}")]
    ModelHashMismatch {
        stored: CanonicalDigest,
        current: CanonicalDigest,
    },
    #[error("session language mismatch: stored={stored}, current={current}")]
    LanguageHashMismatch {
        stored: CanonicalDigest,
        current: CanonicalDigest,
    },
    #[error("knowledge integrity error: {0}")]
    Knowledge(#[from] KnowledgeSnapshotError),
}
