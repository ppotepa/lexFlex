use crate::api::{
    input::TextInput,
    request::EngineRequest,
    response::EngineResponse,
    text::{TextAnalysis, TextAnalysisAlternative, TextAnalysisKind},
};
use crate::catalog::{LanguageRegistry, ModelLoadError, ModelPackageLoader};
use crate::error::EngineErrorCode;
use crate::session::{EngineSessionState, RuntimeSession};
use lexflex_lingua::solve::EvidencePolicy;
use lexflex_lingua::{ExecutionPolicy, ExpansionMode, LinguaGoal, LinguaProgram};
use lexflex_model::{
    canonical_hash, Evidence, EvidenceId, SemanticAssertion, SemanticExpression, SourceSpan,
    WorldId,
};
use lexflex_parser::{LexicalCompositionParser, ParseBudget, ParseInput, ParseOutput};
use lexflex_store::{SessionStore, SessionStoreError};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

mod formal;
mod operations;

pub mod lingua;

pub(crate) use formal::evaluate_text_expression;
pub use lingua::{EngineError, LinguaRuntime};
pub(crate) use operations::workspace_path;

const SESSION_SCHEMA: u32 = 1;

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
    #[error("session model mismatch: stored={stored}, current={current}")]
    ModelMismatch { stored: String, current: String },
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
        let catalog = Arc::new(model.catalog);
        let model_hash = model.model_hash.clone();
        let lingua = LinguaRuntime::from_catalog(catalog.clone());
        let store: SessionStore<EngineSessionState> =
            SessionStore::new(state_root.as_ref().to_path_buf(), SESSION_SCHEMA);
        let languages = LanguageRegistry::load(language_root.as_ref(), catalog.clone())
            .map_err(RuntimeInitError::Languages)?;
        let state = match store.load(&session_id) {
            Ok(state) => {
                if state.model_hash != model_hash {
                    return Err(RuntimeInitError::ModelMismatch {
                        stored: state.model_hash,
                        current: model_hash,
                    });
                }
                state
            }
            Err(SessionStoreError::NotFound { .. }) => {
                EngineSessionState::new(session_id.clone(), model_hash.clone())
            }
            Err(error) => return Err(RuntimeInitError::Store(error)),
        };
        let mut state = state;
        state.rebuild_knowledge();
        let session = RuntimeSession::new(state, catalog);
        Ok(Self {
            lingua,
            store,
            session,
            languages,
        })
    }
}
