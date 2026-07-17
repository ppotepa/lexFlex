use super::*;
use crate::api::outcome::AssertionWriteOutcome;
use crate::api::response::EngineDiagnostic;

impl LexFlexRuntime {
    pub(crate) fn mutate_and_persist<T>(
        &mut self,
        mutation: impl FnOnce(&mut EngineSessionState) -> Result<T, EngineResponse>,
    ) -> Result<T, EngineResponse> {
        let before = self.session.clone();
        let output = match mutation(&mut self.session.state) {
            Ok(output) => output,
            Err(error) => {
                self.session = before;
                return Err(error);
            }
        };
        if let Err(error) = self.session.state.verify(
            &self.session.state.model_hash,
            &self.session.state.language_hash,
        ) {
            self.session = before;
            return Err(EngineResponse::Error {
                code: EngineErrorCode::InternalInvariant,
                message: error.to_string(),
                diagnostics: Vec::new(),
            });
        }
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
        Ok(output)
    }

    pub(crate) fn persist_assertion(
        &mut self,
        assertion: SemanticAssertion,
    ) -> Result<(SemanticAssertion, AssertionWriteOutcome), EngineResponse> {
        let assertion_id = assertion.id.clone();
        let outcome = self.mutate_and_persist(|state| {
            let upsert =
                state
                    .knowledge
                    .upsert(assertion)
                    .map_err(|error| EngineResponse::Error {
                        code: EngineErrorCode::InternalInvariant,
                        message: error.to_string(),
                        diagnostics: Vec::new(),
                    })?;

            Ok(match upsert {
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
            })
        })?;
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
