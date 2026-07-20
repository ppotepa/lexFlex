use crate::cli::output::{print_json, CliExit};
use lexflex_engine::api::request::EngineRequest;
use lexflex_engine::api::response::EngineResponse;
use lexflex_engine::catalog::ModelPackageLoader;
use lexflex_engine::LexFlexRuntime;
use lexflex_language::LanguagePackageLoader;
use lexflex_model::{LanguageId, SemanticExpression};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
struct TranslationCliOutput {
    text: String,
    trace: Option<Vec<lexflex_generation::GenerationTrace>>,
    source_language: LanguageId,
    target_language: LanguageId,
    kind: lexflex_engine::api::text::TextAnalysisKind,
    source_semantic_hash: String,
    target_semantic_hash: String,
}

fn load_expression(path: &Path) -> Result<SemanticExpression, CliExit> {
    let text = crate::cli::input::read_file_bounded(path)?;
    serde_json::from_str(&text).map_err(|error| {
        CliExit::Local(crate::cli::input_error::CliLocalError::Io {
            path: Some(path.to_owned()),
            message: format!("invalid semantic expression JSON: {error}"),
        })
    })
}

fn translation_semantic_hash(
    runtime: &LexFlexRuntime,
    analysis: &lexflex_engine::api::text::TextAnalysis,
) -> Result<String, CliExit> {
    if analysis.kind() == lexflex_engine::api::text::TextAnalysisKind::Goal {
        let goal = lexflex_lingua::LinguaGoal {
            expression: analysis.canonical_expression().clone(),
            variables: analysis.variables().clone(),
            projection: analysis.projection().to_vec(),
            evidence_policy: lexflex_lingua::EvidencePolicy::Ignore,
            world: None,
            limit: None,
        };
        return lexflex_lingua::canonical_goal_semantic_hash(&goal, runtime.catalog())
            .map(|hash| hash.to_string())
            .map_err(|error| CliExit::Engine {
                code: lexflex_engine::error::EngineErrorCode::InternalInvariant,
                message: error.to_string(),
            });
    }
    Ok(analysis.canonical_hash().to_string())
}

fn run_semantic_text(
    expression_path: PathBuf,
    language: String,
    model_root: PathBuf,
    language_root: PathBuf,
    include_trace: bool,
) -> Result<(), CliExit> {
    let expression = load_expression(&expression_path)?;
    let language_id = LanguageId::new(language).map_err(|error| {
        CliExit::Input(crate::cli::input_error::CliInputError::InvalidLanguage(
            error,
        ))
    })?;
    let package = ModelPackageLoader
        .load(&model_root)
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::ModelError,
            message: error.to_string(),
        })?;
    let model = LanguagePackageLoader
        .load(&language_root.join(language_id.as_str()), &package.catalog)
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::ModelError,
            message: error.to_string(),
        })?;
    let result = lexflex_generation::generate(
        &lexflex_generation::GenerationRequest {
            expression,
            include_trace,
        },
        &model,
    )
    .map_err(|error| CliExit::Engine {
        code: lexflex_engine::error::EngineErrorCode::InvalidProgram,
        message: error.to_string(),
    })?;
    print_json(&result).map_err(CliExit::Internal)
}

pub fn generate(
    expression: PathBuf,
    language: String,
    model_root: PathBuf,
    language_root: PathBuf,
    trace: bool,
) -> Result<(), CliExit> {
    run_semantic_text(expression, language, model_root, language_root, trace)
}

pub fn realize(
    expression: PathBuf,
    language: String,
    model_root: PathBuf,
    language_root: PathBuf,
    trace: bool,
) -> Result<(), CliExit> {
    run_semantic_text(expression, language, model_root, language_root, trace)
}

pub fn translate_text(
    runtime: &mut LexFlexRuntime,
    input: crate::cli::input::TextInputArgs,
    target_language: String,
    _model_root: PathBuf,
    _language_root: PathBuf,
    include_trace: bool,
) -> Result<(), CliExit> {
    let input = input.into_text_input()?;
    let response = runtime.handle(EngineRequest::AnalyzeText {
        input,
        include_derivation: include_trace,
    });
    let analysis = match response {
        EngineResponse::TextAnalyzed { analysis, .. } => analysis,
        other => {
            return crate::cli::output::print_response_and_check(&other).map(|_| ());
        }
    };
    let target_id = LanguageId::new(target_language).map_err(|error| {
        CliExit::Input(crate::cli::input_error::CliInputError::InvalidLanguage(
            error,
        ))
    })?;
    let target = runtime
        .language_model(&target_id)
        .ok_or_else(|| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::ModelError,
            message: format!("language package not loaded: {target_id}"),
        })?;
    let translated = lexflex_generation::realize_semantic(
        &lexflex_generation::SemanticRealizationRequest {
            expression: analysis.canonical_expression().clone(),
            kind: match analysis.kind() {
                lexflex_engine::api::text::TextAnalysisKind::Assertion => {
                    lexflex_generation::SemanticRealizationKind::Assertion
                }
                lexflex_engine::api::text::TextAnalysisKind::Goal => {
                    lexflex_generation::SemanticRealizationKind::Goal
                }
            },
            projection: analysis.projection().to_vec(),
            include_trace,
        },
        target,
    )
    .map_err(|error| CliExit::Engine {
        code: lexflex_engine::error::EngineErrorCode::InvalidProgram,
        message: error.to_string(),
    })?;

    let target_response = runtime.handle(EngineRequest::AnalyzeText {
        input: lexflex_engine::api::input::TextInput {
            source_id: format!("translation-target:{}", analysis.canonical_hash()),
            language: target_id.clone(),
            text: translated.text.clone(),
        },
        include_derivation: include_trace,
    });
    let target_analysis = match target_response {
        EngineResponse::TextAnalyzed { analysis, .. } => analysis,
        other => {
            return crate::cli::output::print_response_and_check(&other).map(|_| ());
        }
    };
    if target_analysis.kind() != analysis.kind() {
        return Err(CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::InternalInvariant,
            message: "translation changed assertion/goal kind".into(),
        });
    }
    let source_semantic_hash = translation_semantic_hash(runtime, &analysis)?;
    let target_semantic_hash = translation_semantic_hash(runtime, &target_analysis)?;
    if source_semantic_hash != target_semantic_hash {
        return Err(CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::InternalInvariant,
            message: format!(
                "translation semantic hash mismatch: source={}, target={}",
                source_semantic_hash, target_semantic_hash
            ),
        });
    }

    print_json(&TranslationCliOutput {
        text: translated.text,
        trace: translated.trace,
        source_language: analysis.language().clone(),
        target_language: target_id,
        kind: analysis.kind(),
        source_semantic_hash,
        target_semantic_hash,
    })
    .map_err(CliExit::Internal)?;
    Ok(())
}
