#![forbid(unsafe_code)]

mod bench;
mod fixtures;

use bench::{BenchmarkCase, BenchmarkSuite};
use clap::Parser;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "lexflex-benchmark")]
#[command(about = "Benchmark the lexFlex semantic kernel")]
struct Cli {
    #[arg(long, default_value_t = 100)]
    iterations: usize,

    #[arg(long, value_enum)]
    case: Vec<BenchmarkCase>,
}

fn main() {
    let cli = Cli::parse();
    if cli.iterations == 0 {
        eprintln!("iterations must be greater than zero");
        std::process::exit(2);
    }

    let cases = if cli.case.is_empty() {
        BenchmarkCase::all()
    } else {
        cli.case
    };

    let suite = BenchmarkSuite::new(Arc::new(fixtures::kernel_catalog()));
    let report = suite.run_all(&cases, cli.iterations);

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("benchmark report serializes")
    );
}
