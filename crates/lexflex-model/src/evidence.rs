use crate::{canonical_hash, CanonicalDigest, CanonicalHashError, EvidenceId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: u64,
    pub end: u64,
}

impl SourceSpan {
    pub fn new(start: u64, end: u64) -> Result<Self, EvidenceError> {
        if end < start {
            return Err(EvidenceError::InvalidSpan { start, end });
        }
        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Evidence {
    pub id: EvidenceId,
    pub source_id: String,
    pub span: Option<SourceSpan>,
    pub source_hash: Option<CanonicalDigest>,
}

impl Evidence {
    pub fn create(
        source_id: impl Into<String>,
        span: Option<SourceSpan>,
        source_hash: Option<CanonicalDigest>,
    ) -> Result<Self, EvidenceError> {
        let source_id = source_id.into();
        if source_id.trim().is_empty() {
            return Err(EvidenceError::EmptySourceId);
        }
        if let Some(existing_span) = &span {
            SourceSpan::new(existing_span.start, existing_span.end)?;
        }

        let digest = canonical_hash(&EvidenceIdentity {
            source_id: &source_id,
            span: &span,
            source_hash: &source_hash,
        })?;

        Ok(Self {
            id: EvidenceId::new_unchecked(format!("evidence:{}", digest.as_str())),
            source_id,
            span,
            source_hash,
        })
    }

    pub fn verify(&self) -> Result<(), EvidenceError> {
        if self.source_id.trim().is_empty() {
            return Err(EvidenceError::EmptySourceId);
        }

        if let Some(span) = &self.span {
            SourceSpan::new(span.start, span.end)?;
        }

        let digest = canonical_hash(&EvidenceIdentity {
            source_id: &self.source_id,
            span: &self.span,
            source_hash: &self.source_hash,
        })?;

        let expected = EvidenceId::new_unchecked(format!("evidence:{}", digest.as_str()));
        if self.id != expected {
            return Err(EvidenceError::IdMismatch {
                stored: self.id.clone(),
                expected,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct EvidenceIdentity<'a> {
    source_id: &'a str,
    span: &'a Option<SourceSpan>,
    source_hash: &'a Option<CanonicalDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EvidenceError {
    #[error("invalid source span {start}..{end}")]
    InvalidSpan { start: u64, end: u64 },

    #[error("evidence canonical hash failed: {0}")]
    CanonicalHash(#[from] CanonicalHashError),

    #[error("evidence id mismatch: stored={stored}, expected={expected}")]
    IdMismatch {
        stored: EvidenceId,
        expected: EvidenceId,
    },

    #[error("source id is empty")]
    EmptySourceId,
}
