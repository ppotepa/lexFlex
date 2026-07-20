use lexflex_language::LanguagePackageLoader;
use lexflex_model::{ConceptCatalog, EntityDefinition, EntityId, LanguageId};
use lexflex_parser::{LexicalCompositionParser, ParseInput, ParseOutput};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(serde::Deserialize)]
struct EntityPackage {
    entities: BTreeMap<EntityId, EntityDefinition>,
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
fn capital_statement_is_not_semantically_ambiguous() {
    let catalog = catalog();
    let language = Arc::new(
        LanguagePackageLoader
            .load(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
                &catalog,
            )
            .expect("load language"),
    );
    let parser = LexicalCompositionParser::new(Arc::new(catalog), language);
    let output = parser
        .parse(ParseInput {
            source_id: "test:semantic-ambiguity".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Paris is the capital of France.".into(),
        })
        .expect("parse");
    assert!(matches!(output, ParseOutput::Assertion(_)));
}
