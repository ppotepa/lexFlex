use crate::error::BenchmarkError;
use crate::model::DocumentCaseRun;
use std::fs;
use std::path::Path;

pub fn write_resolution_reports(root: &Path, case: &DocumentCaseRun) -> Result<(), BenchmarkError> {
    let resolution_dir = root.join("document/resolution");
    let summary_dir = root.join("document/resolution-summary");
    let decisions_dir = root.join("document/resolution-decisions");
    let clusters_dir = root.join("document/resolution-clusters");
    let alternatives_dir = root.join("document/resolution-alternatives");
    let gold_dir = root.join("document/resolution-gold");
    let errors_dir = root.join("document/resolution-errors");
    fs::create_dir_all(&resolution_dir).map_err(|source| BenchmarkError::Io {
        path: resolution_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&summary_dir).map_err(|source| BenchmarkError::Io {
        path: summary_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&decisions_dir).map_err(|source| BenchmarkError::Io {
        path: decisions_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&clusters_dir).map_err(|source| BenchmarkError::Io {
        path: clusters_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&alternatives_dir).map_err(|source| BenchmarkError::Io {
        path: alternatives_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&gold_dir).map_err(|source| BenchmarkError::Io {
        path: gold_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&errors_dir).map_err(|source| BenchmarkError::Io {
        path: errors_dir.display().to_string(),
        source,
    })?;
    if let Some(resolution) = &case.document_resolution {
        write_text(
            resolution_dir.join(format!("{}.json", case.case_id)),
            serde_json::to_string_pretty(resolution)?,
        )?;
    }
    write_text(
        summary_dir.join(format!("{}.txt", case.case_id)),
        resolution_summary_text(case),
    )?;
    write_text(
        decisions_dir.join(format!("{}.txt", case.case_id)),
        resolution_decisions_text(case),
    )?;
    write_text(
        clusters_dir.join(format!("{}.txt", case.case_id)),
        resolution_clusters_text(case),
    )?;
    write_text(
        alternatives_dir.join(format!("{}.tsv", case.case_id)),
        resolution_alternatives_tsv(case),
    )?;
    write_text(
        gold_dir.join(format!("{}.txt", case.case_id)),
        "gold corpus not attached\n".to_string(),
    )?;
    write_text(
        errors_dir.join(format!("{}.txt", case.case_id)),
        resolution_errors_text(case),
    )?;
    Ok(())
}

fn write_text(path: impl AsRef<Path>, text: String) -> Result<(), BenchmarkError> {
    let path = path.as_ref();
    fs::write(path, text).map_err(|source| BenchmarkError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn resolution_summary_text(case: &DocumentCaseRun) -> String {
    let mut lines = vec![format!("CASE: {}", case.case_id)];
    if let Some(resolution) = &case.document_resolution_artifacts {
        lines.push(format!("SCHEMA_VERSION: {}", resolution.schema_version));
        lines.push(format!("ALGORITHM_VERSION: {}", resolution.algorithm_version));
        lines.push(format!("RESOLUTION_SHA256: {}", resolution.resolution_sha256));
        lines.push(format!("VALID: {}", resolution.valid));
        lines.push(format!("DETERMINISTIC: {}", resolution.deterministic));
        lines.push(format!("MENTIONS_TOTAL: {}", resolution.mentions_total));
        lines.push(format!("SYNTHETIC_MENTIONS_TOTAL: {}", resolution.synthetic_mentions_total));
        lines.push(format!("DECISIONS_TOTAL: {}", resolution.decisions_total));
        lines.push(format!("CLUSTERS_TOTAL: {}", resolution.clusters_total));
    }
    lines.join("\n") + "\n"
}

fn resolution_decisions_text(case: &DocumentCaseRun) -> String {
    let mut lines = vec![format!("CASE: {}", case.case_id)];
    if let Some(resolution) = &case.document_resolution {
        for decision in resolution.decision_order.iter().filter_map(|id| resolution.decisions.get(id)) {
            lines.push(format!(
                "{} | {:?} | {:?} | score={} | target={:?}",
                decision.id, decision.stage, decision.kind, decision.score, decision.selected_target
            ));
        }
    }
    lines.join("\n") + "\n"
}

fn resolution_clusters_text(case: &DocumentCaseRun) -> String {
    let mut lines = vec![format!("CASE: {}", case.case_id)];
    if let Some(resolution) = &case.document_resolution {
        for cluster in resolution.cluster_order.iter().filter_map(|id| resolution.clusters.get(id)) {
            lines.push(format!(
                "{} | mentions={} | canonical={:?} | concept={}",
                cluster.id,
                cluster.mention_refs.len(),
                cluster.canonical_name,
                cluster.canonical_concept
            ));
        }
    }
    lines.join("\n") + "\n"
}

fn resolution_alternatives_tsv(case: &DocumentCaseRun) -> String {
    let mut lines = vec!["decision_id\tmention\ttarget\tscore\tconfidence_milli\tkind\treason".to_string()];
    if let Some(resolution) = &case.document_resolution {
        for decision in resolution.decision_order.iter().filter_map(|id| resolution.decisions.get(id)) {
            for alt in &decision.alternatives {
                lines.push(format!(
                    "{}\t{}\t{}\t{}\t{}\t{:?}\t{}",
                    decision.id,
                    decision.mention,
                    alt.target,
                    alt.score,
                    alt.confidence_milli,
                    alt.kind,
                    alt.reason
                ));
            }
        }
    }
    lines.join("\n") + "\n"
}

fn resolution_errors_text(case: &DocumentCaseRun) -> String {
    let mut lines = vec![format!("CASE: {}", case.case_id)];
    if !case.document_error_categories.is_empty() {
        lines.push("ERROR_CATEGORIES:".into());
        for error in &case.document_error_categories {
            lines.push(format!("  {error}"));
        }
    }
    if let Some(resolution) = &case.document_resolution {
        if !resolution.diagnostics.is_empty() {
            lines.push("RESOLUTION_DIAGNOSTICS:".into());
            for diagnostic in &resolution.diagnostics {
                lines.push(format!(
                    "  {} | {:?} | {} | {}",
                    diagnostic.id, diagnostic.severity, diagnostic.code, diagnostic.message
                ));
            }
        }
    }
    lines.join("\n") + "\n"
}
