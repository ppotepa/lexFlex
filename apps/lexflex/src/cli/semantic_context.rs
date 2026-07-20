use crate::cli::input::TextInputArgs;
use crate::cli::input_error::CliLocalError;
use crate::cli::output::{print_json, CliExit};
use lexflex_conversation::{ConversationState, Turn};
use lexflex_documents::ingest;
use lexflex_engine::{api::request::EngineRequest, runtime::LexFlexRuntime};
use lexflex_language::LanguageId;
use lexflex_model::SemanticExpression;
use std::path::{Path, PathBuf};

fn read_expression(path: &Path) -> Result<SemanticExpression, CliExit> {
    let text = crate::cli::input::read_file_bounded(path)?;
    serde_json::from_str(&text).map_err(|error| {
        CliExit::Local(CliLocalError::Io {
            path: Some(path.to_path_buf()),
            message: format!("invalid semantic expression JSON: {error}"),
        })
    })
}

pub fn document_ingest(
    document_id: String,
    file: PathBuf,
    store_path: Option<PathBuf>,
) -> Result<(), CliExit> {
    let text = crate::cli::input::read_file_bounded(&file)?;
    let document = ingest(document_id, &text).map_err(|error| CliExit::Engine {
        code: lexflex_engine::error::EngineErrorCode::InvalidEvidence,
        message: error.to_string(),
    })?;
    if let Some(path) = store_path {
        let mut store = if path.exists() {
            lexflex_documents::DocumentStore::load_from_file(&path)
        } else {
            Ok(lexflex_documents::DocumentStore::default())
        }
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::Integrity,
            message: error.to_string(),
        })?;
        store
            .insert(document.clone())
            .map_err(|error| CliExit::Engine {
                code: lexflex_engine::error::EngineErrorCode::InvalidEvidence,
                message: error.to_string(),
            })?;
        store.save_to_file(&path).map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::StoreError,
            message: error.to_string(),
        })?;
    }
    print_json(&document).map_err(CliExit::Internal)
}

pub fn document_remove(document_id: String, store_path: PathBuf) -> Result<(), CliExit> {
    let mut store =
        lexflex_documents::DocumentStore::load_from_file(&store_path).map_err(|error| {
            CliExit::Engine {
                code: lexflex_engine::error::EngineErrorCode::Integrity,
                message: error.to_string(),
            }
        })?;
    let removed = store
        .remove(&document_id)
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::InvalidEvidence,
            message: error.to_string(),
        })?;
    store
        .save_to_file(&store_path)
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::StoreError,
            message: error.to_string(),
        })?;
    print_json(&removed).map_err(CliExit::Internal)
}

pub fn document_replace(
    document_id: String,
    file: PathBuf,
    store_path: PathBuf,
) -> Result<(), CliExit> {
    let text = crate::cli::input::read_file_bounded(&file)?;
    let mut store =
        lexflex_documents::DocumentStore::load_from_file(&store_path).map_err(|error| {
            CliExit::Engine {
                code: lexflex_engine::error::EngineErrorCode::Integrity,
                message: error.to_string(),
            }
        })?;
    store
        .replace(&document_id, &text)
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::InvalidEvidence,
            message: error.to_string(),
        })?;
    let document = store.get(&document_id).cloned().ok_or_else(|| {
        CliExit::Internal(
            crate::cli::internal_error::CliInternalError::UnexpectedEngineResponse {
                message: "replaced document disappeared from verified store".into(),
            },
        )
    })?;
    store
        .save_to_file(&store_path)
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::StoreError,
            message: error.to_string(),
        })?;
    print_json(&document).map_err(CliExit::Internal)
}

pub fn document_inspect(store_path: PathBuf) -> Result<(), CliExit> {
    let store = lexflex_documents::DocumentStore::load_from_file(&store_path).map_err(|error| {
        CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::Integrity,
            message: error.to_string(),
        }
    })?;
    print_json(&store.documents().collect::<Vec<_>>()).map_err(CliExit::Internal)
}

