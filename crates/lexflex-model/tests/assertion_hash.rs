use lexflex_model::{
    ConceptCatalog, Evidence, EvidenceSet, SemanticAssertion, SemanticExpression, WorldId,
};

fn boolean_claim() -> SemanticExpression {
    SemanticExpression::Equals {
        left: Box::new(SemanticExpression::Value(
            lexflex_model::SemanticValue::Boolean(true),
        )),
        right: Box::new(SemanticExpression::Value(
            lexflex_model::SemanticValue::Boolean(true),
        )),
    }
}

#[test]
fn evidence_order_is_canonical() {
    let a = Evidence::create("a", None, None).expect("valid evidence");
    let b = Evidence::create("b", None, None).expect("valid evidence");
    let catalog = ConceptCatalog::default();
    let left = SemanticAssertion::create(
        boolean_claim(),
        EvidenceSet::try_from_iter([b.clone(), a.clone()]).expect("valid evidence set"),
        WorldId::new_unchecked("actual"),
        &catalog,
    )
    .expect("valid assertion");
    let right = SemanticAssertion::create(
        boolean_claim(),
        EvidenceSet::try_from_iter([a, b]).expect("valid evidence set"),
        WorldId::new_unchecked("actual"),
        &catalog,
    )
    .expect("valid assertion");
    assert_eq!(left.canonical_hash(), right.canonical_hash());
}

#[test]
fn world_affects_hash() {
    let evidence = Evidence::create("a", None, None).expect("valid evidence");
    let catalog = ConceptCatalog::default();
    let left = SemanticAssertion::create(
        boolean_claim(),
        EvidenceSet::singleton(evidence.clone()).expect("valid evidence set"),
        WorldId::new_unchecked("actual"),
        &catalog,
    )
    .expect("valid assertion");
    let right = SemanticAssertion::create(
        boolean_claim(),
        EvidenceSet::singleton(evidence).expect("valid evidence set"),
        WorldId::new_unchecked("counterfactual"),
        &catalog,
    )
    .expect("valid assertion");
    assert_ne!(left.canonical_hash(), right.canonical_hash());
}
