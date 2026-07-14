use super::error::DocumentAssemblyError;
use super::preserved::PreservedBlockAccumulator;
use crate::document::model::{Document, DocumentBlockKind, Paragraph};
use crate::document::span::SourceSpan;
use crate::document::id::SentenceId;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledDocumentOutput {
    pub text: String,
    pub copied_block_count: usize,
    pub copied_block_sha256: String,
}

pub struct DocumentOutputAssembler;

impl DocumentOutputAssembler {
    pub fn assemble(
        document: &Document,
        generated: &BTreeMap<SentenceId, String>,
    ) -> Result<AssembledDocumentOutput, DocumentAssemblyError> {
        let mut output = String::new();
        let mut preserved = PreservedBlockAccumulator::new();
        for block_id in document.block_order() {
            let block = document.block(block_id).ok_or_else(|| {
                DocumentAssemblyError::MissingBlock(block_id.clone())
            })?;
            match &block.kind {
                DocumentBlockKind::PreservedWhitespace | DocumentBlockKind::PreservedRaw => {
                    let span = block.span.span.ok_or_else(|| DocumentAssemblyError::MissingExactSpan {
                        field: "block.span".into(),
                    })?;
                    let text = span.slice(document.source()).map_err(DocumentAssemblyError::Span)?;
                    output.push_str(text);
                    preserved.push(block, span, text);
                }
                DocumentBlockKind::Paragraph(paragraph_id) => {
                    let paragraph = document.paragraph(paragraph_id).ok_or_else(|| {
                        DocumentAssemblyError::MissingParagraph(paragraph_id.clone())
                    })?;
                    assemble_paragraph(document, paragraph, generated, &mut output)?;
                }
            }
        }
        let digest = preserved.finish();
        Ok(AssembledDocumentOutput {
            text: output,
            copied_block_count: digest.count,
            copied_block_sha256: digest.sha256,
        })
    }
}

fn assemble_paragraph(
    document: &Document,
    paragraph: &Paragraph,
    generated: &BTreeMap<SentenceId, String>,
    output: &mut String,
) -> Result<(), DocumentAssemblyError> {
    if paragraph.sentence_order.is_empty() {
        let span = paragraph.span.span.ok_or_else(|| DocumentAssemblyError::MissingExactSpan {
            field: "paragraph.span".into(),
        })?;
        output.push_str(span.slice(document.source()).map_err(DocumentAssemblyError::Span)?);
        return Ok(());
    }
    for sentence_id in &paragraph.sentence_order {
        let sentence = document.sentence(sentence_id).ok_or_else(|| {
            DocumentAssemblyError::MissingSentence(sentence_id.clone())
        })?;
        let raw = sentence.raw_span.span.ok_or_else(|| DocumentAssemblyError::MissingExactSpan {
            field: "sentence.raw_span".into(),
        })?;
        let content = sentence.content_span.span.ok_or_else(|| {
            DocumentAssemblyError::MissingExactSpan {
                field: "sentence.content_span".into(),
            }
        })?;
        if !raw.contains_span(&content) {
            return Err(DocumentAssemblyError::ContentOutsideRaw {
                sentence_id: sentence_id.clone(),
            });
        }
        let replacement = generated.get(sentence_id).ok_or_else(|| {
            DocumentAssemblyError::MissingGeneratedContent(sentence_id.clone())
        })?;
        let prefix = SourceSpan::new(raw.start, content.start).map_err(DocumentAssemblyError::Span)?;
        let suffix = SourceSpan::new(content.end, raw.end).map_err(DocumentAssemblyError::Span)?;
        output.push_str(prefix.slice(document.source()).map_err(DocumentAssemblyError::Span)?);
        output.push_str(replacement);
        output.push_str(suffix.slice(document.source()).map_err(DocumentAssemblyError::Span)?);
    }
    Ok(())
}
