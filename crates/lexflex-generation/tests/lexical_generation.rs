use lexflex_generation::{
    build_generation_plan, generate, generate_entity, generate_with_style, GenerationError,
    GenerationNode, GenerationRequest, GenerationResult, GenerationStyle, SemanticRealizationKind,
    SemanticRealizationRequest,
};
use lexflex_language::LanguagePackageLoader;
use lexflex_language::{FeatureName, FeatureStructure, FeatureValue};
use lexflex_model::{ConceptCatalog, EntityDefinition, EntityId, SemanticExpression};
use std::collections::BTreeMap;
use std::path::Path;

fn language_for(code: &str) -> lexflex_language::LanguageModel {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let mut catalog: ConceptCatalog =
        ron::from_str(&std::fs::read_to_string(root.join("model/concepts.ron")).expect("catalog"))
            .expect("catalog parse");
    #[derive(serde::Deserialize)]
    struct EntityPackage {
        entities: BTreeMap<EntityId, EntityDefinition>,
    }
    catalog.entities = ron::from_str::<EntityPackage>(
        &std::fs::read_to_string(root.join("model/entities.ron")).expect("entities"),
    )
    .expect("entities parse")
    .entities;
    LanguagePackageLoader
        .load(&root.join("languages").join(code), &catalog)
        .expect("language")
}

fn language() -> lexflex_language::LanguageModel {
    language_for("en")
}

#[test]
fn generation_plan_exposes_verified_semantic_identity() {
    let expression = SemanticExpression::Entity(EntityId::new_unchecked("PARIS"));
    let plan = build_generation_plan(&expression, &language()).expect("generation plan");

    assert_eq!(plan.language().as_str(), "en");
    assert_eq!(
        plan.semantic_hash(),
        &expression.canonical_hash().expect("canonical expression")
    );
    assert!(matches!(plan.root(), GenerationNode::Lexical { .. }));
}

#[test]
fn text_translation_request_preserves_language_and_ambiguity_policy() {
    let request = lexflex_generation::TextTranslationRequest {
        input: "Paris is the capital of France.".into(),
        source_language: lexflex_model::LanguageId::new_unchecked("en"),
        target_language: lexflex_model::LanguageId::new_unchecked("pl"),
        ambiguity_policy: lexflex_generation::TranslationAmbiguityPolicy::Reject,
        include_trace: true,
    };
    let encoded = ron::to_string(&request).expect("request RON");
    let decoded: lexflex_generation::TextTranslationRequest =
        ron::from_str(&encoded).expect("request round-trip");
    assert_eq!(decoded, request);
}

#[test]
fn entity_generation_uses_language_lexical_anchor() {
    let generated = generate_entity(&EntityId::new_unchecked("PARIS"), &language()).expect("text");
    assert_eq!(generated.text, "Paris");
}

#[test]
fn equal_lexical_candidates_return_ambiguous_result() {
    let mut language = language();
    let base_sense = language
        .compiled_senses
        .values()
        .find(|sense| {
            sense.anchor.as_ref()
                == Some(&lexflex_language::SemanticAnchor::Entity(
                    EntityId::new_unchecked("PARIS"),
                ))
        })
        .cloned()
        .expect("Paris sense");
    let base_lexeme = language
        .lexemes
        .get(&base_sense.lexeme_id)
        .cloned()
        .expect("Paris lexeme");
    let base_form = language
        .forms
        .values()
        .find(|form| form.lexeme_id == base_sense.lexeme_id)
        .cloned()
        .expect("Paris form");

    let alternate_lexeme_id =
        lexflex_language::LexemeId::new_unchecked("lexeme:en:paris:alternate");
    let alternate_sense_id =
        lexflex_language::LexicalSenseId::new_unchecked("sense:en:paris:alternate");
    let alternate_form_id = lexflex_language::FormId::new_unchecked("form:en:Paris:alternate");

    let mut alternate_lexeme = base_lexeme;
    alternate_lexeme.id = alternate_lexeme_id.clone();
    language
        .lexemes
        .insert(alternate_lexeme_id.clone(), alternate_lexeme);

    let mut alternate_sense = base_sense;
    alternate_sense.id = alternate_sense_id;
    alternate_sense.lexeme_id = alternate_lexeme_id.clone();
    language
        .compiled_senses
        .insert(alternate_sense.id.clone(), alternate_sense);

    let mut alternate_form = base_form;
    alternate_form.id = alternate_form_id;
    alternate_form.lexeme_id = alternate_lexeme_id;
    language
        .forms
        .insert(alternate_form.id.clone(), alternate_form);

    let result = lexflex_generation::generate_result(
        &GenerationRequest {
            expression: SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
            include_trace: false,
        },
        &language,
    )
    .expect("generation result");

    assert!(matches!(result, GenerationResult::Ambiguous { .. }));
}

