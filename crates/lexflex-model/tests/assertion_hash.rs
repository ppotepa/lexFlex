use lexflex_model::{Evidence, SemanticAssertion, SemanticExpression, WorldId};

#[test]
fn evidence_order_is_canonical() {
    let a = Evidence::create("a", None, None).expect("valid evidence");
    let b = Evidence::create("b", None, None).expect("valid evidence");
    let left = SemanticAssertion::create(
        SemanticExpression::Value(lexflex_model::SemanticValue::Boolean(true)),
        vec![b.clone(), a.clone()],
        WorldId::new_unchecked("actual"),
    ).expect("valid assertion");
    let right = SemanticAssertion::create(
        SemanticExpression::Value(lexflex_model::SemanticValue::Boolean(true)),
        vec![a, b],
        WorldId::new_unchecked("actual"),
    ).expect("valid assertion");
    assert_eq!(left.canonical_hash, right.canonical_hash);
}

#[test]
fn world_affects_hash() {
    let evidence = Evidence::create("a", None, None).expect("valid evidence");
    let left = SemanticAssertion::create(
        SemanticExpression::Value(lexflex_model::SemanticValue::Boolean(true)),
        vec![evidence.clone()],
        WorldId::new_unchecked("actual"),
    ).expect("valid assertion");
    let right = SemanticAssertion::create(
        SemanticExpression::Value(lexflex_model::SemanticValue::Boolean(true)),
        vec![evidence],
        WorldId::new_unchecked("counterfactual"),
    ).expect("valid assertion");
    assert_ne!(left.canonical_hash, right.canonical_hash);
}