pub fn document_query(expression_path: PathBuf, store_path: PathBuf) -> Result<(), CliExit> {
    let expression = read_expression(&expression_path)?;
    let store = lexflex_documents::DocumentStore::load_from_file(&store_path).map_err(|error| {
        CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::Integrity,
            message: error.to_string(),
        }
    })?;
    let matches = store
        .find_expression(&expression)
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::Canonicalization,
            message: error.to_string(),
        })?
        .into_iter()
        .map(|(document, segment)| {
            serde_json::json!({
                "document_id": document.document_id,
                "source_hash": document.source_hash,
                "segment_id": segment.segment_id,
                "start": segment.start,
                "end": segment.end,
                "text": segment.text,
            })
        })
        .collect::<Vec<_>>();
    print_json(&matches).map_err(CliExit::Internal)
}

pub fn document_analyze(
    runtime: &mut LexFlexRuntime,
    store_path: PathBuf,
    language: String,
) -> Result<(), CliExit> {
    let store = lexflex_documents::DocumentStore::load_from_file(&store_path).map_err(|error| {
        CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::Integrity,
            message: error.to_string(),
        }
    })?;
    let mut candidate = store.clone();
    let segments = store
        .documents()
        .flat_map(|document| {
            document.segments.iter().map(|segment| {
                (
                    document.document_id.clone(),
                    segment.segment_id.clone(),
                    segment.text.clone(),
                )
            })
        })
        .collect::<Vec<_>>();
    let language = LanguageId::new_unchecked(language);
    for (document_id, segment_id, text) in segments {
        let response = runtime.handle(EngineRequest::AnalyzeText {
            input: lexflex_engine::api::input::TextInput {
                source_id: segment_id.clone(),
                language: language.clone(),
                text,
            },
            include_derivation: false,
        });
        let analysis = match response {
            lexflex_engine::api::response::EngineResponse::TextAnalyzed { analysis, .. } => {
                analysis
            }
            lexflex_engine::api::response::EngineResponse::Error { code, message, .. } => {
                return Err(CliExit::Engine { code, message });
            }
            lexflex_engine::api::response::EngineResponse::TextAmbiguous { .. } => {
                return Err(CliExit::Engine {
                    code: lexflex_engine::error::EngineErrorCode::InvalidAssertion,
                    message: format!("document segment {segment_id} has ambiguous analysis"),
                });
            }
            lexflex_engine::api::response::EngineResponse::TextNotParsed { diagnostics } => {
                return Err(CliExit::Engine {
                    code: lexflex_engine::error::EngineErrorCode::InvalidProgram,
                    message: format!(
                        "document segment {segment_id} was not parsed: {diagnostics:?}"
                    ),
                });
            }
            other => {
                return Err(CliExit::Internal(
                    crate::cli::internal_error::CliInternalError::UnexpectedEngineResponse {
                        message: format!("unexpected document analysis response: {other:?}"),
                    },
                ));
            }
        };
        candidate
            .attach_analysis(
                &document_id,
                &segment_id,
                analysis.canonical_expression().clone(),
            )
            .map_err(|error| CliExit::Engine {
                code: lexflex_engine::error::EngineErrorCode::Integrity,
                message: error.to_string(),
            })?;
    }
    candidate
        .save_to_file(&store_path)
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::StoreError,
            message: error.to_string(),
        })?;
    print_json(&candidate).map_err(CliExit::Internal)
}

pub fn provider_ingest(
    document_id: String,
    artifact_path: PathBuf,
    store_path: PathBuf,
) -> Result<(), CliExit> {
    let artifact_text = crate::cli::input::read_file_bounded(&artifact_path)?;
    let artifact: lexflex_provider_llm::ProviderArtifact = serde_json::from_str(&artifact_text)
        .map_err(|error| {
            CliExit::Local(CliLocalError::Io {
                path: Some(artifact_path),
                message: format!("invalid provider artifact JSON: {error}"),
            })
        })?;
    let mut store = if store_path.exists() {
        lexflex_documents::DocumentStore::load_from_file(&store_path)
    } else {
        Ok(lexflex_documents::DocumentStore::default())
    }
    .map_err(|error| CliExit::Engine {
        code: lexflex_engine::error::EngineErrorCode::Integrity,
        message: error.to_string(),
    })?;
    let document =
        lexflex_provider_llm::ingest_artifact_transactionally(&mut store, artifact, document_id)
            .map_err(|error| CliExit::Engine {
                code: lexflex_engine::error::EngineErrorCode::InvalidEvidence,
                message: error.to_string(),
            })?;
    store
        .save_to_file(&store_path)
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::StoreError,
            message: error.to_string(),
        })?;
    print_json(&document).map_err(CliExit::Internal)
}

