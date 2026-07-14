use lexflex::core::interlingua::LanguageId;
use lexflex::document::{
    Document, DocumentBlock, DocumentBlockKind, DocumentBuildError, DocumentBuilder,
    DocumentIdFactory, DocumentInput, DocumentSentence, DocumentSegmentationOptions,
    DocumentStructureError, LocatedSpan, Paragraph, ParagraphId, SentenceId, SourceSpan,
};
use std::collections::BTreeMap;

fn exact(start: usize, end: usize) -> LocatedSpan {
    LocatedSpan::exact(SourceSpan::new(start, end).unwrap())
}

fn sample_document() -> Document {
    let source = "A. B.\n\nC.".to_string();
    let document_id = DocumentIdFactory::from_source("pl", &source);
    let mut document = Document::new(DocumentInput {
        id: Some(document_id.clone()),
        source_language: LanguageId::new("pl"),
        source,
    });
    let p1 = ParagraphId::new(&DocumentIdFactory::paragraph(&document_id, 0).to_string()).unwrap();
    let p2 = ParagraphId::new(&DocumentIdFactory::paragraph(&document_id, 1).to_string()).unwrap();
    let b1 = DocumentIdFactory::block(&document_id, 0);
    let b2 = DocumentIdFactory::block(&document_id, 1);
    let b3 = DocumentIdFactory::block(&document_id, 2);
    let s1 = SentenceId::new(&DocumentIdFactory::sentence(&document_id, 0, 0).to_string()).unwrap();
    let s2 = SentenceId::new(&DocumentIdFactory::sentence(&document_id, 0, 1).to_string()).unwrap();
    let s3 = SentenceId::new(&DocumentIdFactory::sentence(&document_id, 1, 0).to_string()).unwrap();
    let mut blocks = BTreeMap::new();
    blocks.insert(
        b1.clone(),
        DocumentBlock {
            id: b1.clone(),
            ordinal: 0,
            kind: DocumentBlockKind::Paragraph(p1.clone()),
            span: exact(0, 5),
        },
    );
    blocks.insert(
        b2.clone(),
        DocumentBlock {
            id: b2.clone(),
            ordinal: 1,
            kind: DocumentBlockKind::PreservedWhitespace,
            span: exact(5, 7),
        },
    );
    blocks.insert(
        b3.clone(),
        DocumentBlock {
            id: b3.clone(),
            ordinal: 2,
            kind: DocumentBlockKind::Paragraph(p2.clone()),
            span: exact(7, 9),
        },
    );
    let mut paragraphs = BTreeMap::new();
    paragraphs.insert(
        p1.clone(),
        Paragraph {
            id: p1.clone(),
            block_id: b1.clone(),
            ordinal: 0,
            span: exact(0, 5),
            sentence_order: vec![s1.clone(), s2.clone()],
        },
    );
    paragraphs.insert(
        p2.clone(),
        Paragraph {
            id: p2.clone(),
            block_id: b3.clone(),
            ordinal: 1,
            span: exact(7, 9),
            sentence_order: vec![s3.clone()],
        },
    );
    let mut sentences = BTreeMap::new();
    sentences.insert(
        s1.clone(),
        DocumentSentence {
            id: s1.clone(),
            paragraph_id: p1.clone(),
            document_ordinal: 0,
            paragraph_ordinal: 0,
            raw_span: exact(0, 3),
            content_span: exact(0, 2),
        },
    );
    sentences.insert(
        s2.clone(),
        DocumentSentence {
            id: s2.clone(),
            paragraph_id: p1.clone(),
            document_ordinal: 1,
            paragraph_ordinal: 1,
            raw_span: exact(3, 5),
            content_span: exact(3, 5),
        },
    );
    sentences.insert(
        s3.clone(),
        DocumentSentence {
            id: s3.clone(),
            paragraph_id: p2.clone(),
            document_ordinal: 2,
            paragraph_ordinal: 0,
            raw_span: exact(7, 9),
            content_span: exact(7, 9),
        },
    );
    document.block_order = vec![b1, b2, b3];
    document.blocks = blocks;
    document.paragraphs = paragraphs;
    document.sentences = sentences;
    document
}

