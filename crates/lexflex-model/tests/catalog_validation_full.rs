use lexflex_model::{validate_catalog, CatalogValidationIssue, ConceptCatalog, ConceptId,
                    ConceptKind, ConceptParameterSchema, ConceptSchema, EntityDefinition,
                    EntityId, ParameterId, SemanticType};
use std::collections::{BTreeMap, BTreeSet};

fn concept(id: &str, kind: ConceptKind, result_type: SemanticType) -> (ConceptId, ConceptSchema) {
    let concept_id = ConceptId::new_unchecked(id);
    (
        concept_id.clone(),
        ConceptSchema {
            id: concept_id,
            kind,
            parameters: BTreeMap::new(),
            result_type,
        },
    )
}

#[test]
fn valid_catalog_is_clean() {
    let catalog = ConceptCatalog {
        concepts: BTreeMap::from(
            [
                concept(
                    "POLITY",
                    ConceptKind::EntityType,
                    SemanticType::Predicate(Box::new(SemanticType::Entity)),
                ),
                concept(
                    "COUNTRY",
                    ConceptKind::EntityType,
                    SemanticType::Predicate(Box::new(
                        SemanticType::EntityOf(ConceptId::new_unchecked("COUNTRY")),
                    )),
                ),
                concept("SEE_EVENT", ConceptKind::EventType, SemanticType::Boolean),
                concept("LENGTH", ConceptKind::ValueFunction, SemanticType::Concept),
            ],
        ),
        entities: BTreeMap::from(
            [
                (
                    EntityId::new_unchecked("POLAND"),
                    EntityDefinition {
                        id: EntityId::new_unchecked("POLAND"),
                        primary_type: ConceptId::new_unchecked("COUNTRY"),
                        additional_types: BTreeSet::new(),
                    },
                ),
            ],
        ),
        parents: BTreeMap::from(
            [
                (
                    ConceptId::new_unchecked("COUNTRY"),
                    BTreeSet::from([ConceptId::new_unchecked("POLITY")]),
                ),
            ],
        ),
    };

    let mut catalog = catalog;
    catalog
        .concepts
        .get_mut(&ConceptId::new_unchecked("SEE_EVENT"))
        .expect("event")
        .parameters
        .insert(
            ParameterId::new_unchecked("agent"),
            ConceptParameterSchema {
                id: ParameterId::new_unchecked("agent"),
                value_type: SemanticType::Entity,
                required: true,
            },
        );

    let report = validate_catalog(&catalog);
    assert!(report.is_clean(), "{:?}", report.issues);
}

#[test]
fn event_type_requires_parameter_and_boolean_result() {
    let catalog = ConceptCatalog {
        concepts: BTreeMap::from(
            [
                concept("BROKEN_EVENT", ConceptKind::EventType, SemanticType::Entity),
            ],
        ),
        entities: BTreeMap::new(),
        parents: BTreeMap::new(),
    };

    let report = validate_catalog(&catalog);
    assert!(report.issues.iter().any(|issue| {
        matches!(
        issue,
        CatalogValidationIssue::EventTypeRequiresParameter { concept_id }
        if concept_id == "BROKEN_EVENT"
    )
    }));
}

#[test]
fn nested_optional_is_rejected() {
    let catalog = ConceptCatalog {
        concepts: BTreeMap::from(
            [
                concept(
                    "BROKEN_OPTIONAL",
                    ConceptKind::Function,
                    SemanticType::Optional(Box::new(
                        SemanticType::Optional(Box::new(SemanticType::Entity)),
                    )),
                ),
            ],
        ),
        entities: BTreeMap::new(),
        parents: BTreeMap::new(),
    };

    let report = validate_catalog(&catalog);
    assert!(report.issues.iter().any(
        |issue| matches!(issue, CatalogValidationIssue::InvalidOptionalNesting { .. }),
    ));
}

#[test]
fn issue_order_is_deterministic() {
    let catalog = ConceptCatalog {
        concepts: BTreeMap::from(
            [
                (
                    ConceptId::new_unchecked("A"),
                    ConceptSchema {
                        id: ConceptId::new_unchecked("WRONG_A"),
                        kind: ConceptKind::EntityType,
                        parameters: BTreeMap::new(),
                        result_type: SemanticType::Boolean,
                    },
                ),
                (
                    ConceptId::new_unchecked("B"),
                    ConceptSchema {
                        id: ConceptId::new_unchecked("WRONG_B"),
                        kind: ConceptKind::EntityType,
                        parameters: BTreeMap::new(),
                        result_type: SemanticType::Boolean,
                    },
                ),
            ],
        ),
        entities: BTreeMap::new(),
        parents: BTreeMap::new(),
    };

    let report = validate_catalog(&catalog);
    assert!(matches!(
        report.issues.first(),
        Some(CatalogValidationIssue::ConceptKeyMismatch { concept_id, .. }) if concept_id == "A"
    ));
}
