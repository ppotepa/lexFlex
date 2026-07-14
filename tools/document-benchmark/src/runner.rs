use crate::canonical::canonicalize_utterance;
use crate::corpus::{LoadedCorpus, LoadedDocumentCase};
use crate::error::BenchmarkError;
use crate::canonical::canonicalize_document_graph;
use crate::document_adapter::canonicalize_document_artifacts;
use crate::expectations::evaluate_expectations;
use crate::glossary::evaluate_glossary;
use crate::metrics::{build_metrics, MetricInputs};
use crate::model::{
    CorpusSplit, DocumentCaseRun, DocumentRun, MetricStatus, MetricValue, RunSummary,
};
use crate::runner_resolution::run_document_resolution_pipeline;
use lexflex::api::LexFlexAPI;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub split: Option<CorpusSplit>,
    pub case_ids: Vec<String>,
    pub repeat_count: usize,
    pub fail_fast: bool,
    pub include_holdout_reference: bool,
}

pub struct BenchmarkRunner {
    api: LexFlexAPI,
}

impl BenchmarkRunner {
    pub fn new(data_dir: &Path) -> Result<Self, BenchmarkError> {
        let api = LexFlexAPI::builder()
            .data_dir(&data_dir.display().to_string())
            .build()
            .map_err(|e| BenchmarkError::message(e.to_string()))?;
        Ok(Self { api })
    }

    pub fn run(&self, corpus: &LoadedCorpus, options: &RunOptions) -> DocumentRun {
        let _ = options.include_holdout_reference;
        let mut cases = Vec::new();
        for case in &corpus.cases {
            if options
                .split
                .map(|split| split != case.reference.split)
                .unwrap_or(false)
            {
                continue;
            }
            if !options.case_ids.is_empty() && !options.case_ids.contains(&case.metadata.id) {
                continue;
            }
            let run = self.run_case(case, options);
            if options.fail_fast && has_fatal(&run) {
                cases.push(run);
                break;
            }
            cases.push(run);
        }
        let summary = summarize(&cases);
        DocumentRun {
            run_id: "run".into(),
            corpus_id: corpus.manifest.corpus_id.clone(),
            profile_id: corpus.profile.id.clone(),
            git_commit: git_commit(),
            cases,
            summary,
            provenance: None,
        }
    }

