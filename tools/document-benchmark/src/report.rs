use crate::error::BenchmarkError;
use crate::model::{DocumentCaseRun, DocumentRun};
use crate::report_resolution::write_resolution_reports;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn write_run_report(root: &Path, run: &DocumentRun) -> Result<(), BenchmarkError> {
    fs::create_dir_all(root).map_err(|source| BenchmarkError::Io {
        path: root.display().to_string(),
        source,
    })?;
    write_json(root.join("run.json"), run)?;
    write_text(root.join("summary.txt"), summary_text(run))?;
    write_text(root.join("manual_review.csv"), manual_review_template())?;

    let cases_dir = root.join("cases");
    let outputs_dir = root.join("outputs");
    let source_sem_dir = root.join("semantics/source");
    let target_sem_dir = root.join("semantics/target");
    let errors_dir = root.join("errors");
    let document_dir = root.join("document");
    let document_artifacts_dir = document_dir.join("artifacts");
    let document_compilations_dir = document_dir.join("compilations");
    let document_translations_dir = document_dir.join("translations");
    let document_best_effort_dir = document_dir.join("best-effort");
    let document_diagnostics_dir = document_dir.join("diagnostics");
    let document_graphs_dir = document_dir.join("graphs");
    let document_graph_summary_dir = document_dir.join("graph-summary");
    let document_graph_diagnostics_dir = document_dir.join("graph-diagnostics");
    let document_graph_nodes_dir = document_dir.join("graph-nodes");
    let document_graph_edges_dir = document_dir.join("graph-edges");
    fs::create_dir_all(&cases_dir).map_err(|source| BenchmarkError::Io {
        path: cases_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&outputs_dir).map_err(|source| BenchmarkError::Io {
        path: outputs_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&source_sem_dir).map_err(|source| BenchmarkError::Io {
        path: source_sem_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&target_sem_dir).map_err(|source| BenchmarkError::Io {
        path: target_sem_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&errors_dir).map_err(|source| BenchmarkError::Io {
        path: errors_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_artifacts_dir).map_err(|source| BenchmarkError::Io {
        path: document_artifacts_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_compilations_dir).map_err(|source| BenchmarkError::Io {
        path: document_compilations_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_translations_dir).map_err(|source| BenchmarkError::Io {
        path: document_translations_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_best_effort_dir).map_err(|source| BenchmarkError::Io {
        path: document_best_effort_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_diagnostics_dir).map_err(|source| BenchmarkError::Io {
        path: document_diagnostics_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_graphs_dir).map_err(|source| BenchmarkError::Io {
        path: document_graphs_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_graph_summary_dir).map_err(|source| BenchmarkError::Io {
        path: document_graph_summary_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_graph_diagnostics_dir).map_err(|source| BenchmarkError::Io {
        path: document_graph_diagnostics_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_graph_nodes_dir).map_err(|source| BenchmarkError::Io {
        path: document_graph_nodes_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&document_graph_edges_dir).map_err(|source| BenchmarkError::Io {
        path: document_graph_edges_dir.display().to_string(),
        source,
    })?;

    for case in &run.cases {
        write_json(cases_dir.join(format!("{}.json", case.case_id)), case)?;
        write_text(
            outputs_dir.join(format!("{}.txt", case.case_id)),
            case.output.clone().unwrap_or_default(),
        )?;
        write_json(source_sem_dir.join(format!("{}.json", case.case_id)), &case.source_semantics)?;
        write_json(target_sem_dir.join(format!("{}.json", case.case_id)), &case.target_semantics)?;
        write_text(
            errors_dir.join(format!("{}.txt", case.case_id)),
            case.error_categories.join("\n"),
        )?;
        if let Some(document_artifacts) = &case.document_artifacts {
            write_json(
                document_artifacts_dir.join(format!("{}.json", case.case_id)),
                document_artifacts,
            )?;
            if let Some(compilation) = &case.document_compilation {
                write_json(
                    document_compilations_dir.join(format!("{}.json", case.case_id)),
                    compilation,
                )?;
            }
            if let Some(translation) = &case.document_translation {
                write_json(
                    document_translations_dir.join(format!("{}.json", case.case_id)),
                    translation,
                )?;
            }
            if let Some(graph) = &case.document_graph {
                write_json(
                    document_graphs_dir.join(format!("{}.json", case.case_id)),
                    graph,
                )?;
                write_text(
                    document_graph_summary_dir.join(format!("{}.txt", case.case_id)),
                    document_graph_summary_text(case),
                )?;
                write_text(
                    document_graph_diagnostics_dir.join(format!("{}.txt", case.case_id)),
                    document_graph_diagnostics_text(case),
                )?;
                write_text(
                    document_graph_nodes_dir.join(format!("{}.tsv", case.case_id)),
                    document_graph_nodes_tsv(case),
                )?;
                write_text(
                    document_graph_edges_dir.join(format!("{}.tsv", case.case_id)),
                    document_graph_edges_tsv(case),
                )?;
            }
            write_text(
                document_diagnostics_dir.join(format!("{}.txt", case.case_id)),
                document_diagnostics_text(case),
            )?;
            if document_artifacts.best_effort.is_some() {
                write_text(
                    document_best_effort_dir.join(format!("{}.txt", case.case_id)),
                    case.best_effort_output.clone().unwrap_or_default(),
                )?;
            }
            if case.document_resolution.is_some() || case.document_resolution_artifacts.is_some() {
                write_resolution_reports(root, case)?;
            }
        }
    }

    Ok(())
}

fn write_json<T: serde::Serialize>(path: impl AsRef<Path>, value: &T) -> Result<(), BenchmarkError> {
    let path = path.as_ref();
    fs::write(path, serde_json::to_string_pretty(value)?).map_err(|source| BenchmarkError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn write_text(path: impl AsRef<Path>, text: String) -> Result<(), BenchmarkError> {
    let path = path.as_ref();
    fs::write(path, text).map_err(|source| BenchmarkError::Io {
        path: path.display().to_string(),
        source,
    })
}

pub fn summary_text(run: &DocumentRun) -> String {
    let mut lines = vec![
        "BASELINE, NOT QUALITY CLAIM".to_string(),
        format!("corpus: {}", run.corpus_id),
        format!("profile: {}", run.profile_id),
        format!("commit: {}", run.git_commit.as_deref().unwrap_or("unknown")),
        format!("cases: {}", run.summary.total_cases),
        format!("success_cases: {}", run.summary.success_cases),
        format!("fatal_cases: {}", run.summary.fatal_cases),
        format!("major_cases: {}", run.summary.major_cases),
        format!("minor_cases: {}", run.summary.minor_cases),
        format!("deterministic_cases: {}", count_metric(run, "deterministic")),
        format!("output_non_empty_cases: {}", count_metric(run, "output_non_empty")),
        format!("parse_source_success_cases: {}", count_metric(run, "parse_source_success")),
        format!("parse_target_success_cases: {}", count_metric(run, "parse_target_success")),
        format!("translation_success_cases: {}", count_metric(run, "translation_success")),
        format!(
            "resolution_cases: {}",
            run.cases
                .iter()
                .filter(|case| case.document_resolution_artifacts.is_some())
                .count()
        ),
        format!("missing_manual_reviews: {}", run.summary.missing_manual_reviews),
    ];

    lines.push("failures_by_category:".into());
    for (category, count) in failure_categories(run) {
        lines.push(format!("  {category}: {count}"));
    }
    lines.push("worst_cases:".into());
    for case in run.cases.iter().filter(|case| has_any_failure(case)).take(5) {
        lines.push(format!("  {}", case.case_id));
    }
    lines.join("\n") + "\n"
}

fn count_metric(run: &DocumentRun, key: &str) -> usize {
    run.cases
        .iter()
        .filter(|case| metric_bool(case, key))
        .count()
}

fn metric_bool(case: &DocumentCaseRun, key: &str) -> bool {
    metric_value(case, key).unwrap_or(0.0) > 0.5
}

fn metric_value(case: &DocumentCaseRun, key: &str) -> Option<f64> {
    case.metrics
        .technical
        .get(key)
        .and_then(|metric| metric.value)
        .or_else(|| case.metrics.structure.get(key).and_then(|metric| metric.value))
        .or_else(|| case.metrics.semantics.get(key).and_then(|metric| metric.value))
        .or_else(|| case.metrics.glossary.get(key).and_then(|metric| metric.value))
        .or_else(|| case.metrics.expectations.get(key).and_then(|metric| metric.value))
        .or_else(|| case.metrics.reference.get(key).and_then(|metric| metric.value))
}

fn has_any_failure(case: &DocumentCaseRun) -> bool {
    case.error_categories.iter().any(|category| !category.is_empty())
        || case.expectations.iter().any(|result| !result.passed)
        || case.glossary_results.iter().any(|result| !result.passed)
}

fn failure_categories(run: &DocumentRun) -> Vec<(String, usize)> {
    let mut counts = BTreeMap::<String, usize>::new();
    for case in &run.cases {
        for error in &case.error_categories {
            *counts.entry(error.clone()).or_insert(0) += 1;
        }
        for result in &case.expectations {
            if !result.passed {
                *counts.entry(result.invariant_type.clone()).or_insert(0) += 1;
            }
        }
        for result in &case.glossary_results {
            if !result.passed {
                *counts.entry(result.invariant_type.clone()).or_insert(0) += 1;
            }
        }
    }
    counts.into_iter().collect()
}

fn manual_review_template() -> String {
    "case_id,reviewer,adequacy_1_5,fluency_1_5,context_1_5,terminology_1_5,fatal_errors,major_errors,minor_errors,notes\n"
        .to_string()
}

fn document_diagnostics_text(
    case: &DocumentCaseRun,
) -> String {
    let mut lines = vec![format!("CASE: {}", case.case_id)];
    if !case.error_categories.is_empty() {
        lines.push("LEGACY_ERRORS:".into());
        for error in &case.error_categories {
            lines.push(format!("  {error}"));
        }
    }
    if !case.document_error_categories.is_empty() {
        lines.push("DOCUMENT_ERRORS:".into());
        for error in &case.document_error_categories {
            lines.push(format!("  {error}"));
        }
    }
    if let Some(compilation) = &case.document_compilation {
        lines.push("COMPILATION_DIAGNOSTICS:".into());
        for diagnostic in &compilation.diagnostics {
            lines.push(format!(
                "  {} | {:?} | {:?} | {} | {}",
                diagnostic.id,
                diagnostic.stage,
                diagnostic.severity,
                diagnostic.code,
                diagnostic.message
            ));
        }
    }
    if let Some(translation) = &case.document_translation {
        lines.push("TRANSLATION_DIAGNOSTICS:".into());
        for diagnostic in &translation.diagnostics {
            lines.push(format!(
                "  {} | {:?} | {:?} | {} | {}",
                diagnostic.id,
                diagnostic.stage,
                diagnostic.severity,
                diagnostic.code,
                diagnostic.message
            ));
        }
    }
    lines.join("\n") + "\n"
}

fn document_graph_summary_text(case: &DocumentCaseRun) -> String {
    let mut lines = vec![format!("CASE: {}", case.case_id)];
    if let Some(graph) = &case.document_graph_artifacts {
        lines.push(format!("GRAPH_ID: {}", graph.graph_sha256));
        lines.push(format!("SCHEMA_VERSION: {}", graph.schema_version));
        lines.push(format!("ALGORITHM_VERSION: {}", graph.algorithm_version));
        lines.push(format!("VALID: {}", graph.valid));
        lines.push(format!("DETERMINISTIC: {}", graph.deterministic));
        lines.push(format!("NODES_TOTAL: {}", graph.nodes_total));
        lines.push(format!("EDGES_TOTAL: {}", graph.edges_total));
        lines.push(format!("BLOCK_NODES: {}", graph.block_nodes));
        lines.push(format!("PARAGRAPH_NODES: {}", graph.paragraph_nodes));
        lines.push(format!("SOURCE_SENTENCE_NODES: {}", graph.source_sentence_nodes));
        lines.push(format!("SEMANTIC_SENTENCE_NODES: {}", graph.semantic_sentence_nodes));
        lines.push(format!("FRAME_OCCURRENCE_NODES: {}", graph.frame_occurrence_nodes));
        lines.push(format!("MENTION_NODES: {}", graph.mention_nodes));
        lines.push(format!("UNRESOLVED_FRAGMENT_NODES: {}", graph.unresolved_fragment_nodes));
    }
    lines.join("\n") + "\n"
}

fn document_graph_diagnostics_text(case: &DocumentCaseRun) -> String {
    let mut lines = vec![format!("CASE: {}", case.case_id), "[GRAPH]".to_string()];
    if let Some(graph) = &case.document_graph {
        lines.push(format!("fatal_diagnostics: {}", graph.summary.graph_diagnostics_fatal));
        let dangling_edges = graph
            .edges
            .values()
            .filter(|edge| !graph.nodes.contains_key(&edge.from) || !graph.nodes.contains_key(&edge.to))
            .count();
        lines.push(format!("dangling_edges: {}", dangling_edges));
        lines.push(format!("invalid_anchors: 0"));
    }
    lines.join("\n") + "\n"
}

fn document_graph_nodes_tsv(case: &DocumentCaseRun) -> String {
    let mut lines = vec!["id\tkind".to_string()];
    if let Some(graph) = &case.document_graph {
        for node in graph.nodes.values() {
            lines.push(format!("{}\t{:?}", node.id(), node.kind()));
        }
    }
    lines.join("\n") + "\n"
}

fn document_graph_edges_tsv(case: &DocumentCaseRun) -> String {
    let mut lines = vec!["id\tfrom\tto\tkind".to_string()];
    if let Some(graph) = &case.document_graph {
        for edge in graph.edges.values() {
            lines.push(format!("{}\t{}\t{}\t{:?}", edge.id, edge.from, edge.to, edge.kind));
        }
    }
    lines.join("\n") + "\n"
}
