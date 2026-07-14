mod support;

use lexflex::api::LexFlexAPI;
use lexflex::document::compilation::{DocumentCompilationValidator, DocumentCompiler};
use lexflex::document::translation::{BestEffortDocumentTranslator, DocumentTranslationValidator, LexFlexSentenceGenerator};
use support::load_document_v1_cases;

#[test]
fn chapter3_gate_reports_expected_totals() {
    let api = LexFlexAPI::builder().build().unwrap();
    let cases = load_document_v1_cases();
    let mut summary = Chapter3GateSummary::default();
    for case in cases {
        summary.documents += 1;
        summary.sentences += case.expected_sentences;
        summary.paragraphs += case.expected_paragraphs;
        let compilation = api.compile_document(&case.source, "pl").unwrap();
        let translator = BestEffortDocumentTranslator::new(LexFlexSentenceGenerator::new(&api));
        let translation = translator.translate(&compilation, lexflex::core::interlingua::LanguageId::new("en")).unwrap();
        summary.compilation_results += compilation.sentence_results.len();
        summary.translation_results += translation.sentence_results.len();
        summary.resolved += compilation.summary.resolved;
        summary.partial += compilation.summary.partial;
        summary.unresolved += compilation.summary.unresolved;
        summary.failed += compilation.summary.failed;
        summary.translated += translation.summary.translated;
        summary.source_fallback += translation.summary.source_fallback;
        summary.placeholder += translation.summary.placeholder;
        summary.empty_generated_rejected += translation.summary.empty_generated_rejected;
        assert!(DocumentCompilationValidator::validate(&compilation).is_ok(), "{}", case.id);
        assert!(DocumentTranslationValidator::validate(&compilation, &translation).is_ok(), "{}", case.id);
        assert_eq!(compilation.silent_drop_count(), 0, "{}", case.id);
    }
    assert_eq!(summary.documents, 30);
    assert_eq!(summary.sentences, 82);
    assert_eq!(summary.compilation_results, 82);
    assert_eq!(summary.translation_results, 82);
    assert_eq!(summary.placeholder, 0);
}

#[derive(Debug, Default)]
struct Chapter3GateSummary {
    documents: usize,
    paragraphs: usize,
    sentences: usize,
    compilation_results: usize,
    translation_results: usize,
    resolved: usize,
    partial: usize,
    unresolved: usize,
    failed: usize,
    translated: usize,
    source_fallback: usize,
    placeholder: usize,
    empty_generated_rejected: usize,
}
