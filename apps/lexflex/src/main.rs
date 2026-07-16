#![forbid(unsafe_code)]

use crate::cli::error::CliError;
use crate::cli::evidence::EvidencePolicyArg;
use crate::cli::runtime::RuntimeArgs;
use clap::{Parser, Subcommand};
use lexflex_lingua::ExpansionMode;
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

        #[arg(long, value_enum, default_value_t = ExpansionPolicyArg::Preserve)]
        expansion: ExpansionPolicyArg,
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

        #[arg(long, value_enum, default_value_t = EvidencePolicyArg::Required)]
        evidence: EvidencePolicyArg,
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

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum ExpansionPolicyArg {
    Preserve,
    Transparent,
    AllDefined,
}

impl From<ExpansionPolicyArg> for ExpansionMode {
    fn from(value: ExpansionPolicyArg) -> Self {
        match value {
            ExpansionPolicyArg::Preserve => ExpansionMode::PreserveApplications,
            ExpansionPolicyArg::Transparent => ExpansionMode::ExpandTransparent,
            ExpansionPolicyArg::AllDefined => ExpansionMode::ExpandAllDefined,
        }
    }
}

fn main() {
    if let Err(error) = run_from_env() {
        eprintln!("{error}");
        process::exit(error.exit_code());
    }
}

pub fn run_from_env() -> Result<(), CliError> {
    let Cli {
        session,
        state_dir,
        model_root: default_model_root,
        language_root: default_language_root,
        command,
    } = Cli::parse();

    let runtime_args = RuntimeArgs {
        session,
        state_dir,
        model_root: default_model_root.clone(),
        language_root: default_language_root.clone(),
    };

    match command {
        Command::LinguaEval {
            program,
            trace,
            expansion,
        } => {
            let mut runtime = runtime_args.build()?;
            cli::lingua_eval::run(&mut runtime, &program, expansion.into(), trace)
                .map_err(CliError::Command)
        }
        Command::LinguaIngest { program, evidence } => {
            let mut runtime = runtime_args.build()?;
            cli::lingua_ingest::run(&mut runtime, &program, &evidence).map_err(CliError::Command)
        }
        Command::LinguaQuery { goal } => {
            let mut runtime = runtime_args.build()?;
            cli::lingua_query::run(&mut runtime, &goal).map_err(CliError::Command)
        }
        Command::TextAnalyze { input, derivation } => {
            let mut runtime = runtime_args.build()?;
            cli::text_analyze::run(&mut runtime, input, derivation).map_err(CliError::Command)
        }
        Command::TextIngest { input } => {
            let mut runtime = runtime_args.build()?;
            cli::text_ingest::run(&mut runtime, input).map_err(CliError::Command)
        }
        Command::TextAsk {
            input,
            limit,
            evidence,
        } => {
            let mut runtime = runtime_args.build()?;
            cli::text_ask::run(&mut runtime, input, limit, evidence.into())
                .map_err(CliError::Command)
        }
        Command::SessionInspect => {
            let mut runtime = runtime_args.build()?;
            cli::session_inspect::run(&mut runtime).map_err(CliError::Command)
        }
        Command::SessionClear => {
            let mut runtime = runtime_args.build()?;
            cli::session_clear::run(&mut runtime).map_err(CliError::Command)
        }
        Command::ModelValidate { model_root } => cli::model_validate::run(
            model_root.unwrap_or(default_model_root),
            default_language_root,
        )
        .map_err(CliError::Command),
        Command::LanguageValidate { language_root } => cli::language_validate::run(
            default_model_root,
            language_root.unwrap_or(default_language_root),
        )
        .map_err(CliError::Command),
    }
}
