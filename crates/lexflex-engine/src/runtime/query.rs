use super::*;
use crate::runtime::error_mapping::{missing_assertion_response, request_program_error_response};

impl LexFlexRuntime {
    pub(crate) fn handle_query(&self, goal: LinguaGoal) -> EngineResponse {
        let candidates = match self.candidate_assertions(&goal) {
            Ok(candidates) => candidates,
            Err(assertion_id) => {
                return missing_assertion_response(assertion_id);
            }
        };
        match self.lingua.solve(&goal, candidates) {
            Ok(solutions) => EngineResponse::LinguaQueryResult {
                goal,
                solutions,
                snapshot_hash: self.session.state.snapshot_hash().to_string(),
            },
            Err(error) => request_program_error_response(error),
        }
    }

    pub(crate) fn candidate_assertions<'a>(
        &'a self,
        goal: &LinguaGoal,
    ) -> Result<Vec<&'a SemanticAssertion>, lexflex_model::AssertionId> {
        let candidate_ids = self
            .session
            .knowledge_index
            .candidate_ids(goal, self.session.state.knowledge());
        candidate_ids
            .into_iter()
            .map(|id| self.session.state.knowledge().get(&id).ok_or(id))
            .collect()
    }
}
