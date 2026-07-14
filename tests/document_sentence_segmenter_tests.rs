use lexflex::core::interlingua::LanguageId;
use lexflex::document::{
    DocumentInput, DocumentSegmentationOptions, DocumentSegmenter, LosslessDocumentSegmenter,
    LosslessParagraphSegmenter, SegmentationRules, SentenceBoundaryKind, SentenceRangeSegmenter,
    SourceSpan,
};

fn full_document(source: &str, language: &str) -> lexflex::document::Document {
    LosslessDocumentSegmenter
        .segment(
            DocumentInput {
                id: None,
                source_language: LanguageId::new(language),
                source: source.to_string(),
            },
            &DocumentSegmentationOptions::default(),
        )
        .unwrap()
}

fn texts(source: &str, language: &str) -> Vec<String> {
    full_document(source, language)
        .ordered_sentences()
        .iter()
        .map(|sentence| source[sentence.content_span.span.unwrap().start..sentence.content_span.span.unwrap().end].to_string())
        .collect()
}

#[test]
fn sentence_ranges_cover_basic_terminal_forms() {
    assert_eq!(texts("A. B! C? D… E?! F...", "pl"), vec!["A.", "B!", "C?", "D…", "E?!", "F..."]);
}

#[test]
fn closing_quotes_and_brackets_stay_with_sentence() {
    assert_eq!(
        texts("Powiedział: \"Tak.\" (Potem wyszedł.) Koniec.", "pl"),
        vec!["Powiedział: \"Tak.\"", "(Potem wyszedł.)", "Koniec."]
    );
}

#[test]
fn unterminated_text_becomes_end_of_paragraph_sentence() {
    let document = full_document("Tekst bez kropki", "pl");
    let sentence = document.ordered_sentences()[0];
    assert_eq!(document.sentence_text(&sentence.id), Some("Tekst bez kropki"));
    let ranges = SentenceRangeSegmenter::new(SegmentationRules::for_language(&LanguageId::new("pl")))
        .segment(document.source(), SourceSpan { start: 0, end: document.source().len() })
        .unwrap();
    assert_eq!(ranges[0].boundary_kind, SentenceBoundaryKind::EndOfParagraph);
}

#[test]
fn decimals_versions_and_domains_do_not_split() {
    assert_eq!(
        texts("Wersja 1.2 działa. Liczba 3.14 rośnie. Otwórz example.com. Koniec.", "pl"),
        vec!["Wersja 1.2 działa.", "Liczba 3.14 rośnie.", "Otwórz example.com.", "Koniec."]
    );
}

#[test]
fn configured_abbreviations_and_initials_do_not_split() {
    assert_eq!(
        texts("Dr. Kowalski spotkał prof. Nowaka. Np. omówili plan. A. Kowalski wyszedł.", "pl"),
        vec!["Dr. Kowalski spotkał prof. Nowaka.", "Np. omówili plan.", "A. Kowalski wyszedł."]
    );
    assert_eq!(
        texts("Mr. Smith met J. R. R. Tolkien. It was brief, e.g. ten minutes.", "en"),
        vec!["Mr. Smith met J. R. R. Tolkien.", "It was brief, e.g. ten minutes."]
    );
}

#[test]
fn unicode_emoji_and_line_endings_preserve_exact_ranges() {
    for source in [
        "Zażółć 🐈. Łódź płynie.",
        "Pierwsze.\nDrugie.",
        "Pierwsze.\r\nDrugie.",
        "Pierwsze.\rDrugie.",
    ] {
        let document = full_document(source, "pl");
        assert_eq!(document.reconstruct().unwrap(), source);
        assert_eq!(document.sentences().len(), 2);
        assert!(document.validate_structure().is_ok());
    }
}

#[test]
fn sentence_raw_spans_partition_each_paragraph() {
    let source = "  Pierwsze.  Drugie bez kropki\n\nTrzecie!  ";
    let document = full_document(source, "pl");
    for paragraph in document.ordered_paragraphs() {
        let paragraph_span = paragraph.span.span.unwrap();
        let mut cursor = paragraph_span.start;
        for sentence_id in &paragraph.sentence_order {
            let sentence = document.sentence(sentence_id).unwrap();
            let raw = sentence.raw_span.span.unwrap();
            let content = sentence.content_span.span.unwrap();
            assert_eq!(raw.start, cursor);
            assert!(raw.contains_span(&content));
            assert!(raw.validate_for(source).is_ok());
            assert!(content.validate_for(source).is_ok());
            cursor = raw.end;
        }
        assert_eq!(cursor, paragraph_span.end);
    }
}

#[test]
fn full_segmentation_is_deterministic() {
    let source = "A. B.\n\nC bez kropki";
    let first = full_document(source, "pl");
    let second = full_document(source, "pl");
    assert_eq!(first, second);
    assert_eq!(serde_json::to_string(&first).unwrap(), serde_json::to_string(&second).unwrap());
}

#[test]
fn paragraph_only_segmenter_still_creates_no_sentences() {
    let document = LosslessParagraphSegmenter
        .segment(
            DocumentInput {
                id: None,
                source_language: LanguageId::new("pl"),
                source: "A. B.".to_string(),
            },
            &DocumentSegmentationOptions::default(),
        )
        .unwrap();
    assert!(document.sentences().is_empty());
}

#[test]
fn custom_abbreviation_rules_are_respected() {
    let source = "Xyz. Kowalski przyszedł. Koniec.";
    let segmenter = SentenceRangeSegmenter::new(
        SegmentationRules::default().with_abbreviation("xyz."),
    );
    let ranges = segmenter
        .segment(source, SourceSpan { start: 0, end: source.len() })
        .unwrap();
    assert_eq!(ranges.len(), 2);
}
