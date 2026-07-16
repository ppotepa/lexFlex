use lexflex_language::LanguagePackageLoader;
use lexflex_model::ConceptCatalog;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct EntityPackage {
    entities: BTreeMap<lexflex_model::EntityId, lexflex_model::EntityDefinition>,
}

fn catalog() -> ConceptCatalog {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/model");
    let concepts_source =
        std::fs::read_to_string(root.join("concepts.ron")).expect("concept catalog");
    let entities_source =
        std::fs::read_to_string(root.join("entities.ron")).expect("entity catalog");
    let mut catalog: ConceptCatalog = ron::from_str(&concepts_source).expect("parse catalog");
    let entities: EntityPackage = ron::from_str(&entities_source).expect("parse entities");
    catalog.entities = entities.entities;
    catalog
}

#[test]
fn polish_package_loads() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/pl");
    let model = LanguagePackageLoader.load(&root, &catalog()).expect("load");
    assert_eq!(model.lexemes.len(), 11);
    assert_eq!(model.senses.len(), 11);
}
