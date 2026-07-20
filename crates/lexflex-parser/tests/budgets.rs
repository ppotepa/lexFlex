use lexflex_language::LanguagePackageLoader;
use lexflex_model::{ConceptCatalog, EntityDefinition, EntityId, LanguageId};
use lexflex_parser::{
    LexicalCompositionParser, ParseBudget, ParseBudgetLimit, ParseError, ParseInput,
};
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

fn capital_input() -> ParseInput {
    ParseInput {
        source_id: "test:budget:capital".into(),
        language: LanguageId::new("en").expect("language"),
        text: "Paris is the capital of France.".into(),
    }
}

#[test]
fn token_limit_is_enforced() {
    let parser = parser(
        "en",
        ParseBudget {
            max_tokens: 1,
            ..ParseBudget::default()
        },
    );

    assert!(matches!(
        parser.parse(capital_input()),
        Err(ParseError::BudgetExceeded(ParseBudgetLimit::TokenLimit))
    ));
}

#[test]
fn lexical_candidate_limit_is_enforced() {
    let parser = parser(
        "en",
        ParseBudget {
            max_lexical_candidates_per_token: 0,
            ..ParseBudget::default()
        },
    );

    assert!(matches!(
        parser.parse(capital_input()),
        Err(ParseError::BudgetExceeded(
            ParseBudgetLimit::LexicalCandidateLimit
        ))
    ));
}

#[test]
fn cell_item_limit_is_enforced() {
    let parser = parser(
        "en",
        ParseBudget {
            max_items_per_cell: 0,
            ..ParseBudget::default()
        },
    );

    assert!(matches!(
        parser.parse(capital_input()),
        Err(ParseError::BudgetExceeded(ParseBudgetLimit::CellItemLimit))
    ));
}

#[test]
fn total_item_limit_is_enforced() {
    let parser = parser(
        "en",
        ParseBudget {
            max_total_items: 0,
            ..ParseBudget::default()
        },
    );

    assert!(matches!(
        parser.parse(capital_input()),
        Err(ParseError::BudgetExceeded(ParseBudgetLimit::TotalItemLimit))
    ));
}

#[test]
fn derivation_depth_limit_is_enforced() {
    let parser = parser(
        "en",
        ParseBudget {
            max_derivation_depth: 1,
            ..ParseBudget::default()
        },
    );

    assert!(matches!(
        parser.parse(capital_input()),
        Err(ParseError::BudgetExceeded(
            ParseBudgetLimit::DerivationDepthLimit
        ))
    ));
}

#[test]
fn complete_parse_limit_is_enforced() {
    let parser = parser(
        "en",
        ParseBudget {
            max_complete_parses: 0,
            ..ParseBudget::default()
        },
    );

    assert!(matches!(
        parser.parse(capital_input()),
        Err(ParseError::BudgetExceeded(
            ParseBudgetLimit::CompleteParseLimit
        ))
    ));
}

#[test]
fn semantic_node_limit_is_enforced() {
    let parser = parser(
        "en",
        ParseBudget {
            max_semantic_nodes: 0,
            ..ParseBudget::default()
        },
    );

    assert!(matches!(
        parser.parse(capital_input()),
        Err(ParseError::BudgetExceeded(
            ParseBudgetLimit::SemanticNodeLimit
        ))
    ));
}
