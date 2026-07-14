use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Manifest {
    id: String,
    schema: u32,
    sources: Vec<ManifestSource>,
    required_question_count: usize,
    snapshot_only: bool,
}

#[derive(Debug, Deserialize)]
struct ManifestSource { language: String, title: String, path: String }

#[derive(Debug, Deserialize)]
struct Snapshot {
    source_id: String,
    title: String,
    language: String,
    uri: Option<String>,
    revision: Option<String>,
    fetched_at: Option<String>,
    content_sha256: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct Question {
    id: String,
    language: String,
    text: String,
    expected_predicate: String,
    expected_status: String,
    evidence_required: bool,
}

#[test]
fn paris_corpus_is_self_consistent_and_snapshot_only() {
    let manifest: Manifest = ron::from_str(&fs::read_to_string("benchmarks/wikipedia_paris_v1/manifest.ron").unwrap()).unwrap();
    assert_eq!(manifest.id, "wikipedia_paris_v1");
    assert_eq!(manifest.schema, 1);
    assert!(manifest.snapshot_only);
    assert_eq!(manifest.required_question_count, 10);
    for source in &manifest.sources {
        let snapshot: Snapshot = serde_json::from_str(&fs::read_to_string(Path::new("benchmarks/wikipedia_paris_v1").join(&source.path)).unwrap()).unwrap();
        assert_eq!(snapshot.language, source.language);
        assert_eq!(snapshot.title, source.title);
        assert!(snapshot.uri.as_deref().is_some_and(|uri| uri.starts_with("https://")));
        assert!(snapshot.revision.as_deref().is_some_and(|revision| !revision.is_empty()));
        assert!(snapshot.fetched_at.is_none(), "controlled snapshots must not contain runtime fetch time");
        let mut hash = Sha256::new();
        hash.update(snapshot.text.as_bytes());
        let computed = format!("{:x}", hash.finalize());
        assert_eq!(snapshot.content_sha256.len(), computed.len());
        assert_eq!(snapshot.content_sha256.as_bytes(), computed.as_bytes(), "hash mismatch for {}", source.path);
        assert!(!snapshot.source_id.is_empty() && snapshot.text.len() > 10_000);
    }
    let questions: Vec<Question> = ron::from_str(&fs::read_to_string("benchmarks/wikipedia_paris_v1/questions.ron").unwrap()).unwrap();
    assert_eq!(questions.len(), manifest.required_question_count);
    let mut ids = questions.iter().map(|question| question.id.as_str()).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), questions.len());
    assert!(questions.iter().all(|question| ["en", "pl"].contains(&question.language.as_str()) && !question.text.is_empty() && !question.expected_predicate.is_empty() && !question.expected_status.is_empty() && question.evidence_required));
}
