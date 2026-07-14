use super::blocks::check_exact_span;
use super::DocumentStructureError;
use crate::document::model::{Document, DocumentBlockKind};
use std::collections::BTreeSet;

pub(super) fn validate(document: &Document, errors: &mut Vec<DocumentStructureError>) {
    let mut document_ordinals = BTreeSet::new();
    let mut expected_document_ordinal = 0;
    for block_id in &document.block_order {
        let Some(block) = document.blocks.get(block_id) else { continue };
        let DocumentBlockKind::Paragraph(paragraph_id) = &block.kind else { continue };
        let Some(paragraph) = document.paragraphs.get(paragraph_id) else { continue };
        let Some(paragraph_span) = paragraph.span.span else { continue };
        let mut expected_start = paragraph_span.start;
        for (paragraph_ordinal, sentence_id) in paragraph.sentence_order.iter().enumerate() {
            let Some(sentence) = document.sentences.get(sentence_id) else { continue };
            if sentence.paragraph_id != paragraph.id {
                errors.push(DocumentStructureError::SentenceParagraphMismatch { sentence_id: sentence.id.to_string() });
            }
            if sentence.paragraph_ordinal != paragraph_ordinal {
                errors.push(DocumentStructureError::SentenceOrdinalMismatch { id: sentence.id.to_string() });
            }
            if !document_ordinals.insert(sentence.document_ordinal) {
                errors.push(DocumentStructureError::DuplicateDocumentSentenceOrdinal { ordinal: sentence.document_ordinal });
            }
            if sentence.document_ordinal != expected_document_ordinal {
                errors.push(DocumentStructureError::DocumentSentenceOrdinalMismatch {
                    id: sentence.id.to_string(),
                    expected: expected_document_ordinal,
                    actual: sentence.document_ordinal,
                });
            }
            expected_document_ordinal += 1;
            check_exact_span(&document.source, &sentence.raw_span, errors, sentence.id.as_str(), "sentence.raw_span");
            check_exact_span(&document.source, &sentence.content_span, errors, sentence.id.as_str(), "sentence.content_span");
            let (Some(raw), Some(content)) = (sentence.raw_span.span, sentence.content_span.span) else { continue };
            if !paragraph_span.contains_span(&raw) {
                errors.push(DocumentStructureError::SpanOutsideParent { id: sentence.id.to_string(), parent: paragraph.id.to_string() });
            }
            if !raw.contains_span(&content) {
                errors.push(DocumentStructureError::ContentOutsideRaw { id: sentence.id.to_string() });
            }
            if content.is_empty() {
                errors.push(DocumentStructureError::EmptySentenceContent { id: sentence.id.to_string() });
            }
            if raw.start > expected_start {
                errors.push(DocumentStructureError::SentenceGapInParagraph { paragraph_id: paragraph.id.to_string(), start: expected_start, end: raw.start });
            } else if raw.start < expected_start {
                errors.push(DocumentStructureError::SentenceOverlapInParagraph { paragraph_id: paragraph.id.to_string(), left_end: expected_start, right_start: raw.start });
            }
            expected_start = expected_start.max(raw.end);
        }
        if !paragraph.sentence_order.is_empty() && expected_start != paragraph_span.end {
            errors.push(DocumentStructureError::SentenceCoverageMismatch {
                paragraph_id: paragraph.id.to_string(),
                expected_end: paragraph_span.end,
                actual_end: expected_start,
            });
        }
    }
    if expected_document_ordinal != document.sentences.len() {
        errors.push(DocumentStructureError::CoverageMismatch { end: expected_document_ordinal, len: document.sentences.len() });
    }
}
