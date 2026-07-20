mod support;

use lexflex_lingua::{EvidencePolicy, LinguaGoal, LinguaSolver};
use lexflex_model::{
    ConceptId, EntityId, Evidence, EvidenceSet, SemanticAssertion, SemanticExpression,
    SemanticType, SourceSpan, VariableId, WorldId,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use support::model::kernel_catalog;

fn goal(limit: Option<usize>) -> LinguaGoal {
    let city = VariableId::new_unchecked("city");
    LinguaGoal {
        expression: SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(city.clone())),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    lexflex_model::ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        },
        variables: BTreeMap::from([(
            city.clone(),
            SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        )]),
        projection: vec![city],
        evidence_policy: EvidencePolicy::Optional,
        world: Some(WorldId::new_unchecked("world:test")),
        limit,
    }
}

fn evidence(source_id: &str) -> Evidence {
    Evidence::create(
        source_id,
        Some(SourceSpan::new(0, source_id.len() as u64).expect("span")),
        None,
    )
    .expect("evidence")
}

fn capital_assertion(city: &str, source_id: &str) -> SemanticAssertion {
    let catalog = kernel_catalog();
    SemanticAssertion::create(
        SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Entity(EntityId::new_unchecked(city))),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    lexflex_model::ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        },
        EvidenceSet::singleton(evidence(source_id)).expect("valid evidence set"),
        WorldId::new_unchecked("world:test"),
        &catalog,
    )
    .expect("assertion")
}

#[test]
fn reversed_assertion_order_produces_same_result_order() {
    let solver = LinguaSolver::default();
    let paris = capital_assertion("PARIS", "source:paris");
    let warsaw = capital_assertion("WARSAW", "source:warsaw");

    let left = solver
        .solve(&goal(None), [&warsaw, &paris], Arc::new(kernel_catalog()))
        .expect("left");
    let right = solver
        .solve(&goal(None), [&paris, &warsaw], Arc::new(kernel_catalog()))
        .expect("right");

    assert_eq!(left, right);
    assert_eq!(left.len(), 2);
    assert_eq!(&left[0].assertion_id, paris.id());
    assert_eq!(&left[1].assertion_id, warsaw.id());
}

#[test]
fn limit_applies_after_sorting() {
    let solver = LinguaSolver::default();
    let paris = capital_assertion("PARIS", "source:paris");
    let warsaw = capital_assertion("WARSAW", "source:warsaw");

    let result = solver
        .solve(
            &goal(Some(1)),
            [&warsaw, &paris],
            Arc::new(kernel_catalog()),
        )
        .expect("solve");

    assert_eq!(result.len(), 1);
    assert_eq!(&result[0].assertion_id, paris.id());
}

#[test]
fn duplicate_solutions_are_removed() {
    let solver = LinguaSolver::default();
    let paris = capital_assertion("PARIS", "source:paris");

    let result = solver
        .solve(&goal(None), [&paris, &paris], Arc::new(kernel_catalog()))
        .expect("solve");

    assert_eq!(result.len(), 1);
}
