use super::*;

impl LexFlexRuntime {
    pub(crate) fn handle_inspect(&self) -> EngineResponse {
        EngineResponse::SessionInspection {
            assertion_count: self.session.state.knowledge.assertions.len(),
            evidence_count: self.session.state.evidence_count(),
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
            model_hash: self.session.state.model_hash.clone(),
            language_hash: self.languages.registry_hash.clone(),
        }
    }

    pub(crate) fn handle_clear(&mut self) -> EngineResponse {
        let before = self.session.clone();
        self.session.state.knowledge.assertions.clear();
        self.session.state.rebuild_knowledge();
        self.session.knowledge_index =
            crate::knowledge::KnowledgeIndex::rebuild(&self.session.state.knowledge);
        if let Err(error) = self
            .store
            .save(&self.session.state.session_id, &self.session.state)
        {
            self.session = before;
            return EngineResponse::Error {
                code: EngineErrorCode::StoreError,
                message: error.to_string(),
                diagnostics: Vec::new(),
            };
        }
        EngineResponse::SessionCleared {
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
        }
    }
}
