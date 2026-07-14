use crate::document::model::Document;
use crate::document::validation::{DocumentStructureError, DocumentValidator};

pub struct DocumentReconstructor;

impl DocumentReconstructor {
    pub fn reconstruct(document: &Document) -> Result<String, Vec<DocumentStructureError>> {
        DocumentValidator::validate(document)?;
        reconstruct_blocks(document)
    }

    pub fn verify_lossless(document: &Document) -> Result<(), Vec<DocumentStructureError>> {
        DocumentValidator::validate(document)?;
        Self::verify_reconstruction_only(document)
    }

    pub(crate) fn verify_reconstruction_only(
        document: &Document,
    ) -> Result<(), Vec<DocumentStructureError>> {
        let reconstructed = reconstruct_blocks(document)?;
        if reconstructed == document.source {
            Ok(())
        } else {
            Err(vec![DocumentStructureError::CoverageMismatch {
                end: reconstructed.len(),
                len: document.source.len(),
            }])
        }
    }
}

fn reconstruct_blocks(document: &Document) -> Result<String, Vec<DocumentStructureError>> {
    let mut output = String::new();
    for block_id in &document.block_order {
        let block = document.blocks.get(block_id).ok_or_else(|| {
            vec![DocumentStructureError::MissingBlock {
                id: block_id.as_str().to_string(),
            }]
        })?;
        let span = block.span.span.ok_or_else(|| {
            vec![DocumentStructureError::NonExactSpan {
                id: block.id.as_str().to_string(),
                field: "block.span".into(),
            }]
        })?;
        output.push_str(span.slice(&document.source).map_err(|err| {
            vec![DocumentStructureError::InvalidSpan {
                id: block.id.as_str().to_string(),
                field: format!("block.span: {err}"),
            }]
        })?);
    }
    Ok(output)
}
