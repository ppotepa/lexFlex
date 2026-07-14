use super::DocumentStructureError;
use crate::document::model::{Document, DocumentBlockKind};
use crate::document::span::{LocatedSpan, SpanPrecision};
use std::collections::BTreeSet;

pub(super) fn validate(document: &Document, errors: &mut Vec<DocumentStructureError>) {
    if document.block_order.len() != document.blocks.len() {
        errors.push(DocumentStructureError::BlockOrderMismatch);
    }
    let mut seen = BTreeSet::new();
    let mut expected_start = 0;
    let mut paragraph_ordinal = 0;
    for (ordinal, block_id) in document.block_order.iter().enumerate() {
        if !seen.insert(block_id.clone()) {
            errors.push(DocumentStructureError::DuplicateBlockId { id: block_id.to_string() });
            continue;
        }
        let Some(block) = document.blocks.get(block_id) else {
            errors.push(DocumentStructureError::MissingBlock { id: block_id.to_string() });
            continue;
        };
        if block.ordinal != ordinal {
            errors.push(DocumentStructureError::BlockOrdinalMismatch { id: block.id.to_string() });
        }
        check_exact_span(&document.source, &block.span, errors, block.id.as_str(), "block.span");
        if let Some(span) = block.span.span {
            if span.is_empty() {
                errors.push(DocumentStructureError::EmptyBlockSpan { id: block.id.to_string() });
            }
            if span.start > expected_start {
                errors.push(DocumentStructureError::Gap { start: expected_start, end: span.start });
            } else if span.start < expected_start {
                errors.push(DocumentStructureError::Overlap { left_end: expected_start, right_start: span.start });
            }
            expected_start = expected_start.max(span.end);
            if block.kind == DocumentBlockKind::PreservedWhitespace
                && !span.slice(&document.source).unwrap_or_default().chars().all(char::is_whitespace)
            {
                errors.push(DocumentStructureError::PreservedWhitespaceContainsContent { id: block.id.to_string() });
            }
        }
        if let DocumentBlockKind::Paragraph(paragraph_id) = &block.kind {
            if let Some(paragraph) = document.paragraphs.get(paragraph_id) {
                if paragraph.ordinal != paragraph_ordinal {
                    errors.push(DocumentStructureError::ParagraphOrdinalMismatch { id: paragraph.id.to_string() });
                }
                paragraph_ordinal += 1;
                check_exact_span(&document.source, &paragraph.span, errors, paragraph.id.as_str(), "paragraph.span");
                if paragraph.block_id != block.id {
                    errors.push(DocumentStructureError::ParagraphBlockMismatch { paragraph_id: paragraph.id.to_string() });
                }
                if paragraph.span.span != block.span.span {
                    errors.push(DocumentStructureError::ParagraphBlockSpanMismatch { id: paragraph.id.to_string() });
                }
            }
        }
    }
    for block_id in document.blocks.keys() {
        if !seen.contains(block_id) {
            errors.push(DocumentStructureError::UnreferencedBlock { id: block_id.to_string() });
        }
    }
    if expected_start != document.source.len() {
        errors.push(DocumentStructureError::CoverageMismatch { end: expected_start, len: document.source.len() });
    }
}

pub(super) fn check_exact_span(
    source: &str,
    located: &LocatedSpan,
    errors: &mut Vec<DocumentStructureError>,
    id: &str,
    field: &str,
) {
    if located.precision != SpanPrecision::Exact || located.span.is_none() {
        errors.push(DocumentStructureError::NonExactSpan { id: id.to_string(), field: field.to_string() });
    }
    if let Some(span) = located.span {
        if let Err(error) = span.validate_for(source) {
            errors.push(DocumentStructureError::InvalidSpan { id: id.to_string(), field: format!("{field}: {error}") });
        }
    }
}
