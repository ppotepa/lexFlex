use crate::{canonical_hash, EvidenceId};
use serde::{Deserialize, Serialize};

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
    pub source_hash: Option<String>,
}

impl Evidence {
    pub fn create(
        source_id: impl Into<String>,
        span: Option<SourceSpan>,
        source_hash: Option<String>,
    ) -> Self {
        let source_id = source_id.into();
        let id = canonical_hash(&(&source_id, &span, &source_hash));
        Self {
            id: EvidenceId::new_unchecked(format!("evidence:{id}")),
            source_id,
            span,
            source_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    InvalidSpan { start: u64, end: u64 },
}
