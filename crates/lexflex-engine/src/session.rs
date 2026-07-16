use crate::knowledge::{KnowledgeIndex, KnowledgeSnapshot};
use lexflex_model::ConceptCatalog;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const SESSION_SCHEMA: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineSessionState {
    pub schema: u32,
    pub session_id: String,
    pub knowledge: KnowledgeSnapshot,
    pub model_hash: String,
    pub language_hash: String,
}

impl EngineSessionState {
    pub fn new(
        session_id: impl Into<String>,
        model_hash: impl Into<String>,
        language_hash: impl Into<String>,
    ) -> Self {
        Self {
            schema: SESSION_SCHEMA,
            session_id: session_id.into(),
            knowledge: KnowledgeSnapshot::new(),
            model_hash: model_hash.into(),
            language_hash: language_hash.into(),
        }
    }

    pub fn rebuild_hash(&mut self) {
        self.knowledge.rebuild_hash();
    }

    pub fn rebuild_knowledge(&mut self) {
        self.rebuild_hash();
    }

    pub fn evidence_count(&self) -> usize {
        self.knowledge.evidence_count()
    }

    pub fn snapshot_hash(&self) -> &str {
        &self.knowledge.snapshot_hash
    }
}

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
