use crate::corpus::load_corpus;
use crate::error::BenchmarkError;
use crate::provenance::verify_run_provenance;
use crate::validate::validate_corpus;
use crate::model::{DocumentCaseRun, DocumentRun, RunSummary};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct FreezeOptions<'a> {
    pub run: &'a Path,
    pub corpus: &'a Path,
    pub output: &'a Path,
    pub confirm: &'a str,
    pub force: bool,
}

pub fn freeze(options: FreezeOptions<'_>) -> Result<DocumentRun, BenchmarkError> {
    if options.confirm != "document-v1" {
        return Err(BenchmarkError::FreezeRefused);
    }
    if options.output.exists() && !options.force {
        return Err(BenchmarkError::FreezeRefused);
    }

    let corpus = load_corpus(options.corpus)?;
    let validation = validate_corpus(&corpus);
    if !validation.errors.is_empty() {
        return Err(BenchmarkError::CorpusInvalid("corpus validation failed".into()));
    }
    let enabled_cases = corpus.cases.iter().filter(|case| case.reference.enabled).count();
    if enabled_cases < 30 {
        return Err(BenchmarkError::CorpusInvalid(
            "strict release requires 30 enabled cases".into(),
        ));
    }

    let mut run: DocumentRun = serde_json::from_str(&fs::read_to_string(options.run).map_err(|source| {
        BenchmarkError::Io {
            path: options.run.display().to_string(),
            source,
        }
    })?)?;
    verify_run_provenance(&run, &corpus)?;
    if run.corpus_id != corpus.manifest.corpus_id || run.profile_id != corpus.profile.id {
        return Err(BenchmarkError::CorpusInvalid(
            "run corpus/profile id mismatch".into(),
        ));
    }

    for case in &mut run.cases {
        case.runtime_ms = 0;
    }
    run.cases.sort_by(|a, b| a.case_id.cmp(&b.case_id));
    run.summary = summarize_run(&run.cases);

    atomic_write(options.output, &serde_json::to_string_pretty(&run)?)?;
    Ok(run)
}

fn atomic_write(path: &Path, contents: &str) -> Result<(), BenchmarkError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| BenchmarkError::Io {
        path: parent.display().to_string(),
        source,
    })?;
    let temp_path = parent.join(format!(
        ".{}.tmp",
        path.file_name().and_then(|name| name.to_str()).unwrap_or("baseline")
    ));
    fs::write(&temp_path, contents).map_err(|source| BenchmarkError::Io {
        path: temp_path.display().to_string(),
        source,
    })?;
    fs::rename(&temp_path, path).map_err(|source| BenchmarkError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn summarize_run(cases: &[DocumentCaseRun]) -> RunSummary {
    let total_cases = cases.len();
    let success_cases = cases
        .iter()
        .filter(|case| case.output.as_deref().map(|o| !o.trim().is_empty()).unwrap_or(false))
        .count();
    let fatal_cases = cases
        .iter()
        .filter(|case| {
            case.error_categories
                .iter()
                .any(|error| error.starts_with("translation_error") || error.starts_with("parse_source_error"))
        })
        .count();
    let major_cases = cases
        .iter()
        .filter(|case| !case.expectations.iter().all(|result| result.passed))
        .count();
    let minor_cases = cases
        .iter()
        .filter(|case| case.glossary_results.iter().any(|result| !result.passed))
        .count();
    RunSummary {
        total_cases,
        success_cases,
        fatal_cases,
        major_cases,
        minor_cases,
        missing_manual_reviews: total_cases,
    }
}
