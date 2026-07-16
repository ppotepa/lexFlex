use lexflex_language::LanguagePackageLoader;
use lexflex_lingua::{
    canonical_goal_hash, CompileContext, EvidencePolicy, LinguaCompiler, LinguaGoal,
    LinguaInterpreter, LinguaProgram, ProgramId,
};
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
    LexicalCompositionParser::with_budget(
        Arc::new(catalog()),
        language(code),
        ParseBudget::default(),
    )
}

fn lower_assertion(
    draft: lexflex_parser::AssertionDraft,
    catalog: Arc<ConceptCatalog>,
) -> SemanticExpression {
    let compiler = LinguaCompiler::new(catalog);
    let program = LinguaProgram {
        id: ProgramId::new_unchecked("test:event:assertion"),
        declarations: Vec::new(),
        entry: draft.expression,
    };
    let compiled = compiler
        .compile_with_context(&program, &CompileContext::default())
        .expect("compile assertion");
    LinguaInterpreter::default()
        .execute(&compiled)
        .expect("execute assertion")
        .value
}

fn lower_goal(draft: lexflex_parser::GoalDraft, catalog: Arc<ConceptCatalog>) -> LinguaGoal {
    let compiler = LinguaCompiler::new(catalog);
    let program = LinguaProgram {
        id: ProgramId::new_unchecked("test:event:goal"),
        declarations: Vec::new(),
        entry: draft.expression,
    };
    let compiled = compiler
        .compile_with_context(
            &program,
            &CompileContext {
                query_variables: draft.variables.clone(),
            },
        )
        .expect("compile goal");
    let expression = LinguaInterpreter::default()
        .execute(&compiled)
        .expect("execute goal")
        .value;
    LinguaGoal {
        expression,
        variables: draft.variables,
        projection: draft.projection,
        evidence_policy: EvidencePolicy::Ignore,
        world: None,
        limit: None,
    }
}

#[test]
fn event_assertions_are_equal_across_languages() {
    let shared_catalog = Arc::new(catalog());
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

    let lowered_en = lower_assertion(en, Arc::clone(&shared_catalog));
    let lowered_pl = lower_assertion(pl, shared_catalog);
    assert_eq!(lowered_en, lowered_pl);
}

#[test]
fn event_questions_are_alpha_equivalent() {
    let shared_catalog = Arc::new(catalog());
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

    assert_ne!(en.variables, pl.variables);

    let lowered_en = lower_goal(en, Arc::clone(&shared_catalog));
    let lowered_pl = lower_goal(pl, shared_catalog);
    assert_eq!(
        canonical_goal_hash(&lowered_en),
        canonical_goal_hash(&lowered_pl)
    );
}
