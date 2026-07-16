#![forbid(unsafe_code)]

use clap::Parser;
use lexflex_model::SemanticExpression;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "lexflex-model-inspect")]
#[command(about = "Inspect canonical semantic expressions")]
struct Cli {
    #[arg(long, value_name = "PATH")]
    expression: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let source = match std::fs::read_to_string(&cli.expression) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{}: {error}", cli.expression.display());
            std::process::exit(1);
        }
    };
    let expression: SemanticExpression = match serde_json::from_str(&source) {
        Ok(expression) => expression,
        Err(error) => {
            eprintln!("expression json error: {error}");
            std::process::exit(1);
        }
    };

    println!(
        "canonical_hash: {}",
        lexflex_model::canonical_hash(&expression)
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&expression).expect("serialize")
    );
}