#[test]
fn feature_constrained_generation_uses_language_forms() {
    let mut required = FeatureStructure::default();
    required.values.insert(
        FeatureName::new_unchecked("part_of_speech"),
        FeatureValue::new_unchecked("noun"),
    );
    let generated = lexflex_generation::generate_entity_with_features(
        &EntityId::new_unchecked("PARIS"),
        &required,
        true,
        &language(),
    )
    .expect("feature-constrained form");
    assert_eq!(generated.text, "Paris");
    assert!(generated.trace.is_some());
}

#[test]
fn generation_trace_records_sense_form_and_agreement() {
    let mut required = FeatureStructure::default();
    required.values.insert(
        FeatureName::new_unchecked("part_of_speech"),
        FeatureValue::new_unchecked("noun"),
    );
    let generated = lexflex_generation::generate_entity_with_features(
        &EntityId::new_unchecked("PARIS"),
        &required,
        true,
        &language(),
    )
    .expect("generation");
    let trace = generated.trace.expect("trace");
    let events = &trace[0].events;
    assert!(events.iter().any(|event| matches!(
        event,
        lexflex_generation::GenerationTraceEvent::SelectSense { .. }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        lexflex_generation::GenerationTraceEvent::SelectForm { .. }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        lexflex_generation::GenerationTraceEvent::ApplyAgreement { .. }
    )));
}

#[test]
fn conflicting_generation_features_are_rejected_before_form_selection() {
    let mut required = FeatureStructure::default();
    required.values.insert(
        FeatureName::new_unchecked("part_of_speech"),
        FeatureValue::new_unchecked("verb"),
    );
    let error = lexflex_generation::generate_entity_with_features(
        &EntityId::new_unchecked("PARIS"),
        &required,
        false,
        &language(),
    )
    .expect_err("sense/form feature conflict");
    assert_eq!(error, GenerationError::IncompatibleFeatures);
}

#[test]
fn polish_case_features_select_declared_morphological_forms() {
    let language = language_for("pl");
    let mut genitive = FeatureStructure::default();
    genitive.values.insert(
        FeatureName::new_unchecked("case"),
        FeatureValue::new_unchecked("genitive"),
    );
    let france = lexflex_generation::generate_entity_with_features(
        &EntityId::new_unchecked("FRANCE"),
        &genitive,
        false,
        &language,
    )
    .expect("Polish genitive form");
    assert_eq!(france.text, "Francji");

    let mut instrumental = FeatureStructure::default();
    instrumental.values.insert(
        FeatureName::new_unchecked("case"),
        FeatureValue::new_unchecked("instrumental"),
    );
    let capital = lexflex_generation::generate_anchor_with_features(
        &lexflex_language::SemanticAnchor::Concept(lexflex_model::ConceptId::new_unchecked(
            "CAPITAL",
        )),
        &instrumental,
        false,
        &language,
    )
    .expect("Polish instrumental form");
    assert_eq!(capital.text, "stolicą");
}

#[test]
fn unsupported_expression_is_typed() {
    let error = lexflex_generation::generate(
        &lexflex_generation::GenerationRequest {
            expression: SemanticExpression::And(Vec::new()),
            include_trace: false,
        },
        &language(),
    )
    .expect_err("value has no lexical anchor");
    assert_eq!(error, GenerationError::UnsupportedExpression);
}

#[test]
fn scalar_values_have_deterministic_realizations() {
    let language = language();
    for (expression, expected) in [
        (SemanticExpression::Value(true.into()), "true"),
        (SemanticExpression::Value(7_i64.into()), "7"),
        (SemanticExpression::Value("Paris".into()), "Paris"),
    ] {
        let result = generate(
            &GenerationRequest {
                expression,
                include_trace: false,
            },
            &language,
        )
        .expect("scalar realization");
        assert_eq!(result.text, expected);
    }
}

