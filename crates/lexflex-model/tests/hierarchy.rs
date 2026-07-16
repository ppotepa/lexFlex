use lexflex_model::{ConceptCatalog, ConceptId, EntityDefinition, EntityId};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn paris_is_city() {
    let catalog = ConceptCatalog {
        concepts: BTreeMap::new(),
        entities: BTreeMap::from([(
            EntityId::new_unchecked("PARIS"),
            EntityDefinition {
                id: EntityId::new_unchecked("PARIS"),
                primary_type: ConceptId::new_unchecked("CITY"),
                additional_types: BTreeSet::new(),
            },
        )]),
        parents: BTreeMap::new(),
    };
    assert!(catalog.entity_is(
        &EntityId::new_unchecked("PARIS"),
        &ConceptId::new_unchecked("CITY")
    ));
}
