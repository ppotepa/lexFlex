use crate::corpus::{sha256_bytes, sha256_json, LoadedCorpus, LoadedDocumentCase};
use crate::error::BenchmarkError;
use crate::model::{CaseProvenance, DocumentRun, RunProvenance, RunSelectionProvenance};
use std::collections::{BTreeMap, BTreeSet};

pub const BENCHMARK_VERSION: &str = "1.0";
const PROVENANCE_SCHEMA_VERSION: u32 = 1;

pub fn attach_run_provenance(
    run: &mut DocumentRun,
    corpus: &LoadedCorpus,
    selection: RunSelectionProvenance,
    command: Vec<String>,
) -> Result<(), BenchmarkError> {
    let mut cases_by_id: BTreeMap<_, _> = corpus
        .cases
        .iter()
        .map(|case| (case.metadata.id.clone(), case))
        .collect();
    for case in &mut run.cases {
        let loaded = cases_by_id
            .remove(&case.case_id)
            .ok_or_else(|| BenchmarkError::CorpusInvalid(format!("missing case in corpus: {}", case.case_id)))?;
        case.provenance = Some(case_provenance(loaded));
    }
    run.provenance = Some(RunProvenance {
        schema_version: PROVENANCE_SCHEMA_VERSION,
        benchmark_version: BENCHMARK_VERSION.to_string(),
        corpus_sha256: corpus_sha256(corpus),
        manifest_sha256: sha256_json(&corpus.manifest),
        profile_sha256: sha256_json(&corpus.profile),
        command,
        selection,
    });
    Ok(())
}

pub fn verify_run_provenance(
    run: &DocumentRun,
    corpus: &LoadedCorpus,
) -> Result<(), BenchmarkError> {
    let provenance = run
        .provenance
        .as_ref()
        .ok_or_else(|| BenchmarkError::FreezeRefused)?;
    if provenance.schema_version != PROVENANCE_SCHEMA_VERSION {
        return Err(BenchmarkError::CorpusInvalid(
            "unsupported provenance schema version".into(),
        ));
    }
    if provenance.benchmark_version != BENCHMARK_VERSION {
        return Err(BenchmarkError::CorpusInvalid(
            "unexpected benchmark version".into(),
        ));
    }
    if provenance.corpus_sha256 != corpus_sha256(corpus)
        || provenance.manifest_sha256 != sha256_json(&corpus.manifest)
        || provenance.profile_sha256 != sha256_json(&corpus.profile)
    {
        return Err(BenchmarkError::CorpusInvalid(
            "provenance hash mismatch".into(),
        ));
    }
    verify_case_set(run, corpus)?;
    if !provenance.selection.all_enabled_cases
        || provenance.selection.split.is_some()
        || !provenance.selection.case_filter.is_empty()
        || provenance.selection.repeat_count < 2
    {
        return Err(BenchmarkError::FreezeRefused);
    }
    let case_map: BTreeMap<_, _> = corpus
        .cases
        .iter()
        .map(|case| (case.metadata.id.clone(), case))
        .collect();
    for case in &run.cases {
        let loaded = case_map
            .get(&case.case_id)
            .ok_or_else(|| BenchmarkError::CorpusInvalid(format!("missing case: {}", case.case_id)))?;
        let expected = case_provenance(loaded);
        if case.provenance.as_ref() != Some(&expected) {
            return Err(BenchmarkError::CorpusInvalid(format!(
                "case provenance mismatch: {}",
                case.case_id
            )));
        }
    }
    Ok(())
}

pub fn case_provenance(case: &LoadedDocumentCase) -> CaseProvenance {
    CaseProvenance {
        source_sha256: sha256_bytes(case.source.as_bytes()),
        reference_sha256: case
            .references
            .iter()
            .map(|reference| sha256_bytes(reference.as_bytes()))
            .collect::<Vec<_>>(),
        metadata_sha256: sha256_json(&case.metadata),
        expectations_sha256: sha256_json(&case.expectations),
        glossary_sha256: Some(sha256_json(&case.glossary)),
    }
}

fn verify_case_set(run: &DocumentRun, corpus: &LoadedCorpus) -> Result<(), BenchmarkError> {
    let expected: BTreeSet<_> = corpus
        .cases
        .iter()
        .filter(|case| case.reference.enabled)
        .map(|case| case.metadata.id.clone())
        .collect();
    let mut actual = BTreeSet::new();
    for case in &run.cases {
        if !actual.insert(case.case_id.clone()) {
            return Err(BenchmarkError::CorpusInvalid(format!(
                "duplicate run case id: {}",
                case.case_id
            )));
        }
    }
    if actual != expected {
        let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
        let unexpected = actual.difference(&expected).cloned().collect::<Vec<_>>();
        return Err(BenchmarkError::CorpusInvalid(format!(
            "run case set mismatch: missing={missing:?} unexpected={unexpected:?}"
        )));
    }
    Ok(())
}

fn corpus_sha256(corpus: &LoadedCorpus) -> String {
    let mut chunks = vec![
        sha256_json(&corpus.manifest),
        sha256_json(&corpus.profile),
    ];
    let mut cases = corpus.cases.iter().collect::<Vec<_>>();
    cases.sort_by(|a, b| a.metadata.id.cmp(&b.metadata.id));
    for case in cases {
        let provenance = case_provenance(case);
        chunks.push(provenance.source_sha256);
        chunks.extend(provenance.reference_sha256);
        chunks.push(provenance.metadata_sha256);
        chunks.push(provenance.expectations_sha256);
        chunks.push(provenance.glossary_sha256.unwrap_or_default());
    }
    sha256_join(&chunks)
}

fn sha256_join(parts: &[String]) -> String {
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0u8]);
    }
    format!("{:x}", hasher.finalize())
}
