mod support;

use lexflex_lingua::{solve::UnifyError, EvidencePolicy, LinguaGoal, LinguaSolver, SolveError};
use lexflex_model::{
    ConceptId, EntityId, Evidence, SemanticAssertion, SemanticExpression, SemanticType, SourceSpan,
    VariableId, WorldId,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use support::model::kernel_catalog;

fn goal() -> LinguaGoal {
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
        limit: None,
    }
}

fn evidence(source_id: &str) -> Evidence {
    Evidence::create(source_id, Some(SourceSpan::new(0, 4).expect("span")), None).expect("evidence")
}

fn capital_assertion(city: &str) -> SemanticAssertion {
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
        [evidence(&format!("source:{city}"))],
        WorldId::new_unchecked("world:test"),
    )
    .expect("assertion")
}

#[test]
fn normal_mismatch_is_skipped() {
    let solver = LinguaSolver::default();
    let catalog = Arc::new(kernel_catalog());
    let matching = capital_assertion("PARIS");
    let result = solver.solve(&goal(), [&matching], catalog).expect("solve");

    assert_eq!(result.len(), 1);

    let mismatch = SemanticAssertion::create(
        SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
        [evidence("source:mismatch")],
        WorldId::new_unchecked("world:test"),
    )
    .expect("mismatch assertion");

    let result = solver
        .solve(&goal(), [&mismatch], Arc::new(kernel_catalog()))
        .expect("solve mismatch");

    assert!(result.is_empty());
}

#[test]
fn fatal_unify_error_is_returned() {
    let solver = LinguaSolver::default();
    let assertion = SemanticAssertion::create(
        SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
                "candidate",
            ))),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    lexflex_model::ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        },
        [evidence("source:bad")],
        WorldId::new_unchecked("world:test"),
    )
    .expect("assertion");

    let result = solver.solve(&goal(), [&assertion], Arc::new(kernel_catalog()));

    assert!(matches!(
        result,
        Err(SolveError::Unify(UnifyError::UnknownVariableType(variable)))
            if variable == VariableId::new_unchecked("candidate")
    ));
}

#[test]
fn invalid_goal_is_returned_before_scan() {
    let solver = LinguaSolver::default();
    let mut invalid = goal();
    invalid.projection.clear();

    let result = solver.solve(
        &invalid,
        [&capital_assertion("PARIS")],
        Arc::new(kernel_catalog()),
    );

    assert!(matches!(result, Err(SolveError::Goal(_))));
}

#[test]
fn candidate_type_mismatch_is_not_fatal() {
    let solver = LinguaSolver::default();
    let mismatch = SemanticAssertion::create(
        SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Entity(EntityId::new_unchecked(
                "FRANCE",
            ))),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    lexflex_model::ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        },
        [evidence("source:type-mismatch")],
        WorldId::new_unchecked("world:test"),
    )
    .expect("assertion");

    let result = solver
        .solve(&goal(), [&mismatch], Arc::new(kernel_catalog()))
        .expect("solve mismatch");

    assert!(result.is_empty());
}
