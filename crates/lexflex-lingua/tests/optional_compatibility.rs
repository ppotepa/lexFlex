use lexflex_lingua::solve::semantic_types_compatible;
use lexflex_lingua::SemanticType;
use lexflex_model::ConceptCatalog;

#[test]
fn optional_expected_accepts_plain_actual() {
    let catalog = ConceptCatalog::default();
    let actual = SemanticType::Value(lexflex_lingua::ValueType::Date);
    let expected = SemanticType::Optional(Box::new(SemanticType::Value(
        lexflex_lingua::ValueType::Date,
    )));

    assert!(semantic_types_compatible(&actual, &expected, &catalog));
}

#[test]
fn optional_actual_does_not_match_plain_expected() {
    let catalog = ConceptCatalog::default();
    let actual = SemanticType::Optional(Box::new(SemanticType::Value(
        lexflex_lingua::ValueType::Date,
    )));
    let expected = SemanticType::Value(lexflex_lingua::ValueType::Date);

    assert!(!semantic_types_compatible(&actual, &expected, &catalog));
}
