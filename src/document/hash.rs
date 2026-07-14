use crate::document::model::Document;
use crate::document::span::{SourceSpan, SourceSpanError};
use sha2::{Digest, Sha256};

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
