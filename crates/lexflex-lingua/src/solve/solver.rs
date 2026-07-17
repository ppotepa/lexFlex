use crate::solve::{
    canonical_goal_semantic_hash, query_solution_sort_key, unify, validate_goal, EvidencePolicy,
    LinguaGoal, SolveError, Substitution, UnificationContext, UnificationMode,
};
use lexflex_model::{AssertionId, CanonicalDigest, ConceptCatalog, Evidence, EvidenceId, SemanticAssertion};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuerySolution {
    pub substitution: Substitution,
    pub evidence: BTreeMap<EvidenceId, Evidence>,
    pub assertion_id: AssertionId,
    pub assertion_hash: CanonicalDigest,
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
        let _goal_hash = canonical_goal_semantic_hash(goal, catalog.as_ref())?;
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
            match unify(
                &goal.expression,
                &assertion.expression,
                &context,
                &mut substitution,
            ) {
                Ok(()) => results.push(QuerySolution {
                    substitution,
                    evidence: assertion.evidence.clone(),
                    assertion_id: assertion.id.clone(),
                    assertion_hash: assertion.canonical_hash.clone(),
                }),
                Err(error) if error.is_candidate_mismatch() => {}
                Err(error) => return Err(SolveError::Unify(error)),
            }
        }
        let mut ordered = Vec::with_capacity(results.len());
        for result in results {
            ordered.push((query_solution_sort_key(&result)?, result));
        }
        ordered.sort_by(|left, right| left.0.cmp(&right.0));
        let mut results: Vec<_> = ordered.into_iter().map(|(_, result)| result).collect();
        results.dedup_by(|left, right| {
            left.assertion_id == right.assertion_id && left.substitution == right.substitution
        });
        if let Some(limit) = goal.limit {
            results.truncate(limit);
        }
        Ok(results)
    }
}
