use crate::bench::BenchmarkCase;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BenchmarkReport {
    pub cases: Vec<BenchmarkResult>,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkResult {
    pub case: String,
    pub iterations: usize,
    pub total_ns: u128,
    pub avg_ns: u128,
    pub min_ns: u128,
    pub median_ns: u128,
    pub p95_ns: u128,
    pub max_ns: u128,
    pub output_hash: String,
    pub parser_metrics: Option<ParserMetricsSnapshot>,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParserMetricsSnapshot {
    pub token_count: usize,
    pub lexical_candidate_count: usize,
    pub chart_item_count: usize,
    pub chart_replacement_count: usize,
    pub semantic_duplicate_count: usize,
    pub rejected_application_count: usize,
    pub complete_semantic_count: usize,
    pub max_cell_size: usize,
    pub derivation_depth: usize,
    pub max_semantic_nodes: usize,
}

impl BenchmarkResult {
    pub fn from_samples(
        case: BenchmarkCase,
        iterations: usize,
        mut samples: Vec<u128>,
        steps: Vec<&'static str>,
        output_hash: String,
        parser_metrics: Option<ParserMetricsSnapshot>,
    ) -> Self {
        samples.sort_unstable();
        let total_ns = samples.iter().copied().sum();
        let min_ns = *samples.first().unwrap_or(&0);
        let max_ns = *samples.last().unwrap_or(&0);
        let median_ns = samples[samples.len() / 2];
        let p95_ns = samples[((samples.len().saturating_sub(1)) * 95) / 100];
        let avg_ns = total_ns / iterations as u128;
        Self {
            case: case.label().to_string(),
            iterations,
            total_ns,
            avg_ns,
            min_ns,
            median_ns,
            p95_ns,
            max_ns,
            output_hash,
            parser_metrics,
            steps: steps.into_iter().map(str::to_string).collect(),
        }
    }
}
