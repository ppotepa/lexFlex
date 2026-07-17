use super::*;

impl LexFlexRuntime {
    pub(crate) fn handle_inspect(&self) -> EngineResponse {
        EngineResponse::SessionInspection {
            assertion_count: self.session.state.knowledge.assertions.len(),
            evidence_count: self.session.state.evidence_count(),
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
            model_hash: self.session.state.model_hash.to_string(),
            language_hash: self.languages.registry_hash.to_string(),
        }
    }

    pub(crate) fn handle_clear(&mut self) -> EngineResponse {
        if let Err(response) = self.mutate_and_persist(|state| {
            state.knowledge.assertions.clear();
            state
                .rebuild_knowledge()
                .map_err(|error| EngineResponse::Error {
                    code: EngineErrorCode::InternalInvariant,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                })?;
            Ok(())
        }) {
            return response;
        }
        EngineResponse::SessionCleared {
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
        }
    }
}
