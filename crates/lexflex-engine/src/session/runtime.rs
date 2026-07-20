use crate::knowledge::index::KnowledgeIndex;
use crate::session::{EngineSessionState, SessionIntegrityError};
use lexflex_model::{CanonicalDigest, ConceptCatalog};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RuntimeSession {
    pub state: EngineSessionState,
    pub(crate) knowledge_index: KnowledgeIndex,
    pub catalog: Arc<ConceptCatalog>,
}

impl RuntimeSession {
    pub(crate) fn try_from_state(
        requested_session_id: &str,
        state: EngineSessionState,
        catalog: Arc<ConceptCatalog>,
        model_hash: &CanonicalDigest,
        language_hash: &CanonicalDigest,
    ) -> Result<Self, SessionIntegrityError> {
        if state.session_id() != requested_session_id {
            return Err(SessionIntegrityError::SessionIdMismatch {
                stored: state.session_id().to_owned(),
                requested: requested_session_id.to_owned(),
            });
        }
        state.verify(model_hash, language_hash, catalog.as_ref())?;
        let knowledge_index = KnowledgeIndex::rebuild(state.knowledge());
        Ok(Self {
            state,
            knowledge_index,
            catalog,
        })
    }
}
