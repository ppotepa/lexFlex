use lexflex_lingua::normalize_expression;
use lexflex_model::{ConceptId, EntityId, SemanticExpression};
use std::collections::BTreeMap;

#[test]
fn nested_and_is_flattened_and_deduplicated() {
    let expression = SemanticExpression::And(vec![
        SemanticExpression::Entity(EntityId::new_unchecked("B")),
        SemanticExpression::And(vec![
            SemanticExpression::Entity(EntityId::new_unchecked("A")),
            SemanticExpression::Entity(EntityId::new_unchecked("B")),
        ]),
    ]);

    let normalized = normalize_expression(expression)
        .expect("normalize")
        .expression;
    assert_eq!(
        normalized,
        SemanticExpression::And(vec![
            SemanticExpression::Entity(EntityId::new_unchecked("A")),
            SemanticExpression::Entity(EntityId::new_unchecked("B")),
        ])
    );
}

#[test]
fn double_not_collapses() {
    let expression = SemanticExpression::Not(Box::new(SemanticExpression::Not(Box::new(
        SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
    ))));

    let normalized = normalize_expression(expression)
        .expect("normalize")
        .expression;
    assert_eq!(
        normalized,
        SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))
    );
}

#[test]
fn bindings_are_canonicalized_by_key_order() {
    let expression = SemanticExpression::Apply {
        concept: ConceptId::new_unchecked("CAPITAL"),
        bindings: BTreeMap::from([
            (
                lexflex_model::ParameterId::new_unchecked("z"),
                SemanticExpression::Entity(EntityId::new_unchecked("Z")),
            ),
            (
                lexflex_model::ParameterId::new_unchecked("a"),
                SemanticExpression::Entity(EntityId::new_unchecked("A")),
            ),
        ]),
    };

    let normalized = normalize_expression(expression)
        .expect("normalize")
        .expression;
    assert!(
        matches!(normalized, SemanticExpression::Apply { bindings, .. } if bindings.keys().next().map(|p| p.as_str()) == Some("a"))
    );
}
