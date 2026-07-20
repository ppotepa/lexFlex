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
    TextGenerate {
        #[arg(value_name = "EXPRESSION")]
        expression: PathBuf,
        #[arg(long)]
        language: String,
        #[arg(long)]
        trace: bool,
    },
    SemanticRealize {
        #[arg(value_name = "EXPRESSION")]
        expression: PathBuf,
        #[arg(long)]
        language: String,
        #[arg(long)]
        trace: bool,
    },
    TextTranslateText {
        #[command(flatten)]
        input: cli::input::TextInputArgs,

        #[arg(long, value_name = "LANGUAGE")]
        target_language: String,

        #[arg(long)]
        trace: bool,
    },
    DocumentIngest {
        #[arg(value_name = "DOCUMENT_ID")]
        document_id: String,
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long)]
        store: Option<PathBuf>,
    },
    DocumentRemove {
        #[arg(value_name = "DOCUMENT_ID")]
        document_id: String,
        #[arg(long)]
        store: PathBuf,
    },
    DocumentReplace {
        #[arg(value_name = "DOCUMENT_ID")]
        document_id: String,
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long)]
        store: PathBuf,
    },
    DocumentInspect {
        #[arg(long)]
        store: PathBuf,
    },
    DocumentQuery {
        #[arg(value_name = "EXPRESSION")]
        expression: PathBuf,
        #[arg(long)]
        store: PathBuf,
    },
    DocumentAnalyze {
        #[arg(long)]
        store: PathBuf,
        #[arg(long, value_name = "LANGUAGE")]
        language: String,
    },
    ProviderIngest {
        #[arg(value_name = "DOCUMENT_ID")]
        document_id: String,
        #[arg(value_name = "ARTIFACT")]
        artifact: PathBuf,
        #[arg(long)]
        store: PathBuf,
    },
    ConversationTurn {
        #[arg(value_name = "TURN_ID")]
        turn_id: String,
        #[arg(value_name = "EXPRESSION")]
        expression: PathBuf,
        #[arg(long)]
        state: Option<PathBuf>,
    },
    ConversationTextTurn {
        #[arg(value_name = "TURN_ID")]
        turn_id: String,
        #[command(flatten)]
        input: cli::input::TextInputArgs,
        #[arg(long)]
        state: Option<PathBuf>,
    },
    ConversationInspect {
        #[arg(long)]
        state: PathBuf,
    },
    ConversationResolve {
        #[arg(value_name = "ENTITY_TYPE")]
        entity_type: String,
        #[arg(long)]
        state: PathBuf,
    },
    LearningPropose {
        observation: PathBuf,
        proposal: PathBuf,
        #[arg(long)]
        expected_behavior: String,
    },
    LearningObserve {
        observation: PathBuf,
    },
    LearningApprove {
        overlay: PathBuf,
        proposal_id: String,
    },
    LearningReject {
        overlay: PathBuf,
        proposal_id: String,
    },
    LearningPromote {
        overlay: PathBuf,
        proposal_id: String,
    },
    SessionInspect,
    SessionClear,
    SessionMigrate {
        #[arg(long)]
        from_schema: u32,
    },
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
                .map_err(Into::into)
        }
        Command::LinguaIngest { program, evidence } => {
            let mut runtime = runtime_args.build()?;
            cli::lingua_ingest::run(&mut runtime, &program, &evidence).map_err(Into::into)
        }
        Command::LinguaQuery { goal } => {
            let mut runtime = runtime_args.build()?;
            cli::lingua_query::run(&mut runtime, &goal).map_err(Into::into)
        }
        Command::TextAnalyze { input, derivation } => {
            let mut runtime = runtime_args.build()?;
            cli::text_analyze::run(&mut runtime, input, derivation).map_err(Into::into)
        }
        Command::TextIngest { input } => {
            let mut runtime = runtime_args.build()?;
            cli::text_ingest::run(&mut runtime, input).map_err(Into::into)
        }
        Command::TextAsk {
            input,
            limit,
            evidence,
        } => {
            let mut runtime = runtime_args.build()?;
            cli::text_ask::run(&mut runtime, input, limit, evidence.into()).map_err(Into::into)
        }
        Command::TextGenerate {
            expression,
            language,
            trace,
        } => cli::semantic_text::generate(
            expression,
            language,
            default_model_root,
            default_language_root,
            trace,
        )
        .map_err(Into::into),
        Command::SemanticRealize {
            expression,
            language,
            trace,
        } => cli::semantic_text::realize(
            expression,
            language,
            default_model_root,
            default_language_root,
            trace,
        )
        .map_err(Into::into),
        Command::TextTranslateText {
            input,
            target_language,
            trace,
        } => {
            let mut runtime = runtime_args.build()?;
            cli::semantic_text::translate_text(
                &mut runtime,
                input,
                target_language,
                default_model_root,
                default_language_root,
                trace,
            )
            .map_err(Into::into)
        }
        Command::DocumentIngest {
            document_id,
            file,
            store,
        } => cli::semantic_context::document_ingest(document_id, file, store).map_err(Into::into),
        Command::DocumentRemove { document_id, store } => {
            cli::semantic_context::document_remove(document_id, store).map_err(Into::into)
        }
        Command::DocumentReplace {
            document_id,
            file,
            store,
        } => cli::semantic_context::document_replace(document_id, file, store).map_err(Into::into),
        Command::DocumentInspect { store } => {
            cli::semantic_context::document_inspect(store).map_err(Into::into)
        }
        Command::DocumentQuery { expression, store } => {
            cli::semantic_context::document_query(expression, store).map_err(Into::into)
        }
        Command::DocumentAnalyze { store, language } => {
            let mut runtime = runtime_args.build()?;
            cli::semantic_context::document_analyze(&mut runtime, store, language)
                .map_err(Into::into)
        }
        Command::ProviderIngest {
            document_id,
            artifact,
            store,
        } => {
            cli::semantic_context::provider_ingest(document_id, artifact, store).map_err(Into::into)
        }
        Command::ConversationTurn {
            turn_id,
            expression,
            state,
        } => {
            cli::semantic_context::conversation_turn(turn_id, expression, state).map_err(Into::into)
        }
        Command::ConversationTextTurn {
            turn_id,
            input,
            state,
        } => {
            let mut runtime = runtime_args.build()?;
            cli::semantic_context::conversation_text_turn(&mut runtime, turn_id, input, state)
                .map_err(Into::into)
        }
        Command::ConversationInspect { state } => {
            cli::semantic_context::conversation_inspect(state).map_err(Into::into)
        }
        Command::ConversationResolve { entity_type, state } => {
            cli::semantic_context::conversation_resolve(entity_type, state).map_err(Into::into)
        }
        Command::LearningPropose {
            observation,
            proposal,
            expected_behavior,
        } => cli::learning::propose(observation, proposal, expected_behavior).map_err(Into::into),
        Command::LearningObserve { observation } => {
            cli::learning::observe(observation).map_err(Into::into)
        }
        Command::LearningApprove {
            overlay,
            proposal_id,
        } => cli::learning::approve(overlay, proposal_id).map_err(Into::into),
        Command::LearningReject {
            overlay,
            proposal_id,
        } => cli::learning::reject(overlay, proposal_id).map_err(Into::into),
        Command::LearningPromote {
            overlay,
            proposal_id,
        } => cli::learning::promote(overlay, proposal_id).map_err(Into::into),
        Command::SessionInspect => {
            let mut runtime = runtime_args.build()?;
            cli::session_inspect::run(&mut runtime).map_err(Into::into)
        }
        Command::SessionClear => {
            let mut runtime = runtime_args.build()?;
            cli::session_clear::run(&mut runtime).map_err(Into::into)
        }
        Command::SessionMigrate { from_schema } => {
            cli::session_migrate::run(&runtime_args.session, &runtime_args.state_dir, from_schema)
        }
        Command::ModelValidate { model_root } => cli::model_validate::run(
            model_root.unwrap_or(default_model_root),
            default_language_root,
        )
        .map_err(CliError::Validation),
        Command::LanguageValidate { language_root } => cli::language_validate::run(
            default_model_root,
            language_root.unwrap_or(default_language_root),
        )
        .map_err(CliError::Validation),
    }
}
