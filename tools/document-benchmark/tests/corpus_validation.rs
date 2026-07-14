use lexflex_document_benchmark::{corpus, validate};
use std::collections::BTreeSet;
use std::path::Path;

fn corpus_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/document_v1")
}

#[test]
fn test_repository_document_corpus_loads() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    assert_eq!(corpus.manifest.corpus_id, "document-v1");
    assert_eq!(corpus.cases.len(), 30);
}

#[test]
fn test_manifest_ids_are_unique() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    let ids = corpus
        .manifest
        .cases
        .iter()
        .map(|case| case.id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), corpus.manifest.cases.len());
}

#[test]
fn test_manifest_is_sorted() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    let ids = corpus
        .manifest
        .cases
        .iter()
        .map(|case| case.id.clone())
        .collect::<Vec<_>>();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
}

#[test]
fn test_case_directory_matches_id() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    for case in &corpus.cases {
        assert!(case.reference.directory.ends_with(&case.metadata.id));
        assert_eq!(case.reference.id, case.metadata.id);
    }
}

#[test]
fn test_all_case_files_exist() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    for case in &corpus.cases {
        assert!(!case.source.trim().is_empty());
        assert!(!case.references.is_empty());
        assert!(case.paths.directory.exists());
        assert!(case.paths.source.exists());
        assert!(case.paths.expectations.exists());
        assert!(case.paths.glossary.exists());
    }
}

#[test]
fn test_all_sources_are_non_empty_utf8() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    for case in &corpus.cases {
        assert!(!case.source.is_empty());
        assert!(case.source.is_char_boundary(case.source.len()));
    }
}

#[test]
fn test_all_dev_and_regression_cases_have_reference() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    for case in &corpus.cases {
        if !matches!(case.reference.split, lexflex_document_benchmark::model::CorpusSplit::Holdout) {
            assert!(!case.references.is_empty());
        }
    }
}

#[test]
fn test_all_tags_exist_in_profile() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    let report = validate::validate_corpus(&corpus);
    assert!(
        report
            .errors
            .iter()
            .all(|issue| issue.code != "unknown_tag"),
        "unexpected unknown tag errors: {:?}",
        report.errors
    );
}

#[test]
fn test_required_capability_coverage() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    let report = validate::validate_corpus(&corpus);
    assert!(report.coverage.missing_required_tags.is_empty());
    assert!(report.coverage.below_minimum.is_empty());
}

#[test]
fn test_profile_length_tiers_cover_cases() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    for case in &corpus.cases {
        assert!(corpus.profile.length_tier(&case.metadata.length_tier).is_some());
    }
}

#[test]
fn test_glossary_ids_unique() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    for case in &corpus.cases {
        let ids = case
            .glossary
            .iter()
            .map(|constraint| constraint.id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), case.glossary.len());
    }
}

#[test]
fn test_no_absolute_paths() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    assert!(!corpus.manifest.profile_path.starts_with('/'));
    for case in &corpus.manifest.cases {
        assert!(!case.directory.starts_with('/'));
    }
}

#[test]
fn test_strict_release_has_30_cases() {
    let corpus = corpus::load_corpus(&corpus_root()).unwrap();
    assert_eq!(corpus.cases.len(), 30);
}
