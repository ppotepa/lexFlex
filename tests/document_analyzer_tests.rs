mod support;

use lexflex::document::compilation::DocumentCompiler;
use lexflex::document::compilation::DocumentSentenceAnalyzer;
use lexflex::document::compilation::SentenceAnalysisInput;
use support::{segmented, recording_analyzer, AlwaysSuccessAnalyzer, PanicAnalyzer};

#[test]
fn analyzer_receives_exact_content_span_text() {
    let (analyzer, calls) = recording_analyzer();
    let document = segmented("  Ala ma kota.  ");
    let _ = DocumentCompiler::new(analyzer).compile(document).unwrap();
    let calls = calls.lock().unwrap().clone();
    assert_eq!(calls, vec!["Ala ma kota.".to_string()]);
}

#[test]
fn analyzer_id_is_stable() {
    assert_eq!(AlwaysSuccessAnalyzer.analyzer_id(), "always-success");
    assert_eq!(PanicAnalyzer.analyzer_id(), "panic");
}

#[test]
fn repeated_analysis_is_deterministic() {
    let document = segmented("Ala ma kota.");
    let first = DocumentCompiler::new(AlwaysSuccessAnalyzer)
        .compile(document.clone())
        .unwrap();
    let second = DocumentCompiler::new(AlwaysSuccessAnalyzer)
        .compile(document)
        .unwrap();
    assert_eq!(first, second);
}

#[test]
fn source_language_is_forwarded_to_analyzer() {
    struct Checker;
    impl DocumentSentenceAnalyzer for Checker {
        fn analyzer_id(&self) -> &'static str {
            "checker"
        }
        fn analyze(
            &self,
            input: SentenceAnalysisInput<'_>,
        ) -> Result<lexflex::core::interlingua::Utterance, lexflex::document::compilation::SentenceAnalysisError>
        {
            assert_eq!(input.source_language.0, "pl");
            Ok(support::simple_resolved_utterance())
        }
    }
    let document = segmented("Ala ma kota.");
    let _ = DocumentCompiler::new(Checker).compile(document).unwrap();
}
