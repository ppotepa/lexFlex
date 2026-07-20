use lexflex_model::{
    AssertionCatalogError, ConceptCatalog, EntityId, Evidence, EvidenceSet, SemanticAssertion,
    SemanticExpression, SemanticValue, WorldId,
};

fn evidence() -> EvidenceSet {
    EvidenceSet::singleton(Evidence::create("source:assertion", None, None).expect("evidence"))
        .expect("valid evidence set")
}

fn boolean_claim() -> SemanticExpression {
    SemanticExpression::Equals {
        left: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
        right: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
    }
}

#[test]
fn verified_boolean_assertion_is_accepted() {
    let assertion = SemanticAssertion::create(
        boolean_claim(),
        evidence(),
        WorldId::new_unchecked("actual"),
        &ConceptCatalog::default(),
    )
    .expect("valid Boolean assertion");

    assert_eq!(assertion.verify(), Ok(()));
}

#[test]
fn bare_entity_assertion_is_rejected() {
    let result = SemanticAssertion::create(
        SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
        evidence(),
        WorldId::new_unchecked("actual"),
        &ConceptCatalog::default(),
    );

    assert!(matches!(result, Err(AssertionCatalogError::Type(_))));
}
