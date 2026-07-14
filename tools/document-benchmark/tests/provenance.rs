use lexflex_document_benchmark::corpus::load_corpus;
use lexflex_document_benchmark::freeze::{freeze, FreezeOptions};
use lexflex_document_benchmark::model::{
    DocumentCaseRun, DocumentMetrics, DocumentRun, MetricStatus, MetricValue, RunSelectionProvenance,
    RunSummary,
};
use lexflex_document_benchmark::provenance::{attach_run_provenance, verify_run_provenance};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/document_v1")
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

fn empty_metrics() -> DocumentMetrics {
    DocumentMetrics {
        technical: BTreeMap::from([("deterministic".into(), metric(1.0))]),
        structure: BTreeMap::new(),
        semantics: BTreeMap::new(),
        glossary: BTreeMap::new(),
        expectations: BTreeMap::new(),
        reference: BTreeMap::new(),
    }
}

fn metric(value: f64) -> MetricValue {
    MetricValue {
        value: Some(value),
        numerator: Some(value),
        denominator: Some(1.0),
        status: MetricStatus::Exact,
    }
}

fn build_run() -> (lexflex_document_benchmark::corpus::LoadedCorpus, DocumentRun) {
    let corpus = load_corpus(&corpus_root()).unwrap();
    let cases = corpus
        .cases
        .iter()
        .map(|case| DocumentCaseRun {
            case_id: case.metadata.id.clone(),
            split: case.reference.split,
            source: case.source.clone(),
            output: Some(case.source.clone()),
            source_semantics: None,
            target_semantics: None,
            metrics: empty_metrics(),
            expectations: vec![],
            glossary_results: vec![],
            deterministic: true,
            runtime_ms: 1,
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
        })
        .collect::<Vec<_>>();
    let mut run = DocumentRun {
        run_id: "run".into(),
        corpus_id: corpus.manifest.corpus_id.clone(),
        profile_id: corpus.profile.id.clone(),
        git_commit: None,
        cases,
        summary: RunSummary {
            total_cases: corpus.cases.len(),
            success_cases: corpus.cases.len(),
            fatal_cases: 0,
            major_cases: 0,
            minor_cases: 0,
            missing_manual_reviews: corpus.cases.len(),
        },
        provenance: None,
    };
    attach_run_provenance(
        &mut run,
        &corpus,
        RunSelectionProvenance {
            repeat_count: 2,
            split: None,
            case_filter: Vec::new(),
            all_enabled_cases: true,
        },
        vec!["lexflex-document-benchmark".into(), "run".into()],
    )
    .unwrap();
    (corpus, run)
}

#[test]
fn identical_corpus_passes_provenance_verification() {
    let (corpus, run) = build_run();
    verify_run_provenance(&run, &corpus).unwrap();
}

#[test]
fn changing_source_breaks_provenance_hash() {
    let (mut corpus, run) = build_run();
    corpus.cases[0].source.push('X');
    assert!(verify_run_provenance(&run, &corpus).is_err());
}

#[test]
fn missing_provenance_blocks_freeze() {
    let (_corpus, mut run) = build_run();
    run.provenance = None;
    let dir = temp_dir("missing-provenance");
    let run_path = dir.join("run.json");
    let output_path = dir.join("baseline.json");
    fs::write(&run_path, serde_json::to_string_pretty(&run).unwrap()).unwrap();
    let result = freeze(FreezeOptions {
        run: &run_path,
        corpus: &corpus_root(),
        output: &output_path,
        confirm: "document-v1",
        force: false,
    });
    assert!(result.is_err());
    assert!(!output_path.exists());
}

#[test]
fn existing_output_without_force_is_not_overwritten() {
    let (_corpus, run) = build_run();
    let dir = temp_dir("no-force");
    let run_path = dir.join("run.json");
    let output_path = dir.join("baseline.json");
    fs::write(&run_path, serde_json::to_string_pretty(&run).unwrap()).unwrap();
    fs::write(&output_path, "sentinel").unwrap();
    let result = freeze(FreezeOptions {
        run: &run_path,
        corpus: &corpus_root(),
        output: &output_path,
        confirm: "document-v1",
        force: false,
    });
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(&output_path).unwrap(), "sentinel");
}
