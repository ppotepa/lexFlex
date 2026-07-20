use crate::api::{
    input::TextInput,
    request::EngineRequest,
    response::EngineResponse,
    text::{TextAnalysis, TextAnalysisAlternative, TextAnalysisInput, TextAnalysisKind},
};
use crate::catalog::{LanguageRegistry, ModelLoadError, ModelPackageLoader};
use crate::error::EngineErrorCode;
use crate::session::{
    EngineSessionState, RuntimeSession, SessionIntegrityError, SESSION_RECORD_SCHEMA,
};
use lexflex_lingua::solve::EvidencePolicy;
use lexflex_lingua::{ExecutionPolicy, ExpansionMode, LinguaGoal, LinguaProgram};
use lexflex_model::{canonical_hash, SemanticAssertion, SemanticExpression, SourceSpan, WorldId};
use lexflex_parser::ParseOutput;
use lexflex_store::{validate_session_id, SessionPersistence, SessionStore, SessionStoreError};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

mod alternative_failure;
mod ambiguity;
mod diagnostics;
mod dispatch;
mod dispatch_helpers;
mod error_mapping;
mod evidence_error_mapping;
mod formal_error_mapping;
mod formal_expression;
mod formal_identity;
mod formal_result;
mod path;
mod persist;
mod query;
mod session_ops;
mod text_analyze;
mod text_ask;
mod text_budget;
mod text_evidence;
mod text_ingest;

pub mod lingua;

pub use lingua::{EngineError, LinguaRuntime};
pub(crate) use path::workspace_path;
pub use text_budget::{TextRuntimeBudget, TextRuntimeBudgetError};

pub struct LexFlexRuntime {
    lingua: LinguaRuntime,
    store: Box<dyn SessionPersistence<EngineSessionState>>,
    session: RuntimeSession,
    languages: LanguageRegistry,
    text_budget: TextRuntimeBudget,
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
    pub fn catalog(&self) -> &Arc<lexflex_model::ConceptCatalog> {
        self.lingua.catalog()
    }

    pub fn language_model(
        &self,
        language: &lexflex_model::LanguageId,
    ) -> Option<&Arc<lexflex_language::LanguageModel>> {
        self.languages.get(language)
    }

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
        validate_session_id(&session_id).map_err(RuntimeInitError::Store)?;
        let model = ModelPackageLoader
            .load(model_root.as_ref())
            .map_err(RuntimeInitError::Model)?;
        let lingua = LinguaRuntime::try_from_model(&model)
            .map_err(|error| RuntimeInitError::Model(ModelLoadError::ProgramCompile(error)))?;
        let store: SessionStore<EngineSessionState> =
            SessionStore::new(state_root.as_ref().to_path_buf(), SESSION_RECORD_SCHEMA);
        Self::with_loaded_persistence(session_id, state_root, language_root, model, lingua, store)
    }

    pub fn with_persistence(
        session_id: impl Into<String>,
        state_root: impl AsRef<Path>,
        model_root: impl AsRef<Path>,
        language_root: impl AsRef<Path>,
        store: Box<dyn SessionPersistence<EngineSessionState>>,
    ) -> Result<Self, RuntimeInitError> {
        let session_id = session_id.into();
        validate_session_id(&session_id).map_err(RuntimeInitError::Store)?;
        let model = ModelPackageLoader
            .load(model_root.as_ref())
            .map_err(RuntimeInitError::Model)?;
        let lingua = LinguaRuntime::try_from_model(&model)
            .map_err(|error| RuntimeInitError::Model(ModelLoadError::ProgramCompile(error)))?;
        Self::with_loaded_persistence(session_id, state_root, language_root, model, lingua, store)
    }

    fn with_loaded_persistence(
        session_id: String,
        _state_root: impl AsRef<Path>,
        language_root: impl AsRef<Path>,
        model: crate::catalog::LoadedModelPackage,
        lingua: LinguaRuntime,
        store: impl SessionPersistence<EngineSessionState> + 'static,
    ) -> Result<Self, RuntimeInitError> {
        let catalog = model.catalog.clone();
        let model_hash = model.model_hash.clone();
        let languages = LanguageRegistry::load(language_root.as_ref(), catalog.clone())
            .map_err(RuntimeInitError::Languages)?;
        let language_hash = languages.registry_hash.clone();
        let state = match store.load(&session_id) {
            Ok(state) => state,
            Err(SessionStoreError::NotFound { .. }) => EngineSessionState::new(
                session_id.clone(),
                model_hash.clone(),
                language_hash.clone(),
            )
            .map_err(|error| match error {
                crate::session::SessionStateError::SessionId(error) => {
                    RuntimeInitError::Store(error)
                }
                crate::session::SessionStateError::Knowledge(error) => {
                    RuntimeInitError::Integrity(SessionIntegrityError::Knowledge(error))
                }
            })?,
            Err(
                error @ (SessionStoreError::Serde { .. }
                | SessionStoreError::SchemaMismatch { .. }
                | SessionStoreError::SessionIdMismatch { .. }),
            ) => {
                return Err(RuntimeInitError::Integrity(
                    SessionIntegrityError::StoredPayload(error),
                ));
            }
            Err(error) => return Err(RuntimeInitError::Store(error)),
        };
        let session = RuntimeSession::try_from_state(
            &session_id,
            state,
            catalog,
            &model_hash,
            &language_hash,
        )?;
        Ok(Self {
            lingua,
            store: Box::new(store),
            session,
            languages,
            text_budget: TextRuntimeBudget::default(),
        })
    }

    pub fn set_text_budget(&mut self, budget: TextRuntimeBudget) {
        self.text_budget = budget;
    }
}
