use crate::api::{
    input::TextInput,
    request::EngineRequest,
    response::EngineResponse,
    text::{TextAnalysis, TextAnalysisAlternative, TextAnalysisInput, TextAnalysisKind},
};
use crate::catalog::{LanguageRegistry, ModelLoadError, ModelPackageLoader};
use crate::error::EngineErrorCode;
use crate::session::{EngineSessionState, RuntimeSession, SessionIntegrityError, SESSION_SCHEMA};
use lexflex_lingua::solve::EvidencePolicy;
use lexflex_lingua::{ExecutionPolicy, ExpansionMode, LinguaGoal, LinguaProgram};
use lexflex_model::{
    canonical_hash, Evidence, SemanticAssertion, SemanticExpression, SourceSpan, WorldId,
};
use lexflex_parser::ParseOutput;
use lexflex_store::{SessionStore, SessionStoreError};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

mod ambiguity;
mod dispatch;
mod dispatch_helpers;
mod formal_expression;
mod formal_result;
mod mutation;
mod path;
mod persist;
mod query;
mod session_ops;
mod text_analyze;
mod text_ask;
mod text_ingest;

pub mod lingua;

pub use lingua::{EngineError, LinguaRuntime};
pub(crate) use path::workspace_path;

pub struct LexFlexRuntime {
    lingua: LinguaRuntime,
    store: SessionStore<EngineSessionState>,
    session: RuntimeSession,
    languages: LanguageRegistry,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RuntimeInitError {
    #[error("model load error: {0}")]
    Model(#[from] ModelLoadError),
    #[error("session store error: {0}")]
    Store(#[from] SessionStoreError),
    #[error("session integrity error: {0}")]
    Integrity(#[from] SessionIntegrityError),
    #[error("language registry error: {0}")]
    Languages(#[from] crate::catalog::LanguageRegistryError),
}

impl LexFlexRuntime {
    pub fn with_session(session_id: impl Into<String>) -> Result<Self, RuntimeInitError> {
        Self::with_session_and_roots(
            session_id,
            ".lexflex",
            workspace_path("data/model"),
            workspace_path("data/languages"),
        )
    }

    pub fn with_session_and_root(
        session_id: impl Into<String>,
        state_root: impl AsRef<Path>,
    ) -> Result<Self, RuntimeInitError> {
        Self::with_session_and_roots(
            session_id,
            state_root,
            workspace_path("data/model"),
            workspace_path("data/languages"),
        )
    }

    pub fn with_session_and_roots(
        session_id: impl Into<String>,
        state_root: impl AsRef<Path>,
        model_root: impl AsRef<Path>,
        language_root: impl AsRef<Path>,
    ) -> Result<Self, RuntimeInitError> {
        let session_id = session_id.into();
        let model = ModelPackageLoader
            .load(model_root.as_ref())
            .map_err(RuntimeInitError::Model)?;
        let catalog = Arc::new(model.catalog.clone());
        let model_hash = model.model_hash.clone();
        let lingua = LinguaRuntime::from_model(&model);
        let store: SessionStore<EngineSessionState> =
            SessionStore::new(state_root.as_ref().to_path_buf(), SESSION_SCHEMA);
        let languages = LanguageRegistry::load(language_root.as_ref(), catalog.clone())
            .map_err(RuntimeInitError::Languages)?;
        let language_hash = languages.registry_hash.clone();
        let state = match store.load(&session_id) {
            Ok(state) => {
                state.verify(&model_hash, &language_hash)?;
                state
            }
            Err(SessionStoreError::NotFound { .. }) => {
                EngineSessionState::new(session_id.clone(), model_hash.clone(), language_hash)
                    .map_err(|error| {
                        RuntimeInitError::Store(SessionStoreError::Serde {
                            path: state_root.as_ref().join(format!("{session_id}.json")),
                            message: error.to_string(),
                        })
                    })?
            }
            Err(error) => return Err(RuntimeInitError::Store(error)),
        };
        let session = RuntimeSession::new(state, catalog);
        Ok(Self {
            lingua,
            store,
            session,
            languages,
        })
    }
}
