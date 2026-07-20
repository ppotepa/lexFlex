use lexflex_model::{
    validate_semantic_type_references, ConceptCatalog, ConceptId, ConceptKind, ConceptSchema,
    FunctionType, SemanticType, ValueType,
};
use std::collections::BTreeMap;

fn catalog() -> ConceptCatalog {
    let city = ConceptId::new_unchecked("CITY");
    ConceptCatalog {
        concepts: BTreeMap::from([(
            city.clone(),
            ConceptSchema {
                id: city.clone(),
                kind: ConceptKind::EntityType,
                parameters: BTreeMap::new(),
                result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(city))),
            },
        )]),
        entities: BTreeMap::new(),
        parents: BTreeMap::new(),
    }
}

#[test]
fn validates_nested_function_references() {
    let city = ConceptId::new_unchecked("CITY");
    let value_type = SemanticType::Function(FunctionType {
        parameters: BTreeMap::from([(
            lexflex_model::ParameterId::new_unchecked("input"),
            SemanticType::Optional(Box::new(SemanticType::Set(Box::new(
                SemanticType::EntityOf(city.clone()),
            )))),
        )]),
        result: Box::new(SemanticType::Predicate(Box::new(SemanticType::EntityOf(
            city,
        )))),
    });

    validate_semantic_type_references(&value_type, &catalog()).expect("valid references");
}

#[test]
fn rejects_missing_nested_concept() {
    let value_type = SemanticType::Record(BTreeMap::from([(
        "field".into(),
        SemanticType::Optional(Box::new(SemanticType::EntityOf(ConceptId::new_unchecked(
            "MISSING",
        )))),
    )]));

    assert!(validate_semantic_type_references(&value_type, &catalog()).is_err());
}

#[test]
fn rejects_missing_quantity_dimension() {
    let value_type = SemanticType::Value(ValueType::Quantity {
        dimension: ConceptId::new_unchecked("MISSING_DIMENSION"),
    });

    assert!(validate_semantic_type_references(&value_type, &catalog()).is_err());
}
