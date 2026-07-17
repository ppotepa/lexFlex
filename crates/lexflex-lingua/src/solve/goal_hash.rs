use crate::solve::goal_canonical::{canonical_semantic_goal, CanonicalRequestGoal};
use crate::solve::goal_validation::validate_goal;
use crate::solve::LinguaGoal;
use lexflex_model::{
    canonical_hash, CanonicalDigest, CanonicalHashError, ConceptCatalog, ConceptId, EntityDefinition,
    EntityId, SemanticExpression, VariableId,
};

pub fn canonical_goal_semantic_hash(
    goal: &LinguaGoal,
    catalog: &ConceptCatalog,
) -> Result<CanonicalDigest, CanonicalHashError> {
    validate_goal(goal, catalog).map_err(|error| CanonicalHashError::Serialization {
        message: error.to_string(),
    })?;
    let canonical = canonical_semantic_goal(goal).map_err(|error| CanonicalHashError::Serialization {
        message: error.to_string(),
    })?;
    canonical_hash(&canonical)
}

pub fn canonical_goal_request_hash(
    goal: &LinguaGoal,
    catalog: &ConceptCatalog,
) -> Result<CanonicalDigest, CanonicalHashError> {
    validate_goal(goal, catalog).map_err(|error| CanonicalHashError::Serialization {
        message: error.to_string(),
    })?;
    let canonical = canonical_semantic_goal(goal).map_err(|error| CanonicalHashError::Serialization {
        message: error.to_string(),
    })?;
    canonical_hash(&CanonicalRequestGoal {
        semantic: canonical,
        evidence_policy: goal.evidence_policy,
        limit: goal.limit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve::{EvidencePolicy, LinguaGoal};
    use crate::types::SemanticType;
    use lexflex_model::{ConceptCatalog, ConceptId, EntityId, SemanticExpression, VariableId};
    use std::collections::BTreeMap;

    fn catalog() -> ConceptCatalog {
        let mut catalog = ConceptCatalog::default();
        catalog.entities.insert(
            EntityId::new_unchecked("PARIS"),
            EntityDefinition {
                id: EntityId::new_unchecked("PARIS"),
                primary_type: ConceptId::new_unchecked("CITY"),
                additional_types: Default::default(),
            },
        );
        catalog
    }

    #[test]
    fn semantic_hash_is_alpha_equivalent_for_free_variables() {
        let left_variable = VariableId::new_unchecked("answer");
        let right_variable = VariableId::new_unchecked("x");

        let left = LinguaGoal {
            expression: SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(left_variable.clone())),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("PREDICATE"),
                    bindings: BTreeMap::new(),
                }),
            },
            variables: BTreeMap::from([(
                left_variable.clone(),
                SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
            )]),
            projection: vec![left_variable],
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: Some(10),
        };

        let right = LinguaGoal {
            expression: SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(right_variable.clone())),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("PREDICATE"),
                    bindings: BTreeMap::new(),
                }),
            },
            variables: BTreeMap::from([(
                right_variable.clone(),
                SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
            )]),
            projection: vec![right_variable],
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: Some(10),
        };

        assert_eq!(
            canonical_goal_semantic_hash(&left, &catalog()),
            canonical_goal_semantic_hash(&right, &catalog())
        );
    }

    #[test]
    fn semantic_hash_ignores_request_policy() {
        let variable = VariableId::new_unchecked("answer");
        let base = LinguaGoal {
            expression: SemanticExpression::Equals {
                left: Box::new(SemanticExpression::Variable(variable.clone())),
                right: Box::new(SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))),
            },
            variables: BTreeMap::from([(
                variable.clone(),
                SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
            )]),
            projection: vec![variable],
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: Some(1),
        };

        let mut changed = base.clone();
        changed.evidence_policy = EvidencePolicy::Required;
        changed.limit = Some(100);

        assert_eq!(
            canonical_goal_semantic_hash(&base, &catalog()),
            canonical_goal_semantic_hash(&changed, &catalog())
        );
        assert_ne!(
            canonical_goal_request_hash(&base, &catalog()),
            canonical_goal_request_hash(&changed, &catalog())
        );
    }

    #[test]
    fn semantic_hash_is_alpha_equivalent_for_bound_variables() {
        let left = LinguaGoal {
            expression: SemanticExpression::Exists {
                variable: VariableId::new_unchecked("x"),
                value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
                body: Box::new(SemanticExpression::Variable(VariableId::new_unchecked("x"))),
            },
            variables: BTreeMap::new(),
            projection: Vec::new(),
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: None,
        };
        let right = LinguaGoal {
            expression: SemanticExpression::Exists {
                variable: VariableId::new_unchecked("y"),
                value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
                body: Box::new(SemanticExpression::Variable(VariableId::new_unchecked("y"))),
            },
            variables: BTreeMap::new(),
            projection: Vec::new(),
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: None,
        };

        assert_eq!(
            canonical_goal_semantic_hash(&left, &catalog()),
            canonical_goal_semantic_hash(&right, &catalog())
        );
    }
}
