use crate::solve::{
    goal_hash::canonical_goal_semantic_hash_validated, query_solution_sort_key, unify,
    validate_goal, EvidencePolicy, LinguaGoal, SolveError, Substitution, UnificationContext,
    UnificationMode, UnifyOutcome,
};
use lexflex_model::{AssertionId, CanonicalDigest, ConceptCatalog, EvidenceSet, SemanticAssertion};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuerySolution {
    pub substitution: Substitution,
    pub evidence: EvidenceSet,
    pub assertion_id: AssertionId,
    pub assertion_hash: CanonicalDigest,
}

#[derive(Debug, Clone)]
pub struct LinguaSolver {
    pub max_depth: usize,
}

impl Default for LinguaSolver {
    fn default() -> Self {
        Self { max_depth: 1_000 }
    }
}

impl LinguaSolver {
    pub fn solve<'a>(
        &self,
        goal: &LinguaGoal,
        assertions: impl IntoIterator<Item = &'a SemanticAssertion>,
        catalog: Arc<ConceptCatalog>,
    ) -> Result<Vec<QuerySolution>, SolveError> {
        validate_goal(goal, catalog.as_ref())?;
        let _goal_hash = canonical_goal_semantic_hash_validated(goal)?;
        let context = UnificationContext {
            catalog: catalog.clone(),
            variable_types: goal.variables.clone(),
            mode: UnificationMode::Pattern,
            max_depth: self.max_depth,
        };
        let mut results = Vec::new();
        for assertion in assertions {
            assertion.verify_with_catalog(catalog.as_ref())?;
            if goal
                .world
                .as_ref()
                .is_some_and(|world| world != assertion.world())
            {
                continue;
            }
            if matches!(goal.evidence_policy, EvidencePolicy::Required)
                && assertion.evidence().is_empty()
            {
                continue;
            }
            let mut substitution = Substitution::default();
            match unify(
                &goal.expression,
                assertion.expression(),
                &context,
                &mut substitution,
            ) {
                Ok(UnifyOutcome::Matched) => results.push(QuerySolution {
                    substitution,
                    evidence: assertion.evidence().clone(),
                    assertion_id: assertion.id().clone(),
                    assertion_hash: assertion.canonical_hash().clone(),
                }),
                Ok(UnifyOutcome::Mismatch(_)) => {}
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
