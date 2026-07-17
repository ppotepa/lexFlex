use lexflex_model::{ConceptId, ParameterId, SemanticExpression};
use std::collections::BTreeMap;

#[test]
fn canonical_hash_is_stable() {
    let expression = SemanticExpression::Apply {
        concept: ConceptId::new_unchecked("CAPITAL"),
        bindings: BTreeMap::from(
            [
                (
                    ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(lexflex_model::EntityId::new_unchecked("FRANCE")),
                ),
            ],
        ),
    };
    assert_eq!(expression.canonical_hash(), expression.canonical_hash());
}
