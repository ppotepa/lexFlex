use crate::corpus::LoadedCorpus;
use crate::model::{CapabilityCoverage, CapabilityCoverageDeficit, CorpusSplit};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct CorpusValidationIssue {
    pub case_id: Option<String>,
    pub path: Option<PathBuf>,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CorpusValidationReport {
    pub errors: Vec<CorpusValidationIssue>,
    pub warnings: Vec<CorpusValidationIssue>,
    pub coverage: CapabilityCoverage,
}

pub fn validate_corpus(corpus: &LoadedCorpus) -> CorpusValidationReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut counts_by_tag: BTreeMap<String, usize> = BTreeMap::new();
    let mut seen_ids = BTreeSet::new();
    let mut seen_dirs = BTreeSet::new();
    let mut seen_hashes = HashSet::new();
    let mut source_hashes = HashMap::new();
    let mut required_tags = BTreeMap::new();

    for cap in &corpus.profile.capabilities {
        required_tags.insert(cap.corpus_tag.clone(), cap.minimum_cases);
    }

    if let Err(errs) = corpus.profile.validate() {
        for err in errs {
            errors.push(issue(None, None, "profile", err.to_string()));
        }
    }

    let mut previous_id: Option<&str> = None;
    for case in &corpus.cases {
        let id = &case.metadata.id;
        if !seen_ids.insert(id.clone()) {
            errors.push(issue(
                Some(id.clone()),
                Some(case.paths.directory.clone()),
                "duplicate_id",
                "duplicate case id",
            ));
        }
        if let Some(prev) = previous_id {
            if prev > id.as_str() {
                errors.push(issue(
                    Some(id.clone()),
                    Some(case.paths.directory.clone()),
                    "unsorted_manifest",
                    "manifest is not sorted by id",
                ));
            }
        }
        previous_id = Some(id);

        if !seen_dirs.insert(case.reference.directory.clone()) {
            errors.push(issue(
                Some(id.clone()),
                Some(case.paths.directory.clone()),
                "duplicate_directory",
                "duplicate case directory",
            ));
        }

        if case.reference.id != case.metadata.id {
            errors.push(issue(
                Some(id.clone()),
                Some(case.paths.directory.clone()),
                "id_mismatch",
                "manifest id and case id differ",
            ));
        }

        if case.metadata.schema_version != 1 {
            errors.push(issue(
                Some(id.clone()),
                Some(case.paths.directory.clone()),
                "schema_version",
                "unsupported case schema version",
            ));
        }

        if !(1..=5).contains(&case.metadata.difficulty) {
            errors.push(issue(
                Some(id.clone()),
                Some(case.paths.directory.clone()),
                "difficulty",
                "difficulty must be 1..=5",
            ));
        }

        if case.metadata.source_language != corpus.profile.source_language
            || case.metadata.target_language != corpus.profile.target_language
        {
            errors.push(issue(
                Some(id.clone()),
                Some(case.paths.directory.clone()),
                "language",
                "case languages must match profile",
            ));
        }

        if corpus.profile.length_tier(&case.metadata.length_tier).is_none() {
            errors.push(issue(
                Some(id.clone()),
                Some(case.paths.directory.clone()),
                "length_tier",
                "length tier missing from profile",
            ));
        }

        for tag in &case.metadata.tags {
            if !corpus
                .profile
                .capabilities
                .iter()
                .any(|cap| cap.corpus_tag == *tag)
            {
                errors.push(issue(
                    Some(id.clone()),
                    Some(case.paths.directory.clone()),
                    "unknown_tag",
                    format!("unknown capability tag: {tag}"),
                ));
            }
            *counts_by_tag.entry(tag.clone()).or_insert(0) += 1;
        }

        if case.source.trim().is_empty() {
            errors.push(issue(
                Some(id.clone()),
                Some(case.paths.source.clone()),
                "empty_source",
                "source text must not be empty",
            ));
        }

        if case.references.is_empty() {
            errors.push(issue(
                Some(id.clone()),
                Some(case.paths.directory.clone()),
                "references",
                "reference files must not be empty",
            ));
        }

        if !case.reference.blocking && matches!(case.reference.split, CorpusSplit::Holdout) {
            warnings.push(issue(
                Some(id.clone()),
                Some(case.paths.directory.clone()),
                "holdout",
                "holdout case should still be tracked",
            ));
        }

        let source_bytes = case.source.as_bytes().to_vec();
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        hasher.update(&source_bytes);
        let hash = format!("{:x}", hasher.finalize());
        if !seen_hashes.insert(hash.clone()) {
            warnings.push(issue(
                Some(id.clone()),
                Some(case.paths.source.clone()),
                "duplicate_source_hash",
                format!("duplicate source hash: {hash}"),
            ));
        }
        source_hashes.insert(id.clone(), hash);

        if !case.source.ends_with('\n') {
            warnings.push(issue(
                Some(id.clone()),
                Some(case.paths.source.clone()),
                "missing_trailing_newline",
                "source missing trailing newline",
            ));
        }
        for (i, reference) in case.references.iter().enumerate() {
            if !reference.ends_with('\n') {
                warnings.push(issue(
                    Some(id.clone()),
                    case.paths.references.get(i).cloned(),
                    "missing_trailing_newline",
                    "reference missing trailing newline",
                ));
            }
        }

        if let Some(expected) = case.expectations.expected_paragraph_count {
            let actual = source_paragraph_count(&case.source);
            if actual != expected {
                errors.push(issue(
                    Some(id.clone()),
                    Some(case.paths.expectations.clone()),
                    "paragraph_count",
                    "expected paragraph count does not match source",
                ));
            }
        }
    }

    let mut missing_required_tags = Vec::new();
    let mut below_minimum = Vec::new();
    for (tag, required) in required_tags {
        let found = counts_by_tag.get(&tag).copied().unwrap_or_default();
        if found == 0 && required > 0 {
            missing_required_tags.push(tag.clone());
        }
        if found < required {
            below_minimum.push(CapabilityCoverageDeficit {
                tag,
                found,
                required,
            });
        }
    }

    CorpusValidationReport {
        errors,
        warnings,
        coverage: CapabilityCoverage {
            counts_by_tag,
            missing_required_tags,
            below_minimum,
        },
    }
}

fn issue(
    case_id: Option<String>,
    path: Option<PathBuf>,
    code: impl Into<String>,
    message: impl Into<String>,
) -> CorpusValidationIssue {
    CorpusValidationIssue {
        case_id,
        path,
        code: code.into(),
        message: message.into(),
    }
}

fn source_paragraph_count(text: &str) -> usize {
    text.split("\n\n").filter(|part| !part.trim().is_empty()).count()
}
