use crate::{canonical_hash, CanonicalDigest, CanonicalHashError, EvidenceId};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SourceSpan {
    start: u64,
    end: u64,
}

impl SourceSpan {
    pub fn new(start: u64, end: u64) -> Result<Self, EvidenceError> {
        if end < start {
            return Err(EvidenceError::InvalidSpan { start, end });
        }
        Ok(Self { start, end })
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.end
    }
}

impl<'de> Deserialize<'de> for SourceSpan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawSourceSpan {
            start: u64,
            end: u64,
        }

        let raw = RawSourceSpan::deserialize(deserializer)?;
        SourceSpan::new(raw.start, raw.end).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Evidence {
    id: EvidenceId,
    source_id: String,
    span: Option<SourceSpan>,
    source_hash: Option<CanonicalDigest>,
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
            SourceSpan::new(existing_span.start(), existing_span.end())?;
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
            SourceSpan::new(span.start(), span.end())?;
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

    pub fn id(&self) -> &EvidenceId {
        &self.id
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn span(&self) -> Option<&SourceSpan> {
        self.span.as_ref()
    }

    pub fn source_hash(&self) -> Option<&CanonicalDigest> {
        self.source_hash.as_ref()
    }
}

impl<'de> Deserialize<'de> for Evidence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawEvidence {
            id: EvidenceId,
            source_id: String,
            span: Option<SourceSpan>,
            source_hash: Option<CanonicalDigest>,
        }

        let raw = RawEvidence::deserialize(deserializer)?;
        let value = Self {
            id: raw.id,
            source_id: raw.source_id,
            span: raw.span,
            source_hash: raw.source_hash,
        };
        value.verify().map_err(serde::de::Error::custom)?;
        Ok(value)
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

    #[error("evidence key mismatch: key={key}, evidence={evidence}")]
    KeyMismatch {
        key: EvidenceId,
        evidence: EvidenceId,
    },

    #[error("conflicting evidence with id {0}")]
    ConflictingEvidence(EvidenceId),
}
