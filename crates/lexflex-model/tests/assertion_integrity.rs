use lexflex_model::{Evidence, SemanticAssertion, SemanticExpression, SemanticValue, WorldId};

#[test]
fn evidence_does_not_affect_assertion_id() {
    let left = SemanticAssertion::create(
        SemanticExpression::Value(SemanticValue::Boolean(true)),
        vec![Evidence::create("source:a", None, None).expect("evidence")],
        WorldId::new_unchecked("actual"),
    ).expect("assertion");
    let right = SemanticAssertion::create(
        SemanticExpression::Value(SemanticValue::Boolean(true)),
        vec![Evidence::create("source:b", None, None).expect("evidence")],
        WorldId::new_unchecked("actual"),
    ).expect("assertion");
    assert_eq!(left.id, right.id);
    assert_eq!(left.verify(), Ok(()));
    assert_eq!(right.verify(), Ok(()));
}

#[test]
fn merge_returns_exact_added_count() {
    let mut assertion = SemanticAssertion::create(
        SemanticExpression::Value(SemanticValue::Boolean(true)),
        vec![Evidence::create("source:a", None, None).expect("evidence")],
        WorldId::new_unchecked("actual"),
    ).expect("assertion");
    let added = assertion
        .merge_evidence(
            [
                Evidence::create("source:a", None, None).expect("evidence"),
                Evidence::create("source:b", None, None).expect("evidence"),
            ],
        )
        .expect("merge");
    assert_eq!(added, 1);
}
