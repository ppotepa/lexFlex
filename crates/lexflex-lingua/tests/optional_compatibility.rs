use lexflex_lingua::solve::TypeRelation;
use lexflex_lingua::ValueType;
use lexflex_model::{ConceptCatalog, SemanticType};

#[test]
fn optional_expected_matches_plain_actual() {
    let catalog = ConceptCatalog::default();
    let actual = SemanticType::Value(ValueType::Date);
    let expected = SemanticType::Optional(Box::new(SemanticType::Value(ValueType::Date)));

    let relation = TypeRelation::new(&catalog);
    assert!(relation.accepts(&expected, &actual));
}

#[test]
fn optional_actual_does_not_match_plain_expected() {
    let catalog = ConceptCatalog::default();
    let actual = SemanticType::Optional(Box::new(SemanticType::Value(ValueType::Date)));
    let expected = SemanticType::Value(ValueType::Date);

    let relation = TypeRelation::new(&catalog);
    assert!(!relation.accepts(&expected, &actual));
}
