use crate::model::{
    BaselineComparison, BaselineComparisonEntry, ComparisonStatus, DocumentCaseRun, DocumentRun,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn compare_runs(baseline: &DocumentRun, candidate: &DocumentRun) -> BaselineComparison {
    let baseline_cases: BTreeMap<_, _> = baseline
        .cases
        .iter()
        .map(|case| (case.case_id.clone(), case))
        .collect();
    let candidate_cases: BTreeMap<_, _> = candidate
        .cases
        .iter()
        .map(|case| (case.case_id.clone(), case))
        .collect();
    let mut entries = Vec::new();
    let mut critical_regressions = 0;
    let mut ids = BTreeSet::new();
    ids.extend(baseline_cases.keys().cloned());
    ids.extend(candidate_cases.keys().cloned());

    for id in ids {
        let status = match (baseline_cases.get(&id), candidate_cases.get(&id)) {
            (Some(a), Some(b)) => compare_case(a, b),
            (Some(_), None) => ComparisonStatus::Missing,
            (None, Some(_)) => ComparisonStatus::Added,
            (None, None) => ComparisonStatus::Unchanged,
        };
        if matches!(status, ComparisonStatus::Regressed | ComparisonStatus::Missing) {
            critical_regressions += 1;
        }
        entries.push(BaselineComparisonEntry {
            case_id: id,
            status,
            note: None,
        });
    }

    BaselineComparison {
        baseline_run_id: baseline.run_id.clone(),
        candidate_run_id: candidate.run_id.clone(),
        entries,
        critical_regressions,
    }
}

fn compare_case(baseline: &DocumentCaseRun, candidate: &DocumentCaseRun) -> ComparisonStatus {
    if same_case_legacy(baseline, candidate) {
        return ComparisonStatus::Unchanged;
    }

    let mut better = false;
    let mut worse = false;

    compare_bool(
        baseline,
        candidate,
        "translation_success",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );
    compare_bool(
        baseline,
        candidate,
        "output_non_empty",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );
    compare_bool(
        baseline,
        candidate,
        "parse_source_success",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );
    compare_bool(
        baseline,
        candidate,
        "parse_target_success",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );
    compare_bool(
        baseline,
        candidate,
        "deterministic",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );

    compare_higher_is_better(
        baseline,
        candidate,
        "paragraph_preservation_ratio",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "sentence_count_ratio",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "frame_type_recall",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "verb_concept_recall",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "role_signature_recall",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "proper_name_recall",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "polarity_preservation",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "tense_preservation",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "quantification_preservation",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "temporal_preservation",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "glossary_compliance",
        &mut better,
        &mut worse,
    );
    compare_higher_is_better(
        baseline,
        candidate,
        "expectation_passed",
        &mut better,
        &mut worse,
    );
    compare_bool(
        baseline,
        candidate,
        "document_compile_success",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );
    compare_bool(
        baseline,
        candidate,
        "document_best_effort_success",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );
    compare_bool(
        baseline,
        candidate,
        "document_deterministic",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );
    compare_bool(
        baseline,
        candidate,
        "document_compilation_valid",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );
    compare_bool(
        baseline,
        candidate,
        "document_translation_valid",
        |lhs, rhs| rhs && !lhs,
        &mut better,
        &mut worse,
    );
    compare_lower_is_better(
        baseline,
        candidate,
        "silent_drop_count",
        &mut better,
        &mut worse,
    );
    compare_lower_is_better(
        baseline,
        candidate,
        "pending_result_count",
        &mut better,
        &mut worse,
    );
    compare_lower_is_better(
        baseline,
        candidate,
        "placeholder_count",
        &mut better,
        &mut worse,
    );

    if worse && !better {
        ComparisonStatus::Regressed
    } else if better && !worse {
        ComparisonStatus::Improved
    } else {
        ComparisonStatus::Unchanged
    }
}

fn compare_bool<F>(
    baseline: &DocumentCaseRun,
    candidate: &DocumentCaseRun,
    key: &str,
    better_when: F,
    better: &mut bool,
    worse: &mut bool,
) where
    F: Fn(bool, bool) -> bool,
{
    let lhs = metric_bool(baseline, key);
    let rhs = metric_bool(candidate, key);
    if better_when(lhs, rhs) {
        *better = true;
    } else if better_when(rhs, lhs) {
        *worse = true;
    }
}

fn compare_higher_is_better(
    baseline: &DocumentCaseRun,
    candidate: &DocumentCaseRun,
    key: &str,
    better: &mut bool,
    worse: &mut bool,
) {
    let lhs = metric_value(baseline, key);
    let rhs = metric_value(candidate, key);
    match (lhs, rhs) {
        (Some(l), Some(r)) if r > l + 1e-9 => *better = true,
        (Some(l), Some(r)) if r + 1e-9 < l => *worse = true,
        (None, Some(_)) => *better = true,
        (Some(_), None) => *worse = true,
        _ => {}
    }
}

fn compare_lower_is_better(
    baseline: &DocumentCaseRun,
    candidate: &DocumentCaseRun,
    key: &str,
    better: &mut bool,
    worse: &mut bool,
) {
    let lhs = metric_value(baseline, key);
    let rhs = metric_value(candidate, key);
    match (lhs, rhs) {
        (Some(l), Some(r)) if r + 1e-9 < l => *better = true,
        (Some(l), Some(r)) if r > l + 1e-9 => *worse = true,
        (None, Some(_)) => *better = true,
        (Some(_), None) => *worse = true,
        _ => {}
    }
}

fn metric_bool(case: &DocumentCaseRun, key: &str) -> bool {
    metric_value(case, key).unwrap_or(0.0) > 0.5
}

fn metric_value(case: &DocumentCaseRun, key: &str) -> Option<f64> {
    metric_lookup(&case.metrics.technical, key)
        .or_else(|| metric_lookup(&case.metrics.structure, key))
        .or_else(|| metric_lookup(&case.metrics.semantics, key))
        .or_else(|| metric_lookup(&case.metrics.glossary, key))
        .or_else(|| metric_lookup(&case.metrics.expectations, key))
        .or_else(|| metric_lookup(&case.metrics.reference, key))
}

fn metric_lookup(map: &BTreeMap<String, crate::model::MetricValue>, key: &str) -> Option<f64> {
    map.get(key).and_then(|metric| metric.value)
}

fn same_case_legacy(baseline: &DocumentCaseRun, candidate: &DocumentCaseRun) -> bool {
    baseline.case_id == candidate.case_id
        && baseline.split == candidate.split
        && baseline.source == candidate.source
        && baseline.output == candidate.output
        && baseline.source_semantics == candidate.source_semantics
        && baseline.target_semantics == candidate.target_semantics
        && baseline.metrics == candidate.metrics
        && baseline.expectations == candidate.expectations
        && baseline.glossary_results == candidate.glossary_results
        && baseline.deterministic == candidate.deterministic
        && baseline.error_categories == candidate.error_categories
        && baseline.provenance == candidate.provenance
}
