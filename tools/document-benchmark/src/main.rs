use clap::Parser;
use lexflex_document_benchmark::{
    cli::{Cli, Commands, SplitArg},
    compare,
    corpus,
    error::BenchmarkError,
    freeze,
    provenance,
    report,
    runner::{BenchmarkRunner, RunOptions},
    validate,
    model::{CorpusSplit, DocumentRun, RunSelectionProvenance},
};
use std::fs;
use std::path::Path;

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();
    let cli = Cli::parse();
    let code = match run(cli, raw_args) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            match err {
                BenchmarkError::Message(_) => 2,
                BenchmarkError::CorpusInvalid(_) | BenchmarkError::ProfileValidation(_) => 3,
                BenchmarkError::CriticalRegressions(_) => 5,
                BenchmarkError::FreezeRefused => 6,
                _ => 1,
            }
        }
    };
    std::process::exit(code);
}

fn run(cli: Cli, raw_args: Vec<String>) -> Result<(), BenchmarkError> {
    match cli.command {
        Commands::Validate {
            corpus,
            strict_release,
        } => {
            let corpus = corpus::load_corpus(Path::new(&corpus))?;
            let report = validate::validate_corpus(&corpus);
            if strict_release && corpus.cases.len() < 30 {
                return Err(BenchmarkError::CorpusInvalid(
                    "strict release requires 30 cases".into(),
                ));
            }
            if !report.errors.is_empty() {
                for issue in &report.errors {
                    eprintln!(
                        "error [{}] case={:?} path={:?}: {}",
                        issue.code, issue.case_id, issue.path, issue.message
                    );
                }
                return Err(BenchmarkError::CorpusInvalid("corpus invalid".into()));
            }
            Ok(())
        }
        Commands::Run {
            corpus,
            data,
            split,
            case,
            repeat,
            output,
            fail_fast,
            show_reference,
        } => {
            let corpus = corpus::load_corpus(Path::new(&corpus))?;
            let runner = BenchmarkRunner::new(Path::new(&data))?;
            let options = RunOptions {
                split: match split {
                    SplitArg::All => None,
                    SplitArg::Dev => Some(CorpusSplit::Dev),
                    SplitArg::Regression => Some(CorpusSplit::Regression),
                    SplitArg::Holdout => Some(CorpusSplit::Holdout),
                },
                case_ids: case,
                repeat_count: repeat,
                fail_fast,
                include_holdout_reference: show_reference,
            };
            let mut run = runner.run(&corpus, &options);
            provenance::attach_run_provenance(
                &mut run,
                &corpus,
                RunSelectionProvenance {
                    repeat_count: options.repeat_count,
                    split: options.split,
                    case_filter: options.case_ids.clone(),
                    all_enabled_cases: options.split.is_none() && options.case_ids.is_empty(),
                },
                raw_args,
            )?;
            report::write_run_report(Path::new(&output), &run)?;
            Ok(())
        }
        Commands::Compare {
            baseline,
            candidate,
            output,
        } => {
            let baseline: DocumentRun = serde_json::from_str(&read_file(&baseline)?)?;
            let candidate: DocumentRun = serde_json::from_str(&read_file(&candidate)?)?;
            let comparison = compare::compare_runs(&baseline, &candidate);
            fs::write(&output, serde_json::to_string_pretty(&comparison)?).map_err(|source| {
                BenchmarkError::Io {
                    path: output.clone(),
                    source,
                }
            })?;
            if comparison.critical_regressions > 0 {
                return Err(BenchmarkError::CriticalRegressions(
                    comparison.critical_regressions,
                ));
            }
            Ok(())
        }
        Commands::Freeze { run, corpus, output, confirm, force } => {
            let _ = freeze::freeze(freeze::FreezeOptions {
                run: Path::new(&run),
                corpus: Path::new(&corpus),
                output: Path::new(&output),
                confirm: &confirm,
                force,
            })?;
            Ok(())
        }
    }
}

fn read_file(path: &str) -> Result<String, BenchmarkError> {
    fs::read_to_string(path).map_err(|source| BenchmarkError::Io {
        path: path.to_string(),
        source,
    })
}
