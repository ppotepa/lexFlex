use lexflex::api::LexFlexAPI;
use lexflex::document::temporal_discourse::DocumentTemporalDiscourseValidator;

fn api() -> LexFlexAPI {
    LexFlexAPI::builder().data_dir("data").build().unwrap()
}

fn case_source() -> &'static str {
    include_str!("../benchmarks/document_v1/cases/dev/doc-pl-en-001/source.pl.txt")
}

#[test]
fn temporal_discourse_artifact_is_deterministic_and_valid() {
    let api = api();
    let first = api
        .compile_document_temporal_discourse(case_source(), "pl")
        .unwrap();
    let second = api
        .compile_document_temporal_discourse(case_source(), "pl")
        .unwrap();
    assert!(!first.temporal_discourse_sha256.is_empty());
    assert!(first.summary.sentences_total > 0);
    assert!(first.summary.event_profiles_total > 0);
    assert!(first.generation_plan.is_some());
    assert_eq!(first.summary, second.summary);
    assert_eq!(
        first
            .generation_plan
            .as_ref()
            .map(|plan| plan.ordered_steps.len()),
        second
            .generation_plan
            .as_ref()
            .map(|plan| plan.ordered_steps.len())
    );
    DocumentTemporalDiscourseValidator::validate(&first).unwrap();
    DocumentTemporalDiscourseValidator::validate(&second).unwrap();
}

#[test]
fn resolved_translation_matches_best_effort_output() {
    let api = api();
    let resolved = api
        .translate_document_resolved(case_source(), "pl", "en")
        .unwrap();
    let best_effort = api
        .translate_document_best_effort(case_source(), "pl", "en")
        .unwrap();
    assert_eq!(resolved.output, best_effort.output);
    assert_eq!(resolved.translation_sha256, best_effort.translation_sha256);
}
