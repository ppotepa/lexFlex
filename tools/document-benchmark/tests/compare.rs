use lexflex_document_benchmark::compare::compare_runs;
use lexflex_document_benchmark::model::{
    ComparisonStatus, CorpusSplit, DocumentCaseRun, DocumentMetrics, DocumentRun, MetricStatus,
    MetricValue, RunSummary,
};
use std::collections::BTreeMap;

fn metric(value: f64) -> MetricValue {
    MetricValue {
        value: Some(value),
        numerator: Some(value),
        denominator: Some(1.0),
        status: MetricStatus::Exact,
    }
}

fn metrics(
    translation_success: f64,
    output_non_empty: f64,
    deterministic: f64,
    paragraph: f64,
    sentence: f64,
    frame: f64,
    role: f64,
    expectation_passed: f64,
) -> DocumentMetrics {
    let mut technical = BTreeMap::new();
    technical.insert("translation_success".into(), metric(translation_success));
    technical.insert("output_non_empty".into(), metric(output_non_empty));
    technical.insert("deterministic".into(), metric(deterministic));
    technical.insert("parse_source_success".into(), metric(1.0));
    technical.insert("parse_target_success".into(), metric(1.0));

    let mut structure = BTreeMap::new();
    structure.insert("paragraph_preservation_ratio".into(), metric(paragraph));
    structure.insert("sentence_count_ratio".into(), metric(sentence));

    let mut semantics = BTreeMap::new();
    semantics.insert("frame_type_recall".into(), metric(frame));
    semantics.insert("verb_concept_recall".into(), metric(frame));
    semantics.insert("role_signature_recall".into(), metric(role));
    semantics.insert("proper_name_recall".into(), metric(1.0));
    semantics.insert("polarity_preservation".into(), metric(1.0));
    semantics.insert("tense_preservation".into(), metric(1.0));
    semantics.insert("quantification_preservation".into(), metric(1.0));
    semantics.insert("temporal_preservation".into(), metric(1.0));

    let mut glossary = BTreeMap::new();
    glossary.insert("glossary_compliance".into(), metric(1.0));

    let mut expectations = BTreeMap::new();
    expectations.insert("expectation_passed".into(), metric(expectation_passed));

    DocumentMetrics {
        technical,
        structure,
        semantics,
        glossary,
        expectations,
        reference: BTreeMap::new(),
    }
}

fn case(
    id: &str,
    output: Option<&str>,
    metrics: DocumentMetrics,
    runtime_ms: u128,
) -> DocumentCaseRun {
    DocumentCaseRun {
        case_id: id.into(),
        split: CorpusSplit::Dev,
        source: "source".into(),
        output: output.map(|value| value.to_string()),
        source_semantics: None,
        target_semantics: None,
        metrics,
        expectations: vec![],
        glossary_results: vec![],
        deterministic: true,
        runtime_ms,
        error_categories: vec![],
        provenance: None,
        document_artifacts: None,
        document_graph_artifacts: None,
        document_resolution_artifacts: None,
        document_compilation: None,
        document_translation: None,
        document_graph: None,
        document_resolution: None,
        document_error_categories: Vec::new(),
        best_effort_output: None,
    }
}

fn run(id: &str, cases: Vec<DocumentCaseRun>) -> DocumentRun {
    DocumentRun {
        run_id: id.into(),
        corpus_id: "document-v1".into(),
        profile_id: "document-mvp-v1".into(),
        git_commit: None,
        cases,
        summary: RunSummary {
            total_cases: 0,
            success_cases: 0,
            fatal_cases: 0,
            major_cases: 0,
            minor_cases: 0,
            missing_manual_reviews: 0,
        },
        provenance: None,
    }
}

#[test]
fn identical_runs_are_unchanged() {
    let baseline = run(
        "baseline",
        vec![case(
            "doc-pl-en-001",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            10,
        )],
    );
    let candidate = baseline.clone();
    let comparison = compare_runs(&baseline, &candidate);
    assert_eq!(comparison.entries[0].status, ComparisonStatus::Unchanged);
}