#[test]
fn document_reconstructs_source_one_to_one() {
    let document = sample_document();
    assert_eq!(document.reconstruct().unwrap(), document.source);
    assert_eq!(document.paragraph_text(&document.ordered_paragraphs()[0].id).unwrap(), "A. B.");
    assert_eq!(document.sentence_text(&document.ordered_sentences()[0].id).unwrap(), "A.");
    assert_eq!(document.sentence_raw_text(&document.ordered_sentences()[2].id).unwrap(), "C.");
}

#[test]
fn document_validation_passes_for_well_formed_model() {
    let document = sample_document();
    assert!(document.validate_structure().is_ok());
}

#[test]
fn document_serialization_is_deterministic() {
    let a = serde_json::to_string(&sample_document()).unwrap();
    let b = serde_json::to_string(&sample_document()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn document_options_have_safe_defaults() {
    let options = DocumentSegmentationOptions::default();
    assert!(options.preserve_empty_blocks);
    assert!(!options.trim_document_edges);
}

#[test]
fn document_structure_errors_are_reported() {
    let mut document = sample_document();
    document.block_order.pop();
    assert!(matches!(
        document.validate_structure(),
        Err(errors) if errors.iter().any(|error| matches!(error, DocumentStructureError::BlockOrderMismatch))
    ));
}

#[test]
fn document_validation_reports_source_hash_mismatch() {
    let mut document = sample_document();
    document.source.push('X');
    assert!(matches!(
        document.validate_structure(),
        Err(errors) if errors.iter().any(|error| matches!(error, DocumentStructureError::SourceHashMismatch))
    ));
}

#[test]
fn document_validation_reports_sentence_gaps() {
    let mut document = sample_document();
    document
        .sentences
        .get_mut(&SentenceId::new(&DocumentIdFactory::sentence(&document.id, 0, 1).to_string()).unwrap())
        .unwrap()
        .raw_span = exact(4, 5);
    assert!(matches!(
        document.validate_structure(),
        Err(errors) if errors.iter().any(|error| matches!(error, DocumentStructureError::SentenceGapInParagraph { .. }))
    ));
}

#[test]
fn document_builder_creates_valid_documents() {
    let source = "A. B.\n\nC.".to_string();
    let mut builder = DocumentBuilder::new(DocumentInput {
        id: None,
        source_language: LanguageId::new("pl"),
        source: source.clone(),
    });
    let p1 = builder.push_paragraph(exact(0, 5)).unwrap();
    builder
        .push_sentence(p1.clone(), exact(0, 3), exact(0, 2))
        .unwrap();
    builder
        .push_sentence(p1, exact(3, 5), exact(3, 5))
        .unwrap();
    builder.push_preserved_whitespace(exact(5, 7)).unwrap();
    let p2 = builder.push_paragraph(exact(7, 9)).unwrap();
    builder
        .push_sentence(p2, exact(7, 9), exact(7, 9))
        .unwrap();
    let document = builder.finish().unwrap();
    assert_eq!(document.source_sha256().len(), 64);
}

fn builder_for(source: &str) -> DocumentBuilder {
    DocumentBuilder::new(DocumentInput {
        id: None,
        source_language: LanguageId::new("pl"),
        source: source.to_string(),
    })
}

#[test]
fn builder_rejects_noncontiguous_and_invalid_block_spans() {
    let mut starts_late = builder_for("AB");
    assert!(matches!(
        starts_late.push_paragraph(exact(1, 2)),
        Err(DocumentBuildError::NonContiguousBlock {
            expected_start: 0,
            actual_start: 1
        })
    ));

    let mut gap = builder_for("ABCD");
    gap.push_paragraph(exact(0, 1)).unwrap();
    assert!(matches!(
        gap.push_preserved_raw(exact(2, 3)),
        Err(DocumentBuildError::NonContiguousBlock {
            expected_start: 1,
            actual_start: 2
        })
    ));

    let mut overlap = builder_for("ABCD");
    overlap.push_paragraph(exact(0, 2)).unwrap();
    assert!(matches!(
        overlap.push_preserved_raw(exact(1, 3)),
        Err(DocumentBuildError::NonContiguousBlock {
            expected_start: 2,
            actual_start: 1
        })
    ));

    let mut out_of_bounds = builder_for("AB");
    assert!(matches!(
        out_of_bounds.push_paragraph(exact(0, 3)),
        Err(DocumentBuildError::SpanOutOfBounds { .. })
    ));

    let mut empty = builder_for("AB");
    assert!(matches!(
        empty.push_paragraph(exact(0, 0)),
        Err(DocumentBuildError::EmptyBlockSpan)
    ));
}

#[test]
fn builder_finish_rejects_incomplete_coverage() {
    let mut builder = builder_for("AB");
    builder.push_paragraph(exact(0, 1)).unwrap();
    assert!(matches!(
        builder.finish(),
        Err(errors) if errors.iter().any(|error| matches!(
            error,
            DocumentStructureError::CoverageMismatch { end: 1, len: 2 }
        ))
    ));
}

#[test]
fn builder_accepts_paragraph_whitespace_paragraph_partition() {
    let mut builder = builder_for("A\n\nB");
    builder.push_paragraph(exact(0, 1)).unwrap();
    builder.push_preserved_whitespace(exact(1, 3)).unwrap();
    builder.push_paragraph(exact(3, 4)).unwrap();
    let document = builder.finish().unwrap();
    assert_eq!(document.reconstruct().unwrap(), "A\n\nB");
}

#[test]
fn builder_rejects_invalid_sentence_ranges_before_mutation() {
    let mut builder = builder_for("A. B.");
    let paragraph = builder.push_paragraph(exact(0, 5)).unwrap();
    assert!(matches!(
        builder.push_sentence(paragraph.clone(), exact(1, 3), exact(1, 2)),
        Err(DocumentBuildError::NonContiguousSentence { .. })
    ));
    assert!(matches!(
        builder.push_sentence(paragraph.clone(), exact(0, 2), exact(0, 0)),
        Err(DocumentBuildError::EmptySentenceContent)
    ));
    assert!(matches!(
        builder.push_sentence(paragraph.clone(), exact(0, 2), exact(3, 5)),
        Err(DocumentBuildError::ContentOutsideRaw)
    ));
    assert!(matches!(
        builder.push_sentence(paragraph.clone(), exact(0, 6), exact(0, 2)),
        Err(DocumentBuildError::InvalidSpan { .. })
    ));
    builder
        .push_sentence(paragraph.clone(), exact(0, 3), exact(0, 2))
        .unwrap();
    assert!(matches!(
        builder.push_sentence(paragraph, exact(2, 5), exact(3, 5)),
        Err(DocumentBuildError::NonContiguousSentence { .. })
    ));
}

#[test]
fn validator_rejects_block_gaps_overlaps_and_incomplete_coverage() {
    let mut starts_late = sample_document();
    let first = starts_late.block_order[0].clone();
    starts_late.blocks.get_mut(&first).unwrap().span = exact(1, 5);
    assert!(matches!(
        starts_late.validate_structure(),
        Err(errors) if errors.iter().any(|error| matches!(error, DocumentStructureError::Gap { start: 0, .. }))
    ));

    let mut gap = sample_document();
    let second = gap.block_order[1].clone();
    gap.blocks.get_mut(&second).unwrap().span = exact(6, 7);
    assert!(matches!(
        gap.validate_structure(),
        Err(errors) if errors.iter().any(|error| matches!(error, DocumentStructureError::Gap { .. }))
    ));

    let mut overlap = sample_document();
    let second = overlap.block_order[1].clone();
    overlap.blocks.get_mut(&second).unwrap().span = exact(4, 7);
    assert!(matches!(
        overlap.validate_structure(),
        Err(errors) if errors.iter().any(|error| matches!(error, DocumentStructureError::Overlap { .. }))
    ));

    let mut incomplete = sample_document();
    let last = incomplete.block_order[2].clone();
    incomplete.blocks.get_mut(&last).unwrap().span = exact(7, 8);
    assert!(matches!(
        incomplete.validate_structure(),
        Err(errors) if errors.iter().any(|error| matches!(error, DocumentStructureError::CoverageMismatch { .. }))
    ));

    let mut overrun = sample_document();
    let last = overrun.block_order[2].clone();
    overrun.blocks.get_mut(&last).unwrap().span = exact(7, 10);
    assert!(matches!(
        overrun.validate_structure(),
        Err(errors) if errors.iter().any(|error| matches!(error, DocumentStructureError::CoverageMismatch { .. }))
    ));
}

#[test]
fn empty_document_is_a_valid_empty_partition() {
    let document = Document::new(DocumentInput {
        id: None,
        source_language: LanguageId::new("pl"),
        source: String::new(),
    });
    assert!(document.validate_structure().is_ok());
    assert_eq!(document.reconstruct().unwrap(), "");
}
