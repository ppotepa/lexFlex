mod support;

use lexflex::api::LexFlexAPI;
use lexflex::document::compilation::DocumentCompilationValidator;
use lexflex::document::translation::{DocumentTranslationValidator, LexFlexSentenceGenerator, BestEffortDocumentTranslator};
use support::load_document_v1_cases;

#[test]
fn document_v1_translation_gate_has_30_documents_and_82_sentences() {
    let cases = load_document_v1_cases();
    assert_eq!(cases.len(), 30);
    let mut total_sentences = 0;
    let api = LexFlexAPI::builder().build().unwrap();
    for case in cases {
        let compilation = api.compile_document(&case.source, "pl").unwrap();
        assert!(DocumentCompilationValidator::validate(&compilation).is_ok(), "{}", case.id);
        let translator = BestEffortDocumentTranslator::new(LexFlexSentenceGenerator::new(&api));
        let first = translator.translate(&compilation, lexflex::core::interlingua::LanguageId::new("en")).unwrap();
        let second = translator.translate(&compilation, lexflex::core::interlingua::LanguageId::new("en")).unwrap();
        assert!(DocumentTranslationValidator::validate(&compilation, &first).is_ok(), "{}", case.id);
        assert_eq!(first, second, "{}", case.id);
        assert!(!first.output.is_empty(), "{}", case.id);
        total_sentences += case.expected_sentences;
    }
    assert_eq!(total_sentences, 82);
}
