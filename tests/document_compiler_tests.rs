mod support;

use std::collections::BTreeSet;

use lexflex::document::compilation::{
    DocumentCompiler, SentenceProcessingStatus,
};
use support::{
    recording_analyzer, AlwaysSuccessAnalyzer, PartialAnalyzer, PanicAnalyzer,
    SelectiveFailureAnalyzer, UnresolvedAnalyzer, segmented,
};

#[test]
fn exactly_one_result_per_sentence() {
    let document = segmented("Pierwsze. Drugie.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer)
        .compile(document)
        .expect("compile");
    assert_eq!(compilation.sentence_results.len(), 2);
    assert_eq!(compilation.silent_drop_count(), 0);
    assert!(!compilation.has_pending_results());
}

#[test]
fn failure_does_not_stop_following_sentences() {
    let document = segmented("Pierwsze. Drugie. Trzecie.");
    let analyzer = SelectiveFailureAnalyzer {
        fail_ordinals: BTreeSet::from([1]),
    };
    let compilation = DocumentCompiler::new(analyzer).compile(document).unwrap();
    let ordered = compilation.ordered_results();
    assert_eq!(ordered[0].status, SentenceProcessingStatus::Resolved);
    assert_eq!(ordered[1].status, SentenceProcessingStatus::Failed);
    assert_eq!(ordered[2].status, SentenceProcessingStatus::Resolved);
    assert_eq!(compilation.silent_drop_count(), 0);
}

#[test]
fn panic_is_captured() {
    let document = segmented("Pierwsze.");
    let compilation = DocumentCompiler::new(PanicAnalyzer)
        .compile(document)
        .expect("compile");
    let result = compilation.ordered_results().pop().unwrap();
    assert_eq!(result.status, SentenceProcessingStatus::Failed);
    assert!(result.diagnostics.iter().any(|d| d.code == "DOC_ANALYSIS_PANIC"));
}

#[test]
fn unresolved_and_partial_statuses_are_distinct() {
    let unresolved = DocumentCompiler::new(UnresolvedAnalyzer)
        .compile(segmented("A."))
        .unwrap();
    let partial = DocumentCompiler::new(PartialAnalyzer)
        .compile(segmented("A."))
        .unwrap();
    assert_eq!(unresolved.ordered_results()[0].status, SentenceProcessingStatus::Unresolved);
    assert_eq!(partial.ordered_results()[0].status, SentenceProcessingStatus::Partial);
}

#[test]
fn content_span_not_raw_span_is_analyzed() {
    let (analyzer, seen) = recording_analyzer();
    let document = segmented("  Ala ma kota.  ");
    let _ = DocumentCompiler::new(analyzer).compile(document).unwrap();
    let texts = seen.lock().unwrap();
    assert_eq!(texts.as_slice(), ["Ala ma kota."]);
}
