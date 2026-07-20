use lexflex_language::LanguagePackageLoader;
use lexflex_model::ConceptCatalog;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

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
    let required_lexemes = [
        "lexeme:pl:paryz:proper",
        "lexeme:pl:francja:proper",
        "lexeme:pl:stolica:noun",
        "lexeme:pl:tomek:proper",
        "lexeme:pl:iza:proper",
        "lexeme:pl:widziec:verb",
        "lexeme:pl:jaki:question",
        "lexeme:pl:kto:question",
        "lexeme:pl:kogo:question",
    ];
    let loaded_lexemes: BTreeSet<_> = model.lexemes.keys().map(|id| id.as_str()).collect();
    for lexeme in required_lexemes {
        assert!(
            loaded_lexemes.contains(lexeme),
            "missing required lexeme: {lexeme}"
        );
    }

    let required_senses = [
        "sense:pl:paryz:entity",
        "sense:pl:francja:entity",
        "sense:pl:stolica:city",
        "sense:pl:widzi:event",
        "sense:pl:kto:question",
        "sense:pl:kogo:question",
    ];
    let loaded_senses: BTreeSet<_> = model.senses.keys().map(|id| id.as_str()).collect();
    for sense in required_senses {
        assert!(
            loaded_senses.contains(sense),
            "missing required sense: {sense}"
        );
    }
    assert!(model.lexemes.len() >= required_lexemes.len());
    assert!(model.senses.len() >= required_senses.len());
}
