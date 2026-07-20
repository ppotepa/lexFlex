#![forbid(unsafe_code)]

use clap::Parser;
use lexflex_model::SemanticExpression;
use std::io::Read;
use std::path::PathBuf;

const MAX_EXPRESSION_BYTES: usize = 8 * 1024 * 1024;

#[derive(Parser, Debug)]
#[command(name = "lexflex-model-inspect")]
#[command(about = "Inspect canonical semantic expressions")]
struct Cli {
    #[arg(long, value_name = "PATH")]
    expression: PathBuf,
}

fn read_expression(
    path: &std::path::Path,
) -> Result<SemanticExpression, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take((MAX_EXPRESSION_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_EXPRESSION_BYTES {
        return Err("semantic expression exceeds 8 MiB limit".into());
    }
    let source = String::from_utf8(bytes)?;
    Ok(serde_json::from_str(&source)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let expression = read_expression(&cli.expression)?;

    println!(
        "canonical_hash: {}",
        lexflex_model::canonical_hash(&expression)?
    );
    println!("{}", serde_json::to_string_pretty(&expression)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_expression_is_rejected_before_json_parse() {
        let path = std::env::temp_dir().join(format!(
            "lexflex-model-inspect-oversized-{}",
            std::process::id()
        ));
        std::fs::write(&path, vec![b'{'; MAX_EXPRESSION_BYTES + 1]).expect("write");
        let error = read_expression(&path).expect_err("oversized input");
        assert!(error.to_string().contains("8 MiB"));
        let _ = std::fs::remove_file(path);
    }
}
