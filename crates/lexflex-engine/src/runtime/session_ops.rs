use super::*;
use crate::runtime::error_mapping::stored_knowledge_error_response;

impl LexFlexRuntime {
    pub(crate) fn handle_inspect(&self) -> EngineResponse {
        EngineResponse::SessionInspection {
            assertion_count: self.session.state.knowledge().len(),
            evidence_count: self.session.state.evidence_count(),
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
            model_hash: self.session.state.model_hash().to_string(),
            language_hash: self.languages.registry_hash.to_string(),
        }
    }

    pub(crate) fn handle_clear(&mut self) -> EngineResponse {
        let catalog = self.session.catalog.clone();
        if let Err(response) = self.mutate_and_persist(|state| {
            state
                .knowledge_mut()
                .clear(catalog.as_ref())
                .map_err(stored_knowledge_error_response)?;
            Ok(())
        }) {
            return response;
        }
        EngineResponse::SessionCleared {
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
        }
    }
}
