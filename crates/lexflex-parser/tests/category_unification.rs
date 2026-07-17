use lexflex_model::{ConceptCatalog, EntityDefinition, EntityId, SemanticType, TypeRelation};
use std::collections::BTreeMap;

#[derive(serde::Deserialize)]
struct EntityPackage {
    entities: BTreeMap<EntityId, EntityDefinition>,
}

#[test]
fn country_is_subtype_of_polity() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/model");
    let concepts = std::fs::read_to_string(root.join("concepts.ron")).expect("concepts");
    let entities = std::fs::read_to_string(root.join("entities.ron")).expect("entities");
    let mut catalog: ConceptCatalog = ron::from_str(&concepts).expect("catalog");
    let entities: EntityPackage = ron::from_str(&entities).expect("entities");
    catalog.entities = entities.entities;
    let relation = TypeRelation::new(&catalog);
    assert!(relation.accepts(
        &SemanticType::EntityOf(lexflex_model::ConceptId::new_unchecked("POLITY")),
        &SemanticType::EntityOf(lexflex_model::ConceptId::new_unchecked("COUNTRY"))
    ));
}
