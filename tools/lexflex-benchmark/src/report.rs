use crate::bench::BenchmarkCase;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct BenchmarkReport {
    pub cases: Vec<BenchmarkResult>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BenchmarkThresholdError {
    #[error("benchmark case {case} p95 {observed_ns}ns exceeds limit {limit_ns}ns")]
    P95Exceeded {
        case: String,
        observed_ns: u128,
        limit_ns: u128,
    },
}

impl BenchmarkReport {
    pub fn enforce_p95(&self, limit_ns: u128) -> Result<(), BenchmarkThresholdError> {
        self.cases
            .iter()
            .find(|case| case.p95_ns > limit_ns)
            .map_or(Ok(()), |case| {
                Err(BenchmarkThresholdError::P95Exceeded {
                    case: case.case.clone(),
                    observed_ns: case.p95_ns,
                    limit_ns,
                })
            })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p95_threshold_rejects_regression() {
        let report = BenchmarkReport {
            cases: vec![BenchmarkResult {
                case: "entity".into(),
                iterations: 1,
                total_ns: 10,
                avg_ns: 10,
                min_ns: 10,
                median_ns: 10,
                p95_ns: 10,
                max_ns: 10,
                output_hash: "hash".into(),
                parser_metrics: None,
                steps: Vec::new(),
            }],
        };
        assert!(report.enforce_p95(10).is_ok());
        assert!(matches!(
            report.enforce_p95(9),
            Err(BenchmarkThresholdError::P95Exceeded { .. })
        ));
    }
}