pub fn conversation_turn(
    turn_id: String,
    expression: PathBuf,
    state_path: Option<PathBuf>,
) -> Result<(), CliExit> {
    let expression = read_expression(&expression)?;
    let mut state = match state_path.as_ref() {
        Some(path) if path.exists() => ConversationState::load_from_file(path),
        _ => Ok(ConversationState::default()),
    }
    .map_err(|error| CliExit::Engine {
        code: lexflex_engine::error::EngineErrorCode::Integrity,
        message: error.to_string(),
    })?;
    state
        .append_turn(Turn {
            turn_id,
            expression,
        })
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::InvalidProgram,
            message: error.to_string(),
        })?;
    if let Some(path) = state_path {
        state.save_to_file(&path).map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::StoreError,
            message: error.to_string(),
        })?;
    }
    state.verify().map_err(|error| {
        CliExit::Internal(
            crate::cli::internal_error::CliInternalError::UnexpectedEngineResponse {
                message: error.to_string(),
            },
        )
    })?;
    print_json(&state).map_err(CliExit::Internal)
}

pub fn conversation_text_turn(
    runtime: &mut LexFlexRuntime,
    turn_id: String,
    input: TextInputArgs,
    state_path: Option<PathBuf>,
) -> Result<(), CliExit> {
    let input = input.into_text_input()?;
    let response = runtime.handle(EngineRequest::AnalyzeText {
        input,
        include_derivation: false,
    });
    let expression = match response {
        lexflex_engine::api::response::EngineResponse::TextAnalyzed { analysis, .. } => {
            analysis.canonical_expression().clone()
        }
        lexflex_engine::api::response::EngineResponse::Error { code, message, .. } => {
            return Err(CliExit::Engine { code, message });
        }
        lexflex_engine::api::response::EngineResponse::TextAmbiguous { .. } => {
            return Err(CliExit::Engine {
                code: lexflex_engine::error::EngineErrorCode::InvalidAssertion,
                message: "conversation turn has ambiguous analysis".into(),
            });
        }
        lexflex_engine::api::response::EngineResponse::TextNotParsed { diagnostics } => {
            return Err(CliExit::Engine {
                code: lexflex_engine::error::EngineErrorCode::InvalidProgram,
                message: format!("conversation turn was not parsed: {diagnostics:?}"),
            });
        }
        other => {
            return Err(CliExit::Internal(
                crate::cli::internal_error::CliInternalError::UnexpectedEngineResponse {
                    message: format!("unexpected conversation response: {other:?}"),
                },
            ));
        }
    };
    let mut state = match state_path.as_ref() {
        Some(path) if path.exists() => ConversationState::load_from_file(path),
        _ => Ok(ConversationState::default()),
    }
    .map_err(|error| CliExit::Engine {
        code: lexflex_engine::error::EngineErrorCode::Integrity,
        message: error.to_string(),
    })?;
    state
        .append_turn(Turn {
            turn_id,
            expression,
        })
        .map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::InvalidProgram,
            message: error.to_string(),
        })?;
    if let Some(path) = state_path {
        state.save_to_file(&path).map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::StoreError,
            message: error.to_string(),
        })?;
    }
    print_json(&state).map_err(CliExit::Internal)
}

pub fn conversation_inspect(state_path: PathBuf) -> Result<(), CliExit> {
    let state =
        ConversationState::load_from_file(&state_path).map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::Integrity,
            message: error.to_string(),
        })?;
    print_json(&state).map_err(CliExit::Internal)
}

pub fn conversation_resolve(entity_type: String, state_path: PathBuf) -> Result<(), CliExit> {
    let state =
        ConversationState::load_from_file(&state_path).map_err(|error| CliExit::Engine {
            code: lexflex_engine::error::EngineErrorCode::Integrity,
            message: error.to_string(),
        })?;
    let resolution = state.resolve_entity(&entity_type);
    print_json(&resolution).map_err(CliExit::Internal)
}
