use lexflex_language::LanguagePackageLoader;
use lexflex_model::{ConceptCatalog, EntityDefinition, EntityId, LanguageId, SemanticType};
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
fn question_retains_resolved_query_type() {
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
            source_id: "test:query-type".into(),
            language: LanguageId::new("en").expect("language"),
            text: "What is the capital of France?".into(),
        })
        .expect("parse");
    let ParseOutput::Goal(goal) = output else {
        panic!("expected goal");
    };
    let variable = goal.projection.first().expect("projection");
    assert_eq!(
        goal.variables.get(variable),
        Some(&SemanticType::EntityOf(lexflex_model::ConceptId::new_unchecked("CITY")))
    );
}
