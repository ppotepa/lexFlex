//! Evidence export for goal verification per strategist recommendation.
//! Exports durable artifacts from bulk test runs to GROK_GOAL_SCRATCH when set.

use std::path::Path;
use std::fs;

pub fn export_bulk_evidence(out_dir: &Path, scratch: &Path) {
    let _ = fs::create_dir_all(scratch);
    // copy summary
    if let Ok(s) = fs::read_to_string(out_dir.join("bulk_summary.json")) {
        let _ = fs::write(scratch.join("bulk_summary.json"), s);
    }
    // copy first razem *result* json (with semantics) if present; skip graph_hint
    if let Ok(entries) = fs::read_dir(out_dir) {
        for e in entries.flatten() {
            let name = e.file_name();
            let n = name.to_string_lossy();
            if n.contains("razem") && n.ends_with(".json") && !n.contains("graph_hint") {
                if let Ok(d) = fs::read(e.path()) {
                    let _ = fs::write(scratch.join("razem_case.json"), d);
                }
                break;
            }
        }
    }
    // copy one .concept.ron if present
    if let Ok(entries) = fs::read_dir(out_dir) {
        for e in entries.flatten() {
            if e.file_name().to_string_lossy().ends_with(".concept.ron") {
                if let Ok(d) = fs::read(e.path()) {
                    let _ = fs::write(scratch.join("example_concept.ron"), d);
                }
                break;
            }
        }
    }
    // print AC4 counters for capture
    if let Ok(s) = fs::read_to_string(out_dir.join("bulk_summary.json")) {
        if let Ok(j) = serde_json::from_str::<serde_json::Value>(&s) {
            println!("AC4 unique_concepts_inferred: {}", j["unique_concepts_inferred"]);
            println!("AC4 results_with_real_evidence: {}", j["results_with_real_evidence"]);
            if let Some(p) = j.get("results_with_new_concept_proposal") {
                println!("AC4 results_with_new_concept_proposal: {}", p);
            }
        }
    }
}
