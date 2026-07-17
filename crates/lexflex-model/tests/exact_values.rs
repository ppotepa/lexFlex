use lexflex_model::{ConceptId, DateValue, DecimalValue, QuantityValue, SemanticValue};

#[test]
fn quantity_dimension_affects_equality() {
    let left = SemanticValue::Quantity(QuantityValue {
        amount: DecimalValue { canonical: "1".into() },
        unit: ConceptId::new_unchecked("KG"),
        dimension: ConceptId::new_unchecked("MASS"),
    });
    let right = SemanticValue::Quantity(QuantityValue {
        amount: DecimalValue { canonical: "1".into() },
        unit: ConceptId::new_unchecked("KG"),
        dimension: ConceptId::new_unchecked("COUNT"),
    });
    assert_ne!(left, right);
}

#[test]
fn date_is_distinct_from_text() {
    assert_ne!(
        SemanticValue::Date(DateValue { iso8601: "2026-07-15".into() }),
        SemanticValue::Text("2026-07-15".into())
    );
}