    pub fn run_case(&self, case: &LoadedDocumentCase, options: &RunOptions) -> DocumentCaseRun {
        let started = Instant::now();
        let mut source_semantics = None;
        let mut target_semantics = None;
        let mut error_categories = Vec::new();
        let mut attempts = Vec::new();
        let mut document_deterministic = false;

        match self
            .api
            .parse_multi_sentence(&case.source, &case.metadata.source_language)
        {
            Ok(utterance) => {
                source_semantics = Some(canonicalize_utterance(&utterance));
            }
            Err(err) => {
                error_categories.push(format!("parse_source_error:{err}"));
            }
        }

        for _ in 0..options.repeat_count.max(1) {
            match self.api.translate(
                &case.source,
                &case.metadata.source_language,
                &case.metadata.target_language,
            ) {
                Ok(text) => attempts.push(AttemptOutcome { ok: true, text }),
                Err(err) => {
                    error_categories.push(format!("translation_error:{err}"));
                    attempts.push(AttemptOutcome {
                        ok: false,
                        text: String::new(),
                    });
                }
            }
        }

        let deterministic = attempts.windows(2).all(|pair| pair[0] == pair[1]);
        let output = attempts
            .iter()
            .find(|attempt| attempt.ok)
            .map(|attempt| attempt.text.clone());

        if let Some(text) = output.as_deref().filter(|text| !text.trim().is_empty()) {
            if let Ok(utterance) = self
                .api
                .parse_multi_sentence(text, &case.metadata.target_language)
            {
                target_semantics = Some(canonicalize_utterance(&utterance));
            } else {
                error_categories.push("parse_target_error".into());
            }
        }

        let expectations = evaluate_expectations(
            case,
            output.as_deref(),
            source_semantics.as_ref(),
            target_semantics.as_ref(),
        );
        let glossary_results = evaluate_glossary(case, output.as_deref());
        let metrics = build_metrics(MetricInputs {
            source: &case.source,
            output: output.as_deref(),
            source_semantics: source_semantics.as_ref(),
            target_semantics: target_semantics.as_ref(),
            deterministic,
            translation_error: error_categories.iter().any(|e| e.starts_with("translation_error")),
            references: &case.references,
            glossary_results: &glossary_results,
            expectation_results: &expectations,
            runtime_ms: started.elapsed().as_millis(),
        });

        let mut document_error_categories = Vec::new();
        let mut document_artifacts = None;
        let mut document_graph_artifacts = None;
        let mut document_resolution_artifacts = None;
        let mut best_effort_output = None;
        let mut document_compilation = None;
        let mut document_translation = None;
        let mut document_graph = None;
        let mut document_resolution = None;
        match self
            .api
            .compile_document(&case.source, &case.metadata.source_language)
        {
            Ok(compilation) => {
                document_compilation = Some(compilation.clone());
                let mut runs = Vec::new();
                for _ in 0..options.repeat_count.max(1) {
                    match self.api.translate_document_best_effort(
                        &case.source,
                        &case.metadata.source_language,
                        &case.metadata.target_language,
                    ) {
                        Ok(value) => runs.push(value),
                        Err(error) => document_error_categories.push(error.to_string()),
                    }
                }
                document_deterministic = runs.first().map(|first| {
                    runs.iter().all(|candidate| {
                        candidate.output == first.output
                            && candidate.translation_sha256 == first.translation_sha256
                    })
                }).unwrap_or(false);
                best_effort_output = runs.first().map(|x| x.output.clone());
                document_translation = runs.first().cloned();
                match self.api.build_document_graph(&compilation) {
                    Ok(graph) => {
                        let repeat_graph = self.api.build_document_graph(&compilation);
                        let deterministic_graph = repeat_graph
                            .as_ref()
                            .map(|other| other.graph_sha256 == graph.graph_sha256 && other == &graph)
                            .unwrap_or(false);
                        document_graph_artifacts = Some(canonicalize_document_graph(
                            &compilation,
                            &graph,
                            deterministic_graph,
                        ));
                        let resolution_run = run_document_resolution_pipeline(
                            &self.api,
                            &graph,
                            options.repeat_count,
                        );
                        document_deterministic &= resolution_run.deterministic;
                        document_resolution_artifacts = resolution_run.canonical;
                        document_resolution = resolution_run.resolution;
                        document_error_categories.extend(resolution_run.error_categories);
                        document_graph = Some(graph);
                    }
                    Err(error) => {
                        document_error_categories.push(error.to_string());
                    }
                }
                document_artifacts = Some(canonicalize_document_artifacts(
                    &compilation,
                    runs.first(),
                    document_deterministic,
                    document_graph_artifacts.as_ref(),
                ));
            }
            Err(error) => {
                document_error_categories.push(error.to_string());
            }
        }

        let mut metrics = metrics;
        apply_document_metrics(
            &mut metrics,
            document_artifacts.as_ref(),
            document_deterministic,
        );

        error_categories.sort();
        error_categories.dedup();

        DocumentCaseRun {
            case_id: case.metadata.id.clone(),
            split: case.reference.split,
            source: case.source.clone(),
            output,
            source_semantics,
            target_semantics,
            metrics,
            expectations,
            glossary_results,
            deterministic,
            runtime_ms: started.elapsed().as_millis(),
            error_categories,
            provenance: None,
            document_artifacts,
            document_graph_artifacts,
            document_compilation,
            document_translation,
            document_graph,
            document_resolution,
            document_error_categories,
            best_effort_output,
            document_resolution_artifacts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttemptOutcome {
    ok: bool,
    text: String,
}

fn summarize(cases: &[DocumentCaseRun]) -> RunSummary {
    let total_cases = cases.len();
    let success_cases = cases
        .iter()
        .filter(|case| metric_bool(case, "translation_success") && metric_bool(case, "output_non_empty"))
        .count();
    let fatal_cases = cases.iter().filter(|case| has_fatal(case)).count();
    let major_cases = cases
        .iter()
        .filter(|case| !has_fatal(case) && case.expectations.iter().any(|result| !result.passed))
        .count();
    let minor_cases = cases
        .iter()
        .filter(|case| case.expectations.iter().any(|result| !result.passed && result.severity == "minor"))
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

fn has_fatal(case: &DocumentCaseRun) -> bool {
    case.error_categories.iter().any(|error| {
        error.starts_with("panic")
            || error.starts_with("translation_error")
            || error.starts_with("parse_source_error")
    }) || !metric_bool(case, "output_non_empty")
}

fn metric_bool(case: &DocumentCaseRun, key: &str) -> bool {
    metric_value(&case.metrics.technical, key)
        .or_else(|| metric_value(&case.metrics.structure, key))
        .or_else(|| metric_value(&case.metrics.semantics, key))
        .or_else(|| metric_value(&case.metrics.glossary, key))
        .or_else(|| metric_value(&case.metrics.expectations, key))
        .or_else(|| metric_value(&case.metrics.reference, key))
        .unwrap_or(0.0)
        > 0.5
}

fn metric_value(map: &std::collections::BTreeMap<String, MetricValue>, key: &str) -> Option<f64> {
    map.get(key).and_then(|metric| metric.value)
}

fn git_commit() -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn apply_document_metrics(
    metrics: &mut crate::model::DocumentMetrics,
    artifacts: Option<&crate::model::CanonicalDocumentArtifacts>,
    deterministic: bool,
) {
    let Some(artifacts) = artifacts else {
        insert_bool(&mut metrics.technical, "document_compile_success", false);
        insert_bool(&mut metrics.technical, "document_best_effort_success", false);
        insert_bool(&mut metrics.technical, "document_deterministic", deterministic);
        return;
    };

    insert_bool(&mut metrics.technical, "document_compile_success", true);
    insert_bool(
        &mut metrics.technical,
        "document_best_effort_success",
        artifacts.best_effort.as_ref().map(|best| best.valid).unwrap_or(false),
    );
    insert_bool(&mut metrics.technical, "document_deterministic", deterministic);
    insert_bool(&mut metrics.technical, "document_compilation_valid", artifacts.compilation.valid);
    insert_bool(
        &mut metrics.technical,
        "document_translation_valid",
        artifacts.best_effort.as_ref().map(|best| best.valid).unwrap_or(false),
    );
    metrics.technical.insert(
        "silent_drop_count".into(),
        scalar_metric(artifacts.compilation.silent_drop_count as f64),
    );
    metrics.technical.insert(
        "pending_result_count".into(),
        scalar_metric(artifacts.compilation.status_counts.pending as f64),
    );
    metrics.technical.insert(
        "source_fallback_count".into(),
        scalar_metric(
            artifacts
                .best_effort
                .as_ref()
                .map(|best| best.source_fallback as f64)
                .unwrap_or(0.0),
        ),
    );
    metrics.technical.insert(
        "placeholder_count".into(),
        scalar_metric(
            artifacts
                .best_effort
                .as_ref()
                .map(|best| best.placeholder as f64)
                .unwrap_or(0.0),
        ),
    );
    metrics.structure.insert(
        "document_reconstruction_exact".into(),
        scalar_metric(artifacts.structure.reconstruction_exact as u8 as f64),
    );
    metrics.structure.insert(
        "document_sentence_coverage".into(),
        ratio(artifacts.compilation.result_count, artifacts.structure.sentence_count),
    );
    metrics.structure.insert(
        "document_result_coverage".into(),
        ratio(artifacts.compilation.result_count, artifacts.structure.sentence_count),
    );
    metrics.structure.insert(
        "document_paragraph_preservation".into(),
        ratio(
            artifacts
                .best_effort
                .as_ref()
                .map(|best| best.paragraph_count)
                .unwrap_or(artifacts.structure.paragraph_count),
            artifacts.structure.paragraph_count,
        ),
    );
    metrics.structure.insert(
        "document_output_sentence_coverage".into(),
        ratio(
            artifacts
                .best_effort
                .as_ref()
                .map(|best| best.sentence_result_count)
                .unwrap_or(0),
            artifacts.structure.sentence_count,
        ),
    );
    metrics.semantics.insert(
        "document_resolved_ratio".into(),
        ratio(
            artifacts.compilation.status_counts.resolved,
            artifacts.structure.sentence_count,
        ),
    );
    metrics.semantics.insert(
        "document_partial_ratio".into(),
        ratio(
            artifacts.compilation.status_counts.partial,
            artifacts.structure.sentence_count,
        ),
    );
    metrics.semantics.insert(
        "document_unresolved_ratio".into(),
        ratio(
            artifacts.compilation.status_counts.unresolved,
            artifacts.structure.sentence_count,
        ),
    );
    metrics.semantics.insert(
        "document_failed_ratio".into(),
        ratio(
            artifacts.compilation.status_counts.failed,
            artifacts.structure.sentence_count,
        ),
    );
    metrics.semantics.insert(
        "document_usable_ratio".into(),
        ratio(
            artifacts.compilation.status_counts.resolved + artifacts.compilation.status_counts.partial,
            artifacts.structure.sentence_count,
        ),
    );
}

fn insert_bool(
    map: &mut std::collections::BTreeMap<String, MetricValue>,
    key: &str,
    value: bool,
) {
    map.insert(key.to_string(), scalar_metric(value as u8 as f64));
}

fn scalar_metric(value: f64) -> MetricValue {
    MetricValue {
        value: Some(value),
        numerator: Some(value),
        denominator: Some(1.0),
        status: MetricStatus::Exact,
    }
}

fn ratio(numerator: usize, denominator: usize) -> MetricValue {
    if denominator == 0 {
        MetricValue {
            value: Some(1.0),
            numerator: Some(numerator as f64),
            denominator: Some(denominator as f64),
            status: MetricStatus::Exact,
        }
    } else {
        MetricValue {
            value: Some(numerator as f64 / denominator as f64),
            numerator: Some(numerator as f64),
            denominator: Some(denominator as f64),
            status: MetricStatus::Exact,
        }
    }
}
