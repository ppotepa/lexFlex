use lexflex_model::{QualifierId, SemanticExpression, SemanticValue};
use std::collections::BTreeMap;

#[test]
fn qualifier_order_is_deterministic() {
    let expr = SemanticExpression::Qualified {
        expression: Box::new(SemanticExpression::Value(SemanticValue::Boolean(true))),
        qualifiers: BTreeMap::from([
            (
                QualifierId::new_unchecked("a"),
                SemanticExpression::Value(SemanticValue::Boolean(true)),
            ),
            (
                QualifierId::new_unchecked("b"),
                SemanticExpression::Value(SemanticValue::Boolean(false)),
            ),
        ]),
    };
    assert_eq!(expr.canonical_hash(), expr.canonical_hash());
}
