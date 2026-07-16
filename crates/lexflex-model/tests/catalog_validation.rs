use lexflex_model::{ConceptCatalog, ConceptId};

#[test]
fn country_is_subtype_of_polity() {
    let catalog = ConceptCatalog {
        concepts: Default::default(),
        entities: Default::default(),
        parents: std::collections::BTreeMap::from([(
            ConceptId::new_unchecked("COUNTRY"),
            std::collections::BTreeSet::from([ConceptId::new_unchecked("POLITY")]),
        )]),
    };
    assert!(catalog.is_subtype(
        &ConceptId::new_unchecked("COUNTRY"),
        &ConceptId::new_unchecked("POLITY")
    ));
}
