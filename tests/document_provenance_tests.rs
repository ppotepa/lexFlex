mod support;

use lexflex::document::compilation::{
    DocumentCompiler, ProvenanceOperation, ProvenanceOutcome,
};
use lexflex::document::translation::{
    generation_failure_fallback_provenance, generation_skipped_fallback_provenance,
    generation_success_provenance, TranslationFallbackKind, DocumentSentenceGenerator,
    SentenceGenerationError, SentenceGenerationInput,
};
use lexflex::document::hash::sha256_text;
use support::{segmented, AlwaysSuccessAnalyzer, PanicAnalyzer};

struct ErrorGenerator;
impl DocumentSentenceGenerator for ErrorGenerator {
    fn generator_id(&self) -> &'static str { "error-generator" }
    fn generate(&self, input: SentenceGenerationInput<'_>) -> Result<String, SentenceGenerationError> {
        Err(SentenceGenerationError {
            sentence_id: input.sentence_id.clone(),
            code: "TEST_GENERATION_FAILURE".into(),
            message: "intentional".into(),
            cause: None,
        })
    }
}

#[test]
fn sentence_hash_is_content_hash() {
    let document = segmented("Ala ma kota. Kot śpi.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document.clone()).unwrap();
    let sentences = compilation.document.ordered_sentences();
    let sentence = sentences.first().unwrap();
    let span = sentence.content_span.span.unwrap();
    assert_eq!(compilation.sentence_results[&sentence.id].provenance.source_sha256, sha256_text(span.slice(document.source()).unwrap()));
}

#[test]
fn analysis_error_has_status_classified_terminal() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(PanicAnalyzer).compile(document).unwrap();
    let provenance = &compilation.sentence_results.values().next().unwrap().provenance;
    assert!(matches!(provenance.steps.last().unwrap().operation, ProvenanceOperation::StatusClassified));
    assert!(matches!(provenance.steps.last().unwrap().outcome, ProvenanceOutcome::Succeeded));
}

#[test]
fn skipped_generation_uses_skipped_outcome() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document.clone()).unwrap();
    let sentences = compilation.document.ordered_sentences();
    let sentence = sentences.first().unwrap();
    let prefix_len = compilation.sentence_results.get(&sentence.id).unwrap().provenance.steps.len();
    let skipped = generation_skipped_fallback_provenance(
        compilation.sentence_results.get(&sentence.id).unwrap(),
        sentence,
        "gen",
        "CODE",
        "message",
        TranslationFallbackKind::CopySource,
    );
    assert!(matches!(skipped.steps[prefix_len].operation, ProvenanceOperation::GenerationSkipped));
    assert!(matches!(skipped.steps[prefix_len].outcome, ProvenanceOutcome::Skipped));
}

#[test]
fn generation_prefix_is_preserved() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document.clone()).unwrap();
    let sentences = compilation.document.ordered_sentences();
    let sentence = sentences.first().unwrap();
    let translated = generation_success_provenance(
        compilation.sentence_results.get(&sentence.id).unwrap(),
        sentence,
        "gen",
    );
    assert_eq!(
        translated.steps[..compilation.sentence_results[&sentence.id].provenance.steps.len()],
        compilation.sentence_results[&sentence.id].provenance.steps
    );
}

#[test]
fn generator_id_is_retained_on_translation_helpers() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document.clone()).unwrap();
    let sentences = compilation.document.ordered_sentences();
    let sentence = sentences.first().unwrap();
    let translated = generation_failure_fallback_provenance(
        compilation.sentence_results.get(&sentence.id).unwrap(),
        sentence,
        "generator-x",
        "CODE",
        "message",
        TranslationFallbackKind::Placeholder,
    );
    assert_eq!(translated.generator_id.as_deref(), Some("generator-x"));
}
