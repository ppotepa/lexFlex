use super::DocumentStructureError;
use crate::document::model::{Document, DocumentBlockKind};
use std::collections::BTreeSet;

pub(super) fn validate(document: &Document, errors: &mut Vec<DocumentStructureError>) {
    let mut paragraphs = BTreeSet::new();
    for block_id in &document.block_order {
        let Some(block) = document.blocks.get(block_id) else { continue };
        if let DocumentBlockKind::Paragraph(paragraph_id) = &block.kind {
            if !paragraphs.insert(paragraph_id.clone()) {
                errors.push(DocumentStructureError::DuplicateParagraphReference { id: paragraph_id.to_string() });
            }
            if !document.paragraphs.contains_key(paragraph_id) {
                errors.push(DocumentStructureError::MissingParagraph { id: paragraph_id.to_string() });
            }
        }
    }
    for paragraph_id in document.paragraphs.keys() {
        if !paragraphs.contains(paragraph_id) {
            errors.push(DocumentStructureError::UnreferencedParagraph { id: paragraph_id.to_string() });
        }
    }

    let mut sentences = BTreeSet::new();
    for paragraph in document.paragraphs.values() {
        for sentence_id in &paragraph.sentence_order {
            if !sentences.insert(sentence_id.clone()) {
                errors.push(DocumentStructureError::DuplicateSentenceId { id: sentence_id.to_string() });
            }
            if !document.sentences.contains_key(sentence_id) {
                errors.push(DocumentStructureError::MissingSentence { id: sentence_id.to_string() });
            }
        }
    }
    for sentence_id in document.sentences.keys() {
        if !sentences.contains(sentence_id) {
            errors.push(DocumentStructureError::UnreferencedSentence { id: sentence_id.to_string() });
        }
    }
}
