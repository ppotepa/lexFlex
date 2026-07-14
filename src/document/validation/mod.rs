mod blocks;
mod error;
mod references;
mod sentences;

use crate::document::model::{sha256_bytes, Document};

pub use error::DocumentStructureError;

pub struct DocumentValidator;

impl DocumentValidator {
    pub fn validate(document: &Document) -> Result<(), Vec<DocumentStructureError>> {
        let mut errors = Vec::new();
        if document.source_sha256 != sha256_bytes(document.source.as_bytes()) {
            errors.push(DocumentStructureError::SourceHashMismatch);
        }
        blocks::validate(document, &mut errors);
        references::validate(document, &mut errors);
        sentences::validate(document, &mut errors);
        error::sort_errors(&mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
