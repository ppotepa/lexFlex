use lexflex_model::{
    ConceptCatalog, Evidence, EvidenceSet, SemanticAssertion, SemanticExpression, SemanticValue,
    WorldId,
};

fn boolean_claim() -> SemanticExpression {
    SemanticExpression::Equals {
        left: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
        right: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
    }
}

#[test]
fn evidence_does_not_affect_assertion_id() {
    let catalog = ConceptCatalog::default();
    let left = SemanticAssertion::create(
        boolean_claim(),
        EvidenceSet::singleton(Evidence::create("source:a", None, None).expect("evidence"))
            .expect("valid evidence set"),
        WorldId::new_unchecked("actual"),
        &catalog,
    )
    .expect("assertion");
    let right = SemanticAssertion::create(
        boolean_claim(),
        EvidenceSet::singleton(Evidence::create("source:b", None, None).expect("evidence"))
            .expect("valid evidence set"),
        WorldId::new_unchecked("actual"),
        &catalog,
    )
    .expect("assertion");
    assert_eq!(left.id(), right.id());
    assert_eq!(left.verify(), Ok(()));
    assert_eq!(right.verify(), Ok(()));
}

#[test]
fn merge_returns_exact_added_count() {
    let catalog = ConceptCatalog::default();
    let mut assertion = SemanticAssertion::create(
        boolean_claim(),
        EvidenceSet::singleton(Evidence::create("source:a", None, None).expect("evidence"))
            .expect("valid evidence set"),
        WorldId::new_unchecked("actual"),
        &catalog,
    )
    .expect("assertion");
    let added = assertion
        .merge_evidence(
            EvidenceSet::try_from_iter([
                Evidence::create("source:a", None, None).expect("evidence"),
                Evidence::create("source:b", None, None).expect("evidence"),
            ])
            .expect("valid evidence set"),
        )
        .expect("merge");
    assert_eq!(added, 1);
}
