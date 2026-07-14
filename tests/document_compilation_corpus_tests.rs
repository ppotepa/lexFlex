mod support;

use lexflex::api::LexFlexAPI;
use lexflex::document::compilation::DocumentCompilationValidator;
use support::load_document_v1_cases;

#[test]
fn document_v1_compilation_gate_has_30_documents_and_82_sentences() {
    let cases = load_document_v1_cases();
    assert_eq!(cases.len(), 30);
    let mut total_sentences = 0;
    let api = LexFlexAPI::builder().build().unwrap();
    for case in cases {
        let first = api.compile_document(&case.source, "pl").unwrap();
        let second = api.compile_document(&case.source, "pl").unwrap();
        assert_eq!(first.document.sentences().len(), case.expected_sentences, "{}", case.id);
        assert_eq!(first.document.paragraphs().len(), case.expected_paragraphs, "{}", case.id);
        assert!(DocumentCompilationValidator::validate(&first).is_ok(), "{}", case.id);
        assert_eq!(first.silent_drop_count(), 0, "{}", case.id);
        assert_eq!(first.summary.pending, 0, "{}", case.id);
        assert_eq!(first.summary.total_sentences, case.expected_sentences, "{}", case.id);
        assert!(first
            .sentence_results
            .values()
            .all(|result| !result.provenance.source_sha256.is_empty()),
            "{}",
            case.id);
        let mut first_canonical = first.clone();
        first_canonical.compilation_sha256.clear();
        let mut second_canonical = second.clone();
        second_canonical.compilation_sha256.clear();
        assert_eq!(first_canonical, second_canonical, "{}", case.id);
        assert_ne!(first.compilation_sha256, "", "{}", case.id);
        total_sentences += case.expected_sentences;
    }
    assert_eq!(total_sentences, 82);
}
