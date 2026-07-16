#![forbid(unsafe_code)]

mod bench;
mod fixtures;

use bench::{BenchmarkCase, BenchmarkSuite};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "lexflex-benchmark")]
#[command(about = "Benchmark the lexFlex semantic kernel")]
struct Cli {
    #[arg(long, default_value_t = 100)]
    iterations: usize,

    #[arg(long, value_enum)]
    case: Vec<BenchmarkCase>,

    #[arg(long)]
    output: Option<PathBuf>,
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

    let json = serde_json::to_string_pretty(&report).expect("benchmark report serializes");

    if let Some(path) = cli.output {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).expect("benchmark output directory");
            }
        }
        std::fs::write(&path, &json).expect("write benchmark report");
    } else {
        println!("{json}");
    }
}