#[test]
fn compound_generation_uses_explicit_style() {
    let result = generate_with_style(
        &GenerationRequest {
            expression: SemanticExpression::And(vec![
                SemanticExpression::Value("Paris".into()),
                SemanticExpression::Value("France".into()),
            ]),
            include_trace: false,
        },
        &language(),
        &GenerationStyle {
            conjunction: " and ".into(),
            disjunction: " or ".into(),
            negation_prefix: "not ".into(),
            existential_prefix: "exists ".into(),
            universal_prefix: "forall ".into(),
        },
    )
    .expect("compound generation");
    assert_eq!(result.text, "Paris and France");
}

#[test]
fn compound_generation_preserves_all_lexical_trace_entries() {
    let generated = generate_with_style(
        &GenerationRequest {
            expression: SemanticExpression::And(vec![
                SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
                SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
            ]),
            include_trace: true,
        },
        &language(),
        &GenerationStyle::default(),
    )
    .expect("compound generation");
    assert_eq!(generated.trace.as_ref().map(Vec::len), Some(2));
}

#[test]
fn semantic_negation_and_coordination_generate_in_both_languages() {
    let expression = SemanticExpression::And(vec![
        SemanticExpression::Not(Box::new(sees_expression())),
        sees_expression(),
    ]);
    let english = generate(
        &GenerationRequest {
            expression: expression.clone(),
            include_trace: false,
        },
        &language_for("en"),
    )
    .expect("English compound negation");
    assert_eq!(english.text, "not Tom sees Iza and Tom sees Iza");

    let polish = generate(
        &GenerationRequest {
            expression,
            include_trace: false,
        },
        &language_for("pl"),
    )
    .expect("Polish compound negation");
    assert_eq!(polish.text, "nie Tomek widzi Izę i Tomek widzi Izę");
}

#[test]
fn semantic_or_generates_in_both_languages() {
    let expression = SemanticExpression::Or(vec![sees_expression(), sees_expression()]);

    let en = generate(
        &GenerationRequest {
            expression: expression.clone(),
            include_trace: false,
        },
        &language_for("en"),
    )
    .expect("English OR should generate");
    let pl = generate(
        &GenerationRequest {
            expression,
            include_trace: false,
        },
        &language_for("pl"),
    )
    .expect("Polish OR should generate");

    assert_eq!(en.text, "Tom sees Iza or Tom sees Iza");
    assert_eq!(pl.text, "Tomek widzi Izę albo Tomek widzi Izę");
}

#[test]
fn generation_budget_rejects_deep_expression_before_realization() {
    let mut expression = SemanticExpression::Value(true.into());
    for _ in 0..4 {
        expression = SemanticExpression::Not(Box::new(expression));
    }
    let error = lexflex_generation::generate_with_budget(
        &GenerationRequest {
            expression,
            include_trace: false,
        },
        &language(),
        &GenerationStyle::default(),
        lexflex_generation::GenerationBudget {
            max_depth: 2,
            max_nodes: 100,
            max_output_bytes: 100,
        },
    )
    .expect_err("depth budget");
    assert_eq!(error, GenerationError::BudgetExceeded("expression depth"));
}

#[test]
fn styled_generation_enforces_default_budget() {
    let mut expression = SemanticExpression::Value(true.into());
    for _ in 0..130 {
        expression = SemanticExpression::Not(Box::new(expression));
    }
    let error = generate_with_style(
        &GenerationRequest {
            expression,
            include_trace: false,
        },
        &language(),
        &GenerationStyle::default(),
    )
    .expect_err("styled generation must enforce its budget");
    assert_eq!(error, GenerationError::BudgetExceeded("expression depth"));
}

#[test]
fn translation_with_budget_rejects_unbounded_expression() {
    let mut expression = SemanticExpression::Value(true.into());
    for _ in 0..4 {
        expression = SemanticExpression::Not(Box::new(expression));
    }
    let error = lexflex_generation::realize_semantic_with_budget(
        &lexflex_generation::SemanticRealizationRequest {
            expression,
            kind: lexflex_generation::SemanticRealizationKind::Assertion,
            projection: Vec::new(),
            include_trace: false,
        },
        &language(),
        &GenerationStyle::default(),
        lexflex_generation::GenerationBudget {
            max_depth: 1,
            max_nodes: 100,
            max_output_bytes: 100,
        },
    )
    .expect_err("translation must enforce the supplied budget");
    assert_eq!(error, GenerationError::BudgetExceeded("expression depth"));
}

