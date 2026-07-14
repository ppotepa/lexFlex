use crate::document::model::Document;
use crate::document::span::{SourceSpan, SourceSpanError};
use sha2::{Digest, Sha256};
use serde::Serialize;

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn sha256_text(text: &str) -> String {
    sha256_bytes(text.as_bytes())
}

pub fn sha256_span(document: &Document, span: SourceSpan) -> Result<String, SourceSpanError> {
    let text = span.slice(document.source())?;
    Ok(sha256_text(text))
}

/// Serialize semantic artifacts through serde_json's ordered object map so
/// HashMap iteration order can never leak into an artifact checksum.
pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    serde_json::to_vec(&value)
}
