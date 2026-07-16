use lexflex_model::{Evidence, EvidenceId, SemanticAssertion, SemanticExpression, WorldId};

#[test]
fn evidence_order_is_canonical() {
    let a = Evidence {
        id: EvidenceId::new_unchecked("e1"),
        source_id: "a".into(),
        span: None,
        source_hash: None,
    };
    let b = Evidence {
        id: EvidenceId::new_unchecked("e2"),
        source_id: "b".into(),
        span: None,
        source_hash: None,
    };
    let left = SemanticAssertion::create(
        SemanticExpression::Value(lexflex_model::SemanticValue::Boolean(true)),
        vec![b.clone(), a.clone()],
        WorldId::new_unchecked("actual"),
    );
    let right = SemanticAssertion::create(
        SemanticExpression::Value(lexflex_model::SemanticValue::Boolean(true)),
        vec![a, b],
        WorldId::new_unchecked("actual"),
    );
    assert_eq!(left.canonical_hash, right.canonical_hash);
}

#[test]
fn world_affects_hash() {
    let evidence = Evidence {
        id: EvidenceId::new_unchecked("e1"),
        source_id: "a".into(),
        span: None,
        source_hash: None,
    };
    let left = SemanticAssertion::create(
        SemanticExpression::Value(lexflex_model::SemanticValue::Boolean(true)),
        vec![evidence.clone()],
        WorldId::new_unchecked("actual"),
    );
    let right = SemanticAssertion::create(
        SemanticExpression::Value(lexflex_model::SemanticValue::Boolean(true)),
        vec![evidence],
        WorldId::new_unchecked("counterfactual"),
    );
    assert_ne!(left.canonical_hash, right.canonical_hash);
}