#[test]
fn goal_realization_uses_language_question_contract() {
    let result = lexflex_generation::realize_semantic(
        &lexflex_generation::SemanticRealizationRequest {
            expression: SemanticExpression::Value(true.into()),
            kind: lexflex_generation::SemanticRealizationKind::Goal,
            projection: vec![lexflex_model::VariableId::new_unchecked("answer")],
            include_trace: false,
        },
        &language(),
    )
    .expect("goal realization");

    assert_eq!(result.text, "What is true?");
}

#[test]
fn polish_goal_realization_uses_polish_question_contract() {
    let result = lexflex_generation::realize_semantic(
        &SemanticRealizationRequest {
            expression: SemanticExpression::Value(true.into()),
            kind: SemanticRealizationKind::Goal,
            projection: vec![lexflex_model::VariableId::new_unchecked("answer")],
            include_trace: false,
        },
        &language_for("pl"),
    )
    .expect("goal realization");

    assert_eq!(result.text, "Jaka jest true?");
}

#[test]
fn goal_realization_preserves_query_variable_in_expression() {
    let result = lexflex_generation::realize_semantic(
        &SemanticRealizationRequest {
            expression: SemanticExpression::Equals {
                left: Box::new(SemanticExpression::Variable(
                    lexflex_model::VariableId::new_unchecked("country"),
                )),
                right: Box::new(SemanticExpression::Value("France".into())),
            },
            kind: SemanticRealizationKind::Goal,
            projection: vec![lexflex_model::VariableId::new_unchecked("country")],
            include_trace: false,
        },
        &language(),
    )
    .expect("query realization");

    assert_eq!(result.text, "What is France?");
}

#[test]
fn question_prefix_counts_toward_output_budget() {
    let error = lexflex_generation::realize_semantic_with_budget(
        &SemanticRealizationRequest {
            expression: SemanticExpression::Value(true.into()),
            kind: SemanticRealizationKind::Goal,
            projection: vec![lexflex_model::VariableId::new_unchecked("answer")],
            include_trace: false,
        },
        &language(),
        &GenerationStyle::default(),
        lexflex_generation::GenerationBudget {
            max_depth: 10,
            max_nodes: 10,
            max_output_bytes: 5,
        },
    )
    .expect_err("question prefix must be budgeted");

    assert_eq!(error, GenerationError::BudgetExceeded("output bytes"));
}

#[test]
fn duplicate_goal_projection_is_rejected_before_realization() {
    let variable = lexflex_model::VariableId::new_unchecked("answer");
    let error = lexflex_generation::realize_semantic(
        &SemanticRealizationRequest {
            expression: SemanticExpression::Value(true.into()),
            kind: SemanticRealizationKind::Goal,
            projection: vec![variable.clone(), variable.clone()],
            include_trace: false,
        },
        &language(),
    )
    .expect_err("duplicate projection must fail");

    assert_eq!(error, GenerationError::DuplicateProjection(variable));
}

