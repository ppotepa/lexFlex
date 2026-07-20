mod support;

use lexflex_lingua::{EvidencePolicy, LinguaGoal, LinguaSolver, SolveError};
use lexflex_model::{
    ConceptId, EntityId, Evidence, EvidenceSet, SemanticAssertion, SemanticExpression,
    SemanticType, SourceSpan, VariableId, WorldId,
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
        EvidenceSet::singleton(evidence(&format!("source:{city}"))).expect("valid evidence set"),
        WorldId::new_unchecked("world:test"),
        &catalog,
    )
    .expect("assertion")
}

#[test]
fn normal_mismatch_is_skipped() {
    let solver = LinguaSolver::default();
    let catalog = Arc::new(kernel_catalog());
    let matching = capital_assertion("PARIS");
    let result = solver
        .solve(&goal(), [&matching], catalog.clone())
        .expect("solve");

    assert_eq!(result.len(), 1);

    let mismatch = SemanticAssertion::create(
        SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
        EvidenceSet::singleton(evidence("source:mismatch")).expect("valid evidence set"),
        WorldId::new_unchecked("world:test"),
        catalog.as_ref(),
    );
    assert!(matches!(
        mismatch,
        Err(lexflex_model::AssertionCatalogError::NonBoolean(_))
    ));
}

#[test]
fn invalid_candidate_variable_is_fatal_integrity_error() {
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
        EvidenceSet::singleton(evidence("source:bad")).expect("valid evidence set"),
        WorldId::new_unchecked("world:test"),
        &kernel_catalog(),
    );

    assert!(matches!(
        assertion,
        Err(lexflex_model::AssertionCatalogError::Type(_))
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
fn candidate_type_corruption_is_fatal() {
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
        EvidenceSet::singleton(evidence("source:type-mismatch")).expect("valid evidence set"),
        WorldId::new_unchecked("world:test"),
        &kernel_catalog(),
    );
    assert!(matches!(
        mismatch,
        Err(lexflex_model::AssertionCatalogError::Type(_))
    ));
}
