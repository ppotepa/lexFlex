use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "lexflex-document-benchmark")]
#[command(about = "Document benchmark and baseline tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Validate {
        #[arg(long, default_value = "benchmarks/document_v1")]
        corpus: String,
        #[arg(long)]
        strict_release: bool,
    },
    Run {
        #[arg(long, default_value = "benchmarks/document_v1")]
        corpus: String,
        #[arg(long, default_value = "data")]
        data: String,
        #[arg(long, value_enum, default_value_t = SplitArg::All)]
        split: SplitArg,
        #[arg(long, value_delimiter = ',')]
        case: Vec<String>,
        #[arg(long, default_value_t = 1)]
        repeat: usize,
        #[arg(long, default_value = "results/document-v1/run")]
        output: String,
        #[arg(long)]
        fail_fast: bool,
        #[arg(long)]
        show_reference: bool,
    },
    Compare {
        #[arg(long)]
        baseline: String,
        #[arg(long)]
        candidate: String,
        #[arg(long)]
        output: String,
    },
    Freeze {
        #[arg(long)]
        run: String,
        #[arg(long, default_value = "benchmarks/document_v1")]
        corpus: String,
        #[arg(long)]
        output: String,
        #[arg(long)]
        confirm: String,
        #[arg(long)]
        force: bool,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitArg {
    Dev,
    Regression,
    Holdout,
    All,
}
