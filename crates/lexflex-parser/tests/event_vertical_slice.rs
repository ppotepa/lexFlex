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

fn language(code: &str) -> Arc<lexflex_language::LanguageModel> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/languages")
        .join(code);
    Arc::new(
        LanguagePackageLoader
            .load(&root, &catalog())
            .expect("load language"),
    )
}

fn parser(code: &str) -> LexicalCompositionParser {
    LexicalCompositionParser::with_budget(language(code), ParseBudget::default())
}

#[test]
fn event_assertions_are_equal_across_languages() {
    let en = parser("en")
        .parse(ParseInput {
            source_id: "test:en:event".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Tom sees Iza.".into(),
        })
        .expect("parse en");

    let pl = parser("pl")
        .parse(ParseInput {
            source_id: "test:pl:event".into(),
            language: LanguageId::new("pl").expect("language"),
            text: "Tomek widzi Izę.".into(),
        })
        .expect("parse pl");

    let ParseOutput::Assertion(en) = en else {
        panic!("expected assertion");
    };
    let ParseOutput::Assertion(pl) = pl else {
        panic!("expected assertion");
    };

    assert_eq!(en.expression, pl.expression);
}

#[test]
fn event_questions_are_alpha_equivalent() {
    let en = parser("en")
        .parse(ParseInput {
            source_id: "test:en:question".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Who sees Iza?".into(),
        })
        .expect("parse en");

    let pl = parser("pl")
        .parse(ParseInput {
            source_id: "test:pl:question".into(),
            language: LanguageId::new("pl").expect("language"),
            text: "Kto widzi Izę?".into(),
        })
        .expect("parse pl");

    let ParseOutput::Goal(en) = en else {
        panic!("expected goal");
    };
    let ParseOutput::Goal(pl) = pl else {
        panic!("expected goal");
    };

    assert_eq!(en.expression, pl.expression);
    assert_eq!(en.variables, pl.variables);
    assert_eq!(en.projection, pl.projection);
}
