use crate::document::{Document, DocumentBlock, DocumentBlockKind, DocumentStructureError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreservedBlockDigest {
    pub count: usize,
    pub sha256: String,
}

pub struct PreservedBlockAccumulator {
    count: usize,
    hasher: Sha256,
}

impl PreservedBlockAccumulator {
    pub fn new() -> Self {
        Self {
            count: 0,
            hasher: Sha256::new(),
        }
    }

    pub fn push(&mut self, block: &DocumentBlock, span: crate::document::SourceSpan, text: &str) {
        self.hasher.update(block.id.as_str().as_bytes());
        self.hasher.update([0u8]);
        self.hasher.update(span.start.to_le_bytes());
        self.hasher.update(span.end.to_le_bytes());
        self.hasher.update(text.as_bytes());
        self.hasher.update([0xff]);
        self.count += 1;
    }

    pub fn finish(self) -> PreservedBlockDigest {
        PreservedBlockDigest {
            count: self.count,
            sha256: format!("{:x}", self.hasher.finalize()),
        }
    }
}

impl Default for PreservedBlockAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn is_preserved_block(kind: &DocumentBlockKind) -> bool {
    matches!(
        kind,
        DocumentBlockKind::PreservedWhitespace | DocumentBlockKind::PreservedRaw
    )
}

pub fn canonical_preserved_blocks(
    document: &Document,
) -> Result<PreservedBlockDigest, Vec<DocumentStructureError>> {
    let mut errors = Vec::new();
    let mut accumulator = PreservedBlockAccumulator::new();

    for block_id in document.block_order() {
        let Some(block) = document.block(block_id) else {
            errors.push(DocumentStructureError::MissingBlock {
                id: block_id.as_str().to_string(),
            });
            continue;
        };

        if !is_preserved_block(&block.kind) {
            continue;
        }

        let Some(span) = block.span.span else {
            errors.push(DocumentStructureError::NonExactSpan {
                id: block.id.as_str().to_string(),
                field: "block.span".to_string(),
            });
            continue;
        };

        let Ok(text) = span.slice(document.source()) else {
            errors.push(DocumentStructureError::InvalidSpan {
                id: block.id.as_str().to_string(),
                field: "block.span".to_string(),
            });
            continue;
        };

        accumulator.push(block, span, text);
    }

    if errors.is_empty() {
        Ok(accumulator.finish())
    } else {
        errors.sort_by_key(|error| format!("{error:?}"));
        errors.dedup();
        Err(errors)
    }
}
