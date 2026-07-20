use lexflex_language::LanguagePackageLoader;
use lexflex_lingua::{CompileContext, LinguaCompiler, LinguaInterpreter, LinguaProgram, ProgramId};
use lexflex_model::{ConceptCatalog, EntityDefinition, EntityId, LanguageId, SemanticExpression};
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

fn parser(code: &str) -> LexicalCompositionParser {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/languages")
        .join(code);
    let catalog = catalog();
    let language = Arc::new(
        LanguagePackageLoader
            .load(&root, &catalog)
            .expect("load language"),
    );
    LexicalCompositionParser::with_budget(Arc::new(catalog), language, ParseBudget::default())
}

fn lower_assertion(
    draft: lexflex_parser::AssertionDraft,
    catalog: Arc<ConceptCatalog>,
) -> SemanticExpression {
    let compiler = LinguaCompiler::try_new(catalog).expect("compiler");
    let program = LinguaProgram {
        id: ProgramId::new_unchecked("test:capital:assertion"),
        declarations: Vec::new(),
        entry: draft.expression,
    };
    let compiled = compiler
        .compile_with_context(&program, &CompileContext::default())
        .expect("compile");
    LinguaInterpreter::default()
        .execute(&compiled)
        .expect("execute")
        .execution
        .value
}

#[test]
fn english_and_polish_capital_have_same_canonical_hash() {
    let shared_catalog = Arc::new(catalog());
    let en = parser("en")
        .parse(ParseInput {
            source_id: "source:capital:en".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Paris is the capital of France.".into(),
        })
        .expect("parse en");
    let pl = parser("pl")
        .parse(ParseInput {
            source_id: "source:capital:pl".into(),
            language: LanguageId::new("pl").expect("language"),
            text: "Paryż jest stolicą Francji.".into(),
        })
        .expect("parse pl");
    let ParseOutput::Assertion(en) = en else {
        panic!("expected assertion");
    };
    let ParseOutput::Assertion(pl) = pl else {
        panic!("expected assertion");
    };
    assert_eq!(
        lower_assertion(en, Arc::clone(&shared_catalog)),
        lower_assertion(pl, shared_catalog)
    );
}
