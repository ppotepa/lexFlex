use crate::knowledge::KnowledgeIndex;
use crate::session::EngineSessionState;
use lexflex_model::ConceptCatalog;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RuntimeSession {
    pub state: EngineSessionState,
    pub(crate) knowledge_index: KnowledgeIndex,
    pub catalog: Arc<ConceptCatalog>,
}

impl RuntimeSession {
    pub fn new(state: EngineSessionState, catalog: Arc<ConceptCatalog>) -> Self {
        let knowledge_index = KnowledgeIndex::rebuild(&state.knowledge);
        Self {
            state,
            knowledge_index,
            catalog,
        }
    }
}
