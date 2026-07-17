use lexflex_language::LanguagePackageLoader;
use lexflex_model::{ConceptCatalog, EntityDefinition, EntityId, LanguageId};
use lexflex_parser::{LexicalCompositionParser, ParseBudget, ParseInput, ParseOutput};
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

fn parser(code: &str, budget: ParseBudget) -> LexicalCompositionParser {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/languages")
        .join(code);
    let catalog = catalog();
    let language = Arc::new(
        LanguagePackageLoader
            .load(&root, &catalog)
            .expect("load language"),
    );
    LexicalCompositionParser::with_budget(Arc::new(catalog), language, budget)
}

#[test]
fn capital_parse_metrics_are_deterministic() {
    let parser = parser("en", ParseBudget::default());
    let input = ParseInput {
        source_id: "test:metrics:capital".into(),
        language: LanguageId::new("en").expect("language"),
        text: "Paris is the capital of France.".into(),
    };

    let first = parser.parse(input.clone()).expect("first parse");
    let second = parser.parse(input).expect("second parse");

    let ParseOutput::Assertion(first) = first else {
        panic!("expected assertion");
    };
    let ParseOutput::Assertion(second) = second else {
        panic!("expected assertion");
    };

    assert_eq!(first.metrics, second.metrics);
    assert_eq!(first.metrics.token_count, 6);
    assert!(first.metrics.lexical_candidate_count > 0);
    assert!(first.metrics.chart_item_count > 0);
    assert!(first.metrics.max_cell_size > 0);
    assert!(first.metrics.max_derivation_depth > 0);
    assert!(first.metrics.max_semantic_nodes > 0);
    assert_eq!(first.metrics.complete_semantic_count, 1);
}