#[test]
fn apply_rejects_unknown_valency_binding() {
    let error = generate(
        &GenerationRequest {
            expression: SemanticExpression::Apply {
                concept: lexflex_model::ConceptId::new_unchecked("CAPITAL"),
                bindings: std::iter::once((
                    lexflex_model::ParameterId::new_unchecked("unknown"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                ))
                .collect(),
            },
            include_trace: false,
        },
        &language(),
    )
    .expect_err("unknown valency binding must fail");

    assert_eq!(
        error,
        GenerationError::UnknownValencyBinding(lexflex_model::ParameterId::new_unchecked(
            "unknown"
        ))
    );
}

#[test]
fn apply_uses_declared_surface_relation_and_slot_order() {
    let generated = generate(
        &GenerationRequest {
            expression: SemanticExpression::Apply {
                concept: lexflex_model::ConceptId::new_unchecked("CAPITAL"),
                bindings: [(
                    lexflex_model::ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]
                .into_iter()
                .collect(),
            },
            include_trace: true,
        },
        &language(),
    )
    .expect("surface relation realization");
    assert_eq!(generated.text, "capital of France");
    assert!(generated
        .trace
        .expect("trace")
        .iter()
        .flat_map(|entry| entry.events.iter())
        .any(|event| matches!(
            event,
            lexflex_generation::GenerationTraceEvent::BindValency {
                relation: Some(_),
                ..
            }
        )));
}

fn sees_expression() -> SemanticExpression {
    SemanticExpression::Apply {
        concept: lexflex_model::ConceptId::new_unchecked("SEE_EVENT"),
        bindings: [
            (
                lexflex_model::ParameterId::new_unchecked("agent"),
                SemanticExpression::Entity(EntityId::new_unchecked("TOM")),
            ),
            (
                lexflex_model::ParameterId::new_unchecked("patient"),
                SemanticExpression::Entity(EntityId::new_unchecked("IZA")),
            ),
        ]
        .into_iter()
        .collect(),
    }
}

#[test]
fn negation_generation_uses_english_language_realization() {
    let generated = generate(
        &GenerationRequest {
            expression: SemanticExpression::Not(Box::new(sees_expression())),
            include_trace: false,
        },
        &language_for("en"),
    )
    .expect("negative assertion generation");
    assert_eq!(generated.text, "not Tom sees Iza");
}

#[test]
fn negation_generation_uses_polish_language_realization() {
    let generated = generate(
        &GenerationRequest {
            expression: SemanticExpression::Not(Box::new(sees_expression())),
            include_trace: false,
        },
        &language_for("pl"),
    )
    .expect("negative assertion generation");
    assert_eq!(generated.text, "nie Tomek widzi Izę");
}

#[test]
fn relational_and_structural_expressions_use_recursive_generation() {
    let language = language();
    let expression = SemanticExpression::Equals {
        left: Box::new(SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))),
        right: Box::new(SemanticExpression::Value("Paris".into())),
    };
    let generated = generate_with_style(
        &GenerationRequest {
            expression,
            include_trace: false,
        },
        &language,
        &GenerationStyle {
            conjunction: " and ".into(),
            disjunction: " or ".into(),
            negation_prefix: "not ".into(),
            existential_prefix: "exists ".into(),
            universal_prefix: "forall ".into(),
        },
    )
    .expect("structural expression should generate");
    assert!(generated.text.contains(" is "));
}

#[test]
fn generation_preserves_quantifier_semantics() {
    let language = language();
    let expression = SemanticExpression::Exists {
        variable: lexflex_model::VariableId::new_unchecked("x"),
        value_type: lexflex_model::SemanticType::Boolean,
        body: Box::new(SemanticExpression::Value(true.into())),
    };
    let generated = generate(
        &GenerationRequest {
            expression,
            include_trace: false,
        },
        &language,
    )
    .expect("quantifier should have an explicit realization");
    assert!(generated.text.contains("there exists"));
}

#[test]
fn translation_preserves_source_semantic_identity() {
    let expression = SemanticExpression::And(vec![
        SemanticExpression::Value("Paris".into()),
        SemanticExpression::Value("France".into()),
    ]);
    let source_hash = expression.canonical_hash().expect("source hash");
    let translated = lexflex_generation::realize_semantic_with_style(
        &lexflex_generation::SemanticRealizationRequest {
            expression: expression.clone(),
            kind: lexflex_generation::SemanticRealizationKind::Assertion,
            projection: Vec::new(),
            include_trace: false,
        },
        &language(),
        &GenerationStyle {
            conjunction: " et ".into(),
            disjunction: " ou ".into(),
            negation_prefix: "ne ".into(),
            existential_prefix: "il existe ".into(),
            universal_prefix: "pour tout ".into(),
        },
    )
    .expect("translation");
    assert_eq!(translated.text, "Paris et France");
    assert_eq!(expression.canonical_hash().unwrap(), source_hash);
}

#[test]
fn translation_reuses_semantic_generation_path() {
    let result = lexflex_generation::realize_semantic(
        &lexflex_generation::SemanticRealizationRequest {
            expression: lexflex_model::SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
            kind: lexflex_generation::SemanticRealizationKind::Assertion,
            projection: Vec::new(),
            include_trace: true,
        },
        &language(),
    )
    .expect("translation");
    assert_eq!(result.text, "Paris");
    assert!(result.trace.is_some());
}
