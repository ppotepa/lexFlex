use crate::knowledge::snapshot::KnowledgeSnapshotError;
use lexflex_model::CanonicalDigest;
use lexflex_store::SessionStoreError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SessionIntegrityError {
    #[error("session schema mismatch: stored={stored}, expected={expected}")]
    SchemaMismatch { stored: u32, expected: u32 },
    #[error("invalid session id: {value}")]
    InvalidSessionId { value: String },
    #[error("session id mismatch: stored={stored}, requested={requested}")]
    SessionIdMismatch { stored: String, requested: String },
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
    #[error("stored session payload is invalid: {0}")]
    StoredPayload(#[source] SessionStoreError),
    #[error("knowledge integrity error: {0}")]
    Knowledge(#[from] KnowledgeSnapshotError),
}
