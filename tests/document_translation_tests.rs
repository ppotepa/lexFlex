mod support;

use lexflex::document::translation::{BestEffortDocumentTranslator, DocumentTranslationOptions};
use support::{segmented, AlwaysSuccessAnalyzer, EmptyGenerator, PrefixGenerator, recording_generator};
use lexflex::document::compilation::DocumentCompiler;

#[test]
fn translation_pipeline_returns_output_and_is_deterministic() {
    let document = segmented("Ala ma kota. Kot śpi.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document).unwrap();
    let translator = BestEffortDocumentTranslator::new(PrefixGenerator {
        prefix: "EN:".into(),
    });
    let first = translator.translate(&compilation, lexflex::core::interlingua::LanguageId::new("en")).unwrap();
    let second = translator.translate(&compilation, lexflex::core::interlingua::LanguageId::new("en")).unwrap();
    assert_eq!(first, second);
    assert!(!first.output.is_empty());
    assert_eq!(first.summary.coverage_count(), 2);
}

#[test]
fn empty_generator_falls_back_to_source_copy() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document.clone()).unwrap();
    let translation = BestEffortDocumentTranslator::new(EmptyGenerator)
        .translate(&compilation, lexflex::core::interlingua::LanguageId::new("en"))
        .unwrap();
    let sentence = translation.sentence_results.values().next().unwrap();
    assert_eq!(sentence.status, lexflex::document::translation::SentenceTranslationStatus::SourceFallback);
    assert_eq!(sentence.generated_content, compilation.document.sentence_text(&sentence.sentence_id).unwrap());
}

#[test]
fn generator_calls_are_recorded_per_sentence() {
    let document = segmented("Ala ma kota. Kot śpi.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document).unwrap();
    let (generator, calls) = recording_generator();
    let _ = BestEffortDocumentTranslator::new(generator)
        .translate(&compilation, lexflex::core::interlingua::LanguageId::new("en"))
        .unwrap();
    let calls = calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 2);
}

