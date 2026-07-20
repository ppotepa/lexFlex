use super::*;

#[test]
fn benchmark_report_serializes() {
    let report = BenchmarkReport {
        cases: vec![BenchmarkResult {
            case: "entity".into(),
            iterations: 1,
            total_ns: 1,
            avg_ns: 1,
            min_ns: 1,
            median_ns: 1,
            p95_ns: 1,
            max_ns: 1,
            output_hash: "hash".into(),
            parser_metrics: None,
            steps: vec!["compile".into(), "execute".into()],
        }],
    };
    let json = serde_json::to_string(&report);
    assert!(json.as_ref().is_ok_and(|value| value.contains("entity")));
}
