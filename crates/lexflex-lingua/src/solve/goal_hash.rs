use crate::solve::goal_canonical::{canonical_semantic_goal, CanonicalRequestGoal};
use crate::solve::goal_validation::validate_goal;
use crate::solve::{GoalCanonicalizationError, LinguaGoal};
use lexflex_model::{canonical_hash, CanonicalDigest, ConceptCatalog};

pub fn canonical_goal_semantic_hash(
    goal: &LinguaGoal,
    catalog: &ConceptCatalog,
) -> Result<CanonicalDigest, GoalCanonicalizationError> {
    validate_goal(goal, catalog)?;
    canonical_goal_semantic_hash_validated(goal)
}

pub(crate) fn canonical_goal_semantic_hash_validated(
    goal: &LinguaGoal,
) -> Result<CanonicalDigest, GoalCanonicalizationError> {
    let canonical = canonical_semantic_goal(goal)?;
    Ok(canonical_hash(&canonical)?)
}

pub fn canonical_goal_request_hash(
    goal: &LinguaGoal,
    catalog: &ConceptCatalog,
) -> Result<CanonicalDigest, GoalCanonicalizationError> {
    validate_goal(goal, catalog)?;
    let canonical = canonical_semantic_goal(goal)?;
    Ok(canonical_hash(&CanonicalRequestGoal {
        semantic: canonical,
        evidence_policy: goal.evidence_policy,
        limit: goal.limit,
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve::{EvidencePolicy, LinguaGoal};
    use crate::types::SemanticType;
    use lexflex_model::{
        ConceptCatalog, ConceptId, ConceptKind, ConceptSchema, EntityDefinition, EntityId,
        SemanticExpression, VariableId,
    };
    use std::collections::BTreeMap;

    fn catalog() -> ConceptCatalog {
        let city = ConceptId::new_unchecked("CITY");
        let predicate = ConceptId::new_unchecked("PREDICATE");
        let mut catalog = ConceptCatalog::default();
        catalog.concepts.insert(
            city.clone(),
            ConceptSchema {
                id: city.clone(),
                kind: ConceptKind::EntityType,
                parameters: BTreeMap::new(),
                result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                    city.clone(),
                ))),
            },
        );
        catalog.concepts.insert(
            predicate.clone(),
            ConceptSchema {
                id: predicate.clone(),
                kind: ConceptKind::Predicate,
                parameters: BTreeMap::new(),
                result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                    city.clone(),
                ))),
            },
        );
        catalog.entities.insert(
            EntityId::new_unchecked("PARIS"),
            EntityDefinition {
                id: EntityId::new_unchecked("PARIS"),
                primary_type: city,
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

        let catalog = catalog();
        let left_hash = canonical_goal_semantic_hash(&left, &catalog).expect("left goal is valid");
        let right_hash =
            canonical_goal_semantic_hash(&right, &catalog).expect("right goal is valid");
        assert_eq!(left_hash, right_hash);
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

        let catalog = catalog();
        let base_semantic =
            canonical_goal_semantic_hash(&base, &catalog).expect("base goal is valid");
        let changed_semantic =
            canonical_goal_semantic_hash(&changed, &catalog).expect("changed goal is valid");
        assert_eq!(base_semantic, changed_semantic);

        let base_request =
            canonical_goal_request_hash(&base, &catalog).expect("base request goal is valid");
        let changed_request =
            canonical_goal_request_hash(&changed, &catalog).expect("changed request goal is valid");
        assert_ne!(base_request, changed_request);
    }

    #[test]
    fn semantic_hash_is_alpha_equivalent_for_bound_variables() {
        let left = LinguaGoal {
            expression: SemanticExpression::And(vec![
                SemanticExpression::Equals {
                    left: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
                        "answer",
                    ))),
                    right: Box::new(SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))),
                },
                SemanticExpression::Exists {
                    variable: VariableId::new_unchecked("x"),
                    value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
                    body: Box::new(SemanticExpression::Equals {
                        left: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
                            "x",
                        ))),
                        right: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
                            "x",
                        ))),
                    }),
                },
            ]),
            variables: BTreeMap::from([(
                VariableId::new_unchecked("answer"),
                SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
            )]),
            projection: vec![VariableId::new_unchecked("answer")],
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: None,
        };
        let right = LinguaGoal {
            expression: SemanticExpression::And(vec![
                SemanticExpression::Equals {
                    left: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
                        "result",
                    ))),
                    right: Box::new(SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))),
                },
                SemanticExpression::Exists {
                    variable: VariableId::new_unchecked("y"),
                    value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
                    body: Box::new(SemanticExpression::Equals {
                        left: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
                            "y",
                        ))),
                        right: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
                            "y",
                        ))),
                    }),
                },
            ]),
            variables: BTreeMap::from([(
                VariableId::new_unchecked("result"),
                SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
            )]),
            projection: vec![VariableId::new_unchecked("result")],
            evidence_policy: EvidencePolicy::Ignore,
            world: None,
            limit: None,
        };

        let catalog = catalog();
        let left_hash = canonical_goal_semantic_hash(&left, &catalog).expect("left goal is valid");
        let right_hash =
            canonical_goal_semantic_hash(&right, &catalog).expect("right goal is valid");
        assert_eq!(left_hash, right_hash);
    }
}
