mod support;

use lexflex_lingua::{
    canonical_goal_request_hash, canonical_goal_semantic_hash, EvidencePolicy, LinguaGoal,
    SemanticType,
};
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
                bindings: BTreeMap::from([(
                    lexflex_model::ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        },
        variables: BTreeMap::from([(
            answer.clone(),
            SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        )]),
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

#[test]
fn semantic_hash_is_alpha_equivalent_for_free_variables() {
    let catalog = kernel_catalog();
    let left = goal(None, EvidencePolicy::Ignore);
    let mut right = goal(None, EvidencePolicy::Ignore);
    let answer = VariableId::new_unchecked("answer");
    let renamed = VariableId::new_unchecked("renamed_answer");
    right.expression = match right.expression {
        SemanticExpression::Satisfies { predicate, .. } => SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(renamed.clone())),
            predicate,
        },
        other => other,
    };
    right.variables.remove(&answer);
    right.variables.insert(
        renamed.clone(),
        SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
    );
    right.projection = vec![renamed];

    let left_hash = canonical_goal_semantic_hash(&left, &catalog).expect("left goal is valid");
    let right_hash = canonical_goal_semantic_hash(&right, &catalog).expect("right goal is valid");
    assert_eq!(left_hash, right_hash);
}

#[test]
fn semantic_hash_is_alpha_equivalent_for_bound_variables() {
    let catalog = kernel_catalog();
    let answer = VariableId::new_unchecked("answer");
    let city = ConceptId::new_unchecked("CITY");

    let left = LinguaGoal {
        expression: SemanticExpression::And(vec![
            SemanticExpression::Equals {
                left: Box::new(SemanticExpression::Variable(answer.clone())),
                right: Box::new(SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))),
            },
            SemanticExpression::Exists {
                variable: VariableId::new_unchecked("x"),
                value_type: SemanticType::EntityOf(city.clone()),
                body: Box::new(SemanticExpression::Equals {
                    left: Box::new(SemanticExpression::Variable(VariableId::new_unchecked("x"))),
                    right: Box::new(SemanticExpression::Variable(VariableId::new_unchecked("x"))),
                }),
            },
        ]),
        variables: BTreeMap::from([(answer.clone(), SemanticType::EntityOf(city.clone()))]),
        projection: vec![answer.clone()],
        evidence_policy: EvidencePolicy::Ignore,
        world: None,
        limit: None,
    };

    let right = LinguaGoal {
        expression: SemanticExpression::And(vec![
            SemanticExpression::Equals {
                left: Box::new(SemanticExpression::Variable(answer.clone())),
                right: Box::new(SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))),
            },
            SemanticExpression::Exists {
                variable: VariableId::new_unchecked("y"),
                value_type: SemanticType::EntityOf(city.clone()),
                body: Box::new(SemanticExpression::Equals {
                    left: Box::new(SemanticExpression::Variable(VariableId::new_unchecked("y"))),
                    right: Box::new(SemanticExpression::Variable(VariableId::new_unchecked("y"))),
                }),
            },
        ]),
        variables: BTreeMap::from([(answer.clone(), SemanticType::EntityOf(city))]),
        projection: vec![answer],
        evidence_policy: EvidencePolicy::Ignore,
        world: None,
        limit: None,
    };

    let left_hash = canonical_goal_semantic_hash(&left, &catalog).expect("left goal is valid");
    let right_hash = canonical_goal_semantic_hash(&right, &catalog).expect("right goal is valid");
    assert_eq!(left_hash, right_hash);
}