#[test]
fn improvement_in_translation_success_is_reported() {
    let baseline = run(
        "baseline",
        vec![case(
            "doc-pl-en-001",
            Some(""),
            metrics(0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0),
            10,
        )],
    );
    let candidate = run(
        "candidate",
        vec![case(
            "doc-pl-en-001",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            10,
        )],
    );
    let comparison = compare_runs(&baseline, &candidate);
    assert_eq!(comparison.entries[0].status, ComparisonStatus::Improved);
}

#[test]
fn improvement_in_invariant_is_reported() {
    let baseline = run(
        "baseline",
        vec![case(
            "doc-pl-en-001",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0),
            10,
        )],
    );
    let candidate = run(
        "candidate",
        vec![case(
            "doc-pl-en-001",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            10,
        )],
    );
    let comparison = compare_runs(&baseline, &candidate);
    assert_eq!(comparison.entries[0].status, ComparisonStatus::Improved);
}

#[test]
fn loss_of_invariant_is_regressed() {
    let baseline = run(
        "baseline",
        vec![case(
            "doc-pl-en-001",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            10,
        )],
    );
    let candidate = run(
        "candidate",
        vec![case(
            "doc-pl-en-001",
            Some(""),
            metrics(0.0, 0.0, 1.0, 0.8, 0.8, 0.8, 0.8, 0.0),
            10,
        )],
    );
    let comparison = compare_runs(&baseline, &candidate);
    assert_eq!(comparison.entries[0].status, ComparisonStatus::Regressed);
}

#[test]
fn better_output_without_metric_change_is_not_regressed() {
    let baseline = run(
        "baseline",
        vec![case(
            "doc-pl-en-001",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            10,
        )],
    );
    let candidate = run(
        "candidate",
        vec![case(
            "doc-pl-en-001",
            Some("B"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            10,
        )],
    );
    let comparison = compare_runs(&baseline, &candidate);
    assert_ne!(comparison.entries[0].status, ComparisonStatus::Regressed);
}

#[test]
fn missing_and_added_cases_are_reported() {
    let baseline = run(
        "baseline",
        vec![case(
            "doc-pl-en-001",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            10,
        )],
    );
    let candidate = run(
        "candidate",
        vec![case(
            "doc-pl-en-002",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            10,
        )],
    );
    let comparison = compare_runs(&baseline, &candidate);
    assert!(comparison
        .entries
        .iter()
        .any(|entry| entry.case_id == "doc-pl-en-001" && entry.status == ComparisonStatus::Missing));
    assert!(comparison
        .entries
        .iter()
        .any(|entry| entry.case_id == "doc-pl-en-002" && entry.status == ComparisonStatus::Added));
}

#[test]
fn runtime_ms_does_not_affect_regression() {
    let baseline = run(
        "baseline",
        vec![case(
            "doc-pl-en-001",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            10,
        )],
    );
    let candidate = run(
        "candidate",
        vec![case(
            "doc-pl-en-001",
            Some("A"),
            metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
            999,
        )],
    );
    let comparison = compare_runs(&baseline, &candidate);
    assert_eq!(comparison.entries[0].status, ComparisonStatus::Unchanged);
}

#[test]
fn case_order_does_not_affect_result() {
    let baseline = run(
        "baseline",
        vec![
            case(
                "doc-pl-en-001",
                Some("A"),
                metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
                10,
            ),
            case(
                "doc-pl-en-002",
                Some("A"),
                metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
                10,
            ),
        ],
    );
    let candidate = run(
        "candidate",
        vec![
            case(
                "doc-pl-en-002",
                Some("A"),
                metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
                10,
            ),
            case(
                "doc-pl-en-001",
                Some("A"),
                metrics(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
                10,
            ),
        ],
    );
    let comparison = compare_runs(&baseline, &candidate);
    assert_eq!(comparison.entries.len(), 2);
    assert!(comparison
        .entries
        .iter()
        .all(|entry| entry.status == ComparisonStatus::Unchanged));
}
