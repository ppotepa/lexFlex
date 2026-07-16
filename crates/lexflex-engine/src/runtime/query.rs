use super::*;
use crate::api::response::EngineDiagnostic;

impl LexFlexRuntime {
    pub(crate) fn handle_query(&self, goal: LinguaGoal) -> EngineResponse {
        let candidates = match self.candidate_assertions(&goal) {
            Ok(candidates) => candidates,
            Err(assertion_id) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InternalInvariant,
                    message: format!("missing indexed assertion: {assertion_id}"),
                    diagnostics: vec![EngineDiagnostic::MissingAssertion { assertion_id }],
                };
            }
        };
        match self.lingua.solve(&goal, candidates) {
            Ok(solutions) => EngineResponse::LinguaQueryResult {
                goal,
                solutions,
                snapshot_hash: self.session.state.snapshot_hash().to_string(),
            },
            Err(error) => EngineResponse::Error {
                code: EngineErrorCode::InvalidGoal,
                message: error.to_string(),
                diagnostics: Vec::new(),
            },
        }
    }

    pub(crate) fn candidate_assertions<'a>(
        &'a self,
        goal: &LinguaGoal,
    ) -> Result<Vec<&'a SemanticAssertion>, lexflex_model::AssertionId> {
        let candidate_ids = self
            .session
            .knowledge_index
            .candidate_ids(goal, &self.session.state.knowledge);
        candidate_ids
            .into_iter()
            .map(|id| self.session.state.knowledge.assertions.get(&id).ok_or(id))
            .collect()
    }
}
