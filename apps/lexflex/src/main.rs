#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use lexflex_lingua::solve::EvidencePolicy;
use std::path::PathBuf;
use std::process;

mod cli;

#[derive(Parser, Debug)]
#[command(name = "lexflex-app")]
#[command(about = "Lingua runtime for lexFlex")]
struct Cli {
    #[arg(long, default_value = "demo")]
    session: String,

    #[arg(long, default_value = ".lexflex")]
    state_dir: PathBuf,

    #[arg(long, default_value = "data/model")]
    model_root: PathBuf,

    #[arg(long, default_value = "data/languages")]
    language_root: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    LinguaEval {
        #[arg(value_name = "PROGRAM")]
        program: PathBuf,

        #[arg(long)]
        trace: bool,
    },
    LinguaIngest {
        #[arg(value_name = "PROGRAM")]
        program: PathBuf,

        #[arg(value_name = "EVIDENCE")]
        evidence: PathBuf,
    },
    LinguaQuery {
        #[arg(value_name = "GOAL")]
        goal: PathBuf,
    },
    TextAnalyze {
        #[command(flatten)]
        input: cli::input::TextInputArgs,

        #[arg(long)]
        derivation: bool,
    },
    TextIngest {
        #[command(flatten)]
        input: cli::input::TextInputArgs,
    },
    TextAsk {
        #[command(flatten)]
        input: cli::input::TextInputArgs,

        #[arg(long, default_value_t = 10)]
        limit: usize,

        #[arg(long, default_value = "required")]
        evidence: String,
    },
    SessionInspect,
    SessionClear,
    ModelValidate {
        #[arg(value_name = "MODEL_ROOT")]
        model_root: Option<PathBuf>,
    },
    LanguageValidate {
        #[arg(value_name = "LANGUAGE_ROOT")]
        language_root: Option<PathBuf>,
    },
}

fn main() {
    if let Err(error) = run_from_env() {
        eprintln!("{error}");
        process::exit(1);
    }
}

pub fn run_from_env() -> Result<(), String> {
    let Cli {
        session,
        state_dir,
        model_root: default_model_root,
        language_root: default_language_root,
        command,
    } = Cli::parse();

    match command {
        Command::LinguaEval { program, trace } => {
            let mut runtime = lexflex_engine::runtime::LexFlexRuntime::with_session_and_roots(
                session,
                state_dir,
                default_model_root,
                default_language_root.clone(),
            )
            .map_err(|error| error.to_string())?;
            cli::lingua_eval::run(&mut runtime, &program, trace)
        }
        Command::LinguaIngest { program, evidence } => {
            let mut runtime = lexflex_engine::runtime::LexFlexRuntime::with_session_and_roots(
                session,
                state_dir,
                default_model_root,
                default_language_root.clone(),
            )
            .map_err(|error| error.to_string())?;
            cli::lingua_ingest::run(&mut runtime, &program, &evidence)
        }
        Command::LinguaQuery { goal } => {
            let mut runtime = lexflex_engine::runtime::LexFlexRuntime::with_session_and_roots(
                session,
                state_dir,
                default_model_root,
                default_language_root.clone(),
            )
            .map_err(|error| error.to_string())?;
            cli::lingua_query::run(&mut runtime, &goal)
        }
        Command::TextAnalyze { input, derivation } => {
            let mut runtime = lexflex_engine::runtime::LexFlexRuntime::with_session_and_roots(
                session,
                state_dir,
                default_model_root,
                default_language_root.clone(),
            )
            .map_err(|error| error.to_string())?;
            cli::text_analyze::run(&mut runtime, input, derivation)
        }
        Command::TextIngest { input } => {
            let mut runtime = lexflex_engine::runtime::LexFlexRuntime::with_session_and_roots(
                session,
                state_dir,
                default_model_root,
                default_language_root.clone(),
            )
            .map_err(|error| error.to_string())?;
            cli::text_ingest::run(&mut runtime, input)
        }
        Command::TextAsk {
            input,
            limit,
            evidence,
        } => {
            let mut runtime = lexflex_engine::runtime::LexFlexRuntime::with_session_and_roots(
                session,
                state_dir,
                default_model_root,
                default_language_root.clone(),
            )
            .map_err(|error| error.to_string())?;
            let evidence = match evidence.as_str() {
                "required" => EvidencePolicy::Required,
                "optional" => EvidencePolicy::Optional,
                "ignore" => EvidencePolicy::Ignore,
                other => return Err(format!("invalid evidence policy: {other}")),
            };
            cli::text_ask::run(&mut runtime, input, limit, evidence)
        }
        Command::SessionInspect => {
            let mut runtime = lexflex_engine::runtime::LexFlexRuntime::with_session_and_roots(
                session,
                state_dir,
                default_model_root,
                default_language_root.clone(),
            )
            .map_err(|error| error.to_string())?;
            cli::session_inspect::run(&mut runtime)
        }
        Command::SessionClear => {
            let mut runtime = lexflex_engine::runtime::LexFlexRuntime::with_session_and_roots(
                session,
                state_dir,
                default_model_root,
                default_language_root.clone(),
            )
            .map_err(|error| error.to_string())?;
            cli::session_clear::run(&mut runtime)
        }
        Command::ModelValidate { model_root } => cli::model_validate::run(
            model_root.unwrap_or(default_model_root),
            default_language_root,
        ),
        Command::LanguageValidate { language_root } => cli::language_validate::run(
            default_model_root,
            language_root.unwrap_or(default_language_root),
        ),
    }
}
