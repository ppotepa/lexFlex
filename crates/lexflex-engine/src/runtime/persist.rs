use super::*;
use crate::api::outcome::AssertionWriteOutcome;
use crate::api::response::EngineDiagnostic;

impl LexFlexRuntime {
    pub(crate) fn persist_assertion(
        &mut self,
        assertion: SemanticAssertion,
    ) -> Result<(SemanticAssertion, AssertionWriteOutcome), EngineResponse> {
        let assertion_id = assertion.id.clone();
        let before = self.session.clone();
        let outcome = match self.session.state.knowledge.upsert(assertion) {
            crate::knowledge::UpsertOutcome::Inserted { assertion_id } => {
                AssertionWriteOutcome::Inserted { assertion_id }
            }
            crate::knowledge::UpsertOutcome::EvidenceMerged {
                assertion_id,
                added,
            } => AssertionWriteOutcome::EvidenceMerged {
                assertion_id,
                added_evidence: added,
            },
            crate::knowledge::UpsertOutcome::Unchanged { assertion_id } => {
                AssertionWriteOutcome::Unchanged { assertion_id }
            }
        };
        self.session.state.rebuild_knowledge();
        self.session.knowledge_index =
            crate::knowledge::KnowledgeIndex::rebuild(&self.session.state.knowledge);
        if let Err(error) = self
            .store
            .save(&self.session.state.session_id, &self.session.state)
        {
            self.session = before;
            return Err(EngineResponse::Error {
                code: EngineErrorCode::StoreError,
                message: error.to_string(),
                diagnostics: Vec::new(),
            });
        }
        let Some(assertion) = self
            .session
            .state
            .knowledge
            .assertions
            .get(&assertion_id)
            .cloned()
        else {
            return Err(EngineResponse::Error {
                code: EngineErrorCode::InternalInvariant,
                message: format!("missing assertion after upsert: {assertion_id}"),
                diagnostics: vec![EngineDiagnostic::MissingAssertion {
                    assertion_id: assertion_id.clone(),
                }],
            });
        };
        Ok((assertion, outcome))
    }
}
