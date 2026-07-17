mod support;

use lexflex_lingua::{canonical_goal_request_hash, canonical_goal_semantic_hash, EvidencePolicy,
                     LinguaGoal, SemanticType};
use lexflex_model::{ConceptId, EntityId, SemanticExpression, VariableId};
use std::collections::BTreeMap;
use support::model::kernel_catalog;

fn goal(limit: Option<usize>, evidence_policy: EvidencePolicy) -> LinguaGoal {
    let answer = VariableId::new_unchecked("answer");
    LinguaGoal {
        expression: SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(answer.clone())),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from(
                    [
                        (
                            lexflex_model::ParameterId::new_unchecked("scope"),
                            SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                        ),
                    ],
                ),
            }),
        },
        variables: BTreeMap::from(
            [
                (
                    answer.clone(),
                    SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
                ),
            ],
        ),
        projection: vec![answer],
        evidence_policy,
        world: None,
        limit,
    }
}

#[test]
fn semantic_hash_ignores_request_policy_but_request_hash_changes() {
    let catalog = kernel_catalog();
    let left = goal(None, EvidencePolicy::Ignore);
    let right = goal(Some(1), EvidencePolicy::Required);
    assert_eq!(
        canonical_goal_semantic_hash(&left, &catalog).expect("hash"),
        canonical_goal_semantic_hash(&right, &catalog).expect("hash")
    );
    assert_ne!(
        canonical_goal_request_hash(&left, &catalog).expect("hash"),
        canonical_goal_request_hash(&right, &catalog).expect("hash")
    );
}
