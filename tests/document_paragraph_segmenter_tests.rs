use lexflex::core::interlingua::LanguageId;
use lexflex::document::{
    Document, DocumentBlockKind, DocumentInput, DocumentReconstructor,
    DocumentSegmentationError, DocumentSegmentationOptions, DocumentSegmenter,
    LosslessParagraphSegmenter, SentenceSegmentationMode, SpanPrecision,
};

fn segment(source: &str) -> Document {
    LosslessParagraphSegmenter
        .segment(
            DocumentInput {
                id: None,
                source_language: LanguageId::new("pl"),
                source: source.to_string(),
            },
            &DocumentSegmentationOptions::default(),
        )
        .unwrap()
}

fn assert_lossless(source: &str, expected_paragraphs: usize) -> Document {
    let document = segment(source);
    assert!(document.validate_structure().is_ok());
    assert!(DocumentReconstructor::verify_lossless(&document).is_ok());
    assert_eq!(document.reconstruct().unwrap(), source);
    assert_eq!(document.paragraphs().len(), expected_paragraphs);
    assert!(document.sentences().is_empty());
    document
}

fn block_kinds(document: &Document) -> Vec<&DocumentBlockKind> {
    document
        .block_order()
        .iter()
        .map(|id| &document.block(id).unwrap().kind)
        .collect()
}

#[test]
fn empty_and_whitespace_only_sources_are_lossless() {
    let empty = assert_lossless("", 0);
    assert!(empty.blocks().is_empty());

    let whitespace = assert_lossless(" \t\r\n", 0);
    assert_eq!(whitespace.blocks().len(), 1);
    assert!(matches!(
        block_kinds(&whitespace).as_slice(),
        [DocumentBlockKind::PreservedWhitespace]
    ));
}

#[test]
fn single_paragraph_and_terminal_line_endings_are_partitioned() {
    let plain = assert_lossless("Ala ma kota.", 1);
    assert_eq!(plain.blocks().len(), 1);

    for source in ["Ala ma kota.\n", "Ala ma kota.\r\n", "Ala ma kota.\r"] {
        let document = assert_lossless(source, 1);
        assert!(matches!(
            block_kinds(&document).as_slice(),
            [
                DocumentBlockKind::Paragraph(_),
                DocumentBlockKind::PreservedWhitespace
            ]
        ));
    }
}

#[test]
fn multiline_content_and_blank_lines_define_paragraphs() {
    let multiline = assert_lossless("Pierwsza linia\nDruga linia", 1);
    assert_eq!(multiline.paragraph_text(&multiline.ordered_paragraphs()[0].id), Some("Pierwsza linia\nDruga linia"));

    for source in ["A\n\nB", "A\r\n\r\nB", "A\r\rB"] {
        let document = assert_lossless(source, 2);
        assert!(matches!(
            block_kinds(&document).as_slice(),
            [
                DocumentBlockKind::Paragraph(_),
                DocumentBlockKind::PreservedWhitespace,
                DocumentBlockKind::Paragraph(_)
            ]
        ));
    }
}

#[test]
fn mixed_endings_and_multiple_blank_lines_are_preserved() {
    assert_lossless("A\r\nB\n\nC\rD", 2);
    assert_lossless("A\n\n\n\nB", 2);
    assert_lossless("A\n \t \nB", 2);
}

#[test]
fn leading_and_trailing_whitespace_are_separate_blocks() {
    let leading = assert_lossless("\n\nA", 1);
    assert!(matches!(
        block_kinds(&leading).as_slice(),
        [
            DocumentBlockKind::PreservedWhitespace,
            DocumentBlockKind::Paragraph(_)
        ]
    ));
    assert_lossless("A\n\n", 1);
    assert_lossless("\r\nA\r\n\r\n", 1);
}

#[test]
fn unicode_emoji_tabs_and_unicode_blank_lines_are_safe() {
    assert_lossless("Zażółć gęślą.\n\nŁódź płynie.", 2);
    assert_lossless("Kot 🐈.\n\nPies 🐕.", 2);
    assert_lossless("A\tB", 1);
    assert_lossless("A\n\u{2003}\nB", 2);
}

#[test]
fn lossy_options_are_rejected() {
    let input = || DocumentInput {
        id: None,
        source_language: LanguageId::new("pl"),
        source: " A ".to_string(),
    };
    let segmenter = LosslessParagraphSegmenter;
    let mut trim = DocumentSegmentationOptions::default();
    trim.trim_document_edges = true;
    assert!(matches!(
        segmenter.segment(input(), &trim),
        Err(DocumentSegmentationError::UnsupportedOption(option))
            if option == "trim_document_edges"
    ));

    let mut discard = DocumentSegmentationOptions::default();
    discard.preserve_empty_blocks = false;
    assert!(matches!(
        segmenter.segment(input(), &discard),
        Err(DocumentSegmentationError::UnsupportedOption(option))
            if option == "preserve_empty_blocks=false"
    ));
}

#[test]
fn sentence_mode_does_not_change_paragraph_segmentation() {
    let source = "A\n\nB";
    let input = |_mode| DocumentInput {
        id: None,
        source_language: LanguageId::new("pl"),
        source: source.to_string(),
    };
    let segmenter = LosslessParagraphSegmenter;
    let mut conservative = DocumentSegmentationOptions::default();
    conservative.sentence_mode = SentenceSegmentationMode::Conservative;
    let mut engine = conservative.clone();
    engine.sentence_mode = SentenceSegmentationMode::ExistingEngineCompatible;
    let first = segmenter.segment(input(conservative.sentence_mode), &conservative).unwrap();
    let second = segmenter.segment(input(engine.sentence_mode), &engine).unwrap();
    assert_eq!(first, second);
}

#[test]
fn fixed_edge_cases_form_exact_deterministic_partitions() {
    let fixtures = [
        "", " ", "\n", "\r", "\r\n", "A", "A\n", "\nA", "A\n\nB",
        "A\r\nB\n\nC\rD", "A\n \t \nB", "Zażółć 🐈\n\nŁódź",
    ];
    for source in fixtures {
        let first = segment(source);
        let second = segment(source);
        assert_eq!(first, second, "document mismatch for {source:?}");
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
        assert_eq!(first.source_sha256(), second.source_sha256());
        let mut cursor = 0;
        let mut reconstructed = String::new();
        for block_id in first.block_order() {
            let block = first.block(block_id).unwrap();
            assert_eq!(block.span.precision, SpanPrecision::Exact);
            let span = block.span.span.unwrap();
            assert!(!span.is_empty());
            assert!(span.validate_for(source).is_ok());
            assert_eq!(span.start, cursor);
            reconstructed.push_str(span.slice(source).unwrap());
            cursor = span.end;
        }
        assert_eq!(cursor, source.len());
        assert_eq!(reconstructed, source);
    }
}
