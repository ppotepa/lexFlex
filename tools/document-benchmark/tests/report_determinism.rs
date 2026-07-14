use lexflex_document_benchmark::model::{
    ComparisonStatus, CorpusSplit, DocumentCaseRun, DocumentMetrics, DocumentRun, RunSummary,
};
use lexflex_document_benchmark::report::{summary_text, write_run_report};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn run(case_id: &str, output: Option<&str>) -> DocumentRun {
    DocumentRun {
        run_id: "run-001".into(),
        corpus_id: "document-v1".into(),
        profile_id: "document-mvp-v1".into(),
        git_commit: Some("deadbeef".into()),
        cases: vec![DocumentCaseRun {
            case_id: case_id.into(),
            split: CorpusSplit::Dev,
            source: "source".into(),
            output: output.map(|value| value.to_string()),
            source_semantics: None,
            target_semantics: None,
            metrics: DocumentMetrics {
                technical: BTreeMap::new(),
                structure: BTreeMap::new(),
                semantics: BTreeMap::new(),
                glossary: BTreeMap::new(),
                expectations: BTreeMap::new(),
                reference: BTreeMap::new(),
            },
            expectations: vec![],
            glossary_results: vec![],
            deterministic: true,
            runtime_ms: 10,
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
        }],
        summary: RunSummary {
            total_cases: 1,
            success_cases: 1,
            fatal_cases: 0,
            major_cases: 0,
            minor_cases: 0,
            missing_manual_reviews: 1,
        },
        provenance: None,
    }
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lexflex-document-benchmark-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn report_files_are_deterministic() {
    let run = run("doc-pl-en-001", Some("Hello"));
    let left = temp_dir("left");
    let right = temp_dir("right");
    write_run_report(&left, &run).unwrap();
    write_run_report(&right, &run).unwrap();
    assert_eq!(
        fs::read_to_string(left.join("run.json")).unwrap(),
        fs::read_to_string(right.join("run.json")).unwrap()
    );
    assert_eq!(
        fs::read_to_string(left.join("summary.txt")).unwrap(),
        fs::read_to_string(right.join("summary.txt")).unwrap()
    );
    assert!(summary_text(&run).contains("BASELINE, NOT QUALITY CLAIM"));
    let _ = ComparisonStatus::Unchanged;
}
