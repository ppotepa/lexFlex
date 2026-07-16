use crate::solve::{
    goal::canonical_goal_hash, unify, validate_goal, EvidencePolicy, LinguaGoal, SolveError,
    Substitution, UnificationContext, UnificationMode,
};
use lexflex_model::{AssertionId, ConceptCatalog, Evidence, SemanticAssertion};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuerySolution {
    pub substitution: Substitution,
    pub evidence: Vec<Evidence>,
    pub assertion_id: AssertionId,
    pub assertion_hash: String,
}

#[derive(Debug, Default, Clone)]
pub struct LinguaSolver {
    pub max_depth: usize,
}

impl LinguaSolver {
    pub fn solve<'a>(
        &self,
        goal: &LinguaGoal,
        assertions: impl IntoIterator<Item = &'a SemanticAssertion>,
        catalog: Arc<ConceptCatalog>,
    ) -> Result<Vec<QuerySolution>, SolveError> {
        validate_goal(goal, catalog.as_ref())?;
        let _goal_hash = canonical_goal_hash(goal);
        let context = UnificationContext {
            catalog: catalog.clone(),
            variable_types: goal.variables.clone(),
            mode: UnificationMode::Pattern,
            max_depth: self.max_depth.max(1_000),
        };
        let mut results = Vec::new();
        for assertion in assertions {
            if goal
                .world
                .as_ref()
                .is_some_and(|world| world != &assertion.world)
            {
                continue;
            }
            if matches!(goal.evidence_policy, EvidencePolicy::Required)
                && assertion.evidence.is_empty()
            {
                continue;
            }
            let mut substitution = Substitution::default();
            if unify(
                &goal.expression,
                &assertion.expression,
                &context,
                &mut substitution,
            )
            .is_ok()
            {
                results.push(QuerySolution {
                    substitution,
                    evidence: assertion.evidence.clone(),
                    assertion_id: assertion.id.clone(),
                    assertion_hash: assertion.canonical_hash.clone(),
                });
            }
            if goal.limit.is_some_and(|limit| results.len() >= limit) {
                break;
            }
        }
        Ok(results)
    }
}
