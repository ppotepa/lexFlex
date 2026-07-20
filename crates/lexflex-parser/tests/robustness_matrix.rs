use lexflex_language::LanguagePackageLoader;
use lexflex_model::{ConceptCatalog, EntityDefinition, EntityId, LanguageId};
use lexflex_parser::{LexicalCompositionParser, ParseBudget, ParseInput};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(serde::Deserialize)]
struct EntityPackage {
    entities: BTreeMap<EntityId, EntityDefinition>,
}

fn parser_with_budget(budget: ParseBudget) -> LexicalCompositionParser {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let mut catalog: ConceptCatalog =
        ron::from_str(&std::fs::read_to_string(root.join("model/concepts.ron")).expect("catalog"))
            .expect("catalog parse");
    catalog.entities = ron::from_str::<EntityPackage>(
        &std::fs::read_to_string(root.join("model/entities.ron")).expect("entities"),
    )
    .expect("entities parse")
    .entities;
    let language = LanguagePackageLoader
        .load(&root.join("languages/en"), &catalog)
        .expect("language");
    LexicalCompositionParser::with_budget(Arc::new(catalog), Arc::new(language), budget)
}

#[test]
fn adversarial_texts_return_typed_parse_results() {
    let parser = parser_with_budget(ParseBudget::default());
    for (index, text) in [
        "",
        " ",
        "?!",
        "Paris,,, France",
        "\n\t\r",
        &"unknown ".repeat(64),
    ]
    .into_iter()
    .enumerate()
    {
        let result = parser.parse(ParseInput {
            source_id: format!("test:robustness:{index}"),
            language: LanguageId::new_unchecked("en"),
            text: text.to_owned(),
        });
        let _ = result;
    }
}

#[test]
fn token_budget_rejects_before_chart_expansion() {
    let parser = parser_with_budget(ParseBudget {
        max_tokens: 3,
        ..ParseBudget::default()
    });
    let result = parser.parse(ParseInput {
        source_id: "test:robustness:budget".into(),
        language: LanguageId::new_unchecked("en"),
        text: "Paris is the capital of France.".into(),
    });
    assert!(matches!(
        result,
        Err(lexflex_parser::ParseError::BudgetExceeded(
            lexflex_parser::ParseBudgetLimit::TokenLimit
        ))
    ));
}
