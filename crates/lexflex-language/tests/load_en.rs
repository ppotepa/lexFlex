use lexflex_language::{
    CategoryType, LanguageLoadError, LanguageModelValidator, LanguagePackageLoader, MeaningTemplate,
    MeaningTemplateId, SemanticAnchor, SyntacticCategory, ValencySlot,
};
use lexflex_lingua::LinguaExpression;
use lexflex_model::{ConceptCatalog, EntityId, VariableId};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

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
fn english_package_loads() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en");
    let model = LanguagePackageLoader.load(&root, &catalog()).expect("load");
    assert_eq!(model.lexemes.len(), 13);
    assert_eq!(model.senses.len(), 13);
    assert!(model
        .senses
        .values()
        .any(|sense| matches!(sense.base_category, SyntacticCategory::Function { .. })));
    assert_eq!(
        model.form_index.lookup("paris"),
        &[lexflex_language::FormId::new("form:en:Paris")]
    );
    assert_eq!(
        model
            .sense_index
            .lookup(&lexflex_language::LexemeId::new("lexeme:en:paris:proper")),
        &[lexflex_language::LexicalSenseId::new(
            "sense:en:paris:entity"
        )]
    );
    assert!(model
        .senses
        .values()
        .any(|sense| sense.id.as_str() == "sense:en:see:event"));
}

#[test]
fn unsafe_package_paths_are_rejected() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lexflex-language-{stamp}"));
    std::fs::create_dir_all(&root).expect("create root");
    for (field, unsafe_path) in [
        ("lexemes", "../escape.ron"),
        ("senses", "../escape.ron"),
        ("forms", "../escape.ron"),
        ("paradigms", "../escape.ron"),
        ("lexemes", "/escape.ron"),
        ("senses", "/escape.ron"),
        ("forms", "/escape.ron"),
        ("paradigms", "/escape.ron"),
    ] {
        for file in ["lexemes.ron", "senses.ron", "forms.ron", "paradigms.ron"] {
            std::fs::copy(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../data/languages/en")
                    .join(file),
                root.join(file),
            )
            .expect("copy baseline");
        }
        std::fs::write(
            root.join("manifest.ron"),
            format!(
                r#"(
    schema: 1,
    package_id: "lexflex:language:test:unsafe",
    language: "en",
    lexemes: "{lexemes}",
    senses: "{senses}",
    forms: "{forms}",
    paradigms: "{paradigms}",
)"#,
                lexemes = if field == "lexemes" {
                    unsafe_path
                } else {
                    "lexemes.ron"
                },
                senses = if field == "senses" {
                    unsafe_path
                } else {
                    "senses.ron"
                },
                forms = if field == "forms" {
                    unsafe_path
                } else {
                    "forms.ron"
                },
                paradigms = if field == "paradigms" {
                    unsafe_path
                } else {
                    "paradigms.ron"
                }
            ),
        )
        .expect("write manifest");

        let err = LanguagePackageLoader
            .load(&root, &catalog())
            .expect_err("unsafe path must fail");
        assert!(matches!(err, LanguageLoadError::UnsafePath { .. }));
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn unsupported_manifest_schema_is_rejected() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lexflex-language-schema-{stamp}"));
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 2,
    package_id: "lexflex:language:test:schema",
    language: "en",
    lexemes: "lexemes.ron",
    senses: "senses.ron",
    forms: "forms.ron",
    paradigms: "paradigms.ron",
)"#,
    )
    .expect("write manifest");
    std::fs::write(root.join("lexemes.ron"), "[]").expect("write lexemes");
    std::fs::write(root.join("senses.ron"), "[]").expect("write senses");
    std::fs::write(root.join("forms.ron"), "[]").expect("write forms");
    std::fs::write(root.join("paradigms.ron"), "[]").expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &catalog())
        .expect_err("unsupported schema must fail");
    assert!(matches!(err, LanguageLoadError::UnsupportedSchema { .. }));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn lexeme_language_mismatch_is_rejected() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lexflex-language-lang-mismatch-{stamp}"));
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 1,
    package_id: "lexflex:language:test:mismatch",
    language: "en",
    lexemes: "lexemes.ron",
    senses: "senses.ron",
    forms: "forms.ron",
    paradigms: "paradigms.ron",
)"#,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("lexemes.ron"),
        r#"[
    (
        id: "lexeme:test:one",
        language: "fr",
        lemma: "bonjour",
    ),
]"#,
    )
    .expect("write lexemes");
    std::fs::write(root.join("senses.ron"), "[]").expect("write senses");
    std::fs::write(root.join("forms.ron"), "[]").expect("write forms");
    std::fs::write(root.join("paradigms.ron"), "[]").expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &catalog())
        .expect_err("language mismatch must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn feature_conflict_between_sense_and_form_is_rejected() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lexflex-language-feature-conflict-{stamp}"));
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 1,
    package_id: "lexflex:language:test:feature-conflict",
    language: "en",
    lexemes: "lexemes.ron",
    senses: "senses.ron",
    forms: "forms.ron",
    paradigms: "paradigms.ron",
)"#,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("lexemes.ron"),
        r#"[
    (
        id: "lexeme:test:one",
        language: "en",
        lemma: "alpha",
    ),
]"#,
    )
    .expect("write lexemes");
    std::fs::write(
        root.join("senses.ron"),
        r#"[
    (
        id: "sense:test:one",
        lexeme_id: "lexeme:test:one",
        anchor: Some(
            Concept(
                "ENTITY",
            ),
        ),
        base_category: Atom(
            kind: NounPhrase,
            semantic_type: Concrete(
                EntityOf("CITY"),
            ),
            features: (
                values: {
                    "number": "singular",
                },
            ),
        ),
        meaning: (
            id: "meaning:test:one",
            expression: Entity(
                "PARIS",
            ),
            query_variables: {},
        ),
        features: (
            values: {
                "number": "singular",
            },
        ),
        valency: [],
        priority: 0,
    ),
]"#,
    )
    .expect("write senses");
    std::fs::write(
        root.join("forms.ron"),
        r#"[
    (
        id: "form:test:one",
        lexeme_id: "lexeme:test:one",
        surface: "alpha",
        normalized: "alpha",
        features: (
            values: {
                "number": "plural",
            },
        ),
    ),
]"#,
    )
    .expect("write forms");
    std::fs::write(root.join("paradigms.ron"), "[]").expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &catalog())
        .expect_err("feature conflict must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn form_normalized_mismatch_is_rejected() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lexflex-language-form-normalized-{stamp}"));
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 1,
    package_id: "lexflex:language:test:form-normalized",
    language: "en",
    lexemes: "lexemes.ron",
    senses: "senses.ron",
    forms: "forms.ron",
    paradigms: "paradigms.ron",
)"#,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("lexemes.ron"),
        r#"[
    (
        id: "lexeme:test:one",
        language: "en",
        lemma: "alpha",
    ),
]"#,
    )
    .expect("write lexemes");
    std::fs::write(
        root.join("senses.ron"),
        r#"[
    (
        id: "sense:test:one",
        lexeme_id: "lexeme:test:one",
        anchor: Some(
            Concept(
                "ENTITY",
            ),
        ),
        base_category: Atom(
            kind: NounPhrase,
            semantic_type: Concrete(
                EntityOf("CITY"),
            ),
            features: (
                values: {},
            ),
        ),
        meaning: (
            id: "meaning:test:one",
            expression: Entity(
                "PARIS",
            ),
            query_variables: {},
        ),
        features: (
            values: {},
        ),
        valency: [],
        priority: 0,
    ),
]"#,
    )
    .expect("write senses");
    std::fs::write(
        root.join("forms.ron"),
        r#"[
    (
        id: "form:test:one",
        lexeme_id: "lexeme:test:one",
        surface: "Alpha",
        normalized: "wrong",
        features: (
            values: {},
        ),
    ),
]"#,
    )
    .expect("write forms");
    std::fs::write(root.join("paradigms.ron"), "[]").expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &catalog())
        .expect_err("form normalized mismatch must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn paradigm_form_normalized_mismatch_is_rejected() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lexflex-language-paradigm-form-{stamp}"));
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 1,
    package_id: "lexflex:language:test:paradigm-form",
    language: "en",
    lexemes: "lexemes.ron",
    senses: "senses.ron",
    forms: "forms.ron",
    paradigms: "paradigms.ron",
)"#,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("lexemes.ron"),
        r#"[
    (
        id: "lexeme:test:one",
        language: "en",
        lemma: "alpha",
    ),
]"#,
    )
    .expect("write lexemes");
    std::fs::write(
        root.join("senses.ron"),
        r#"[
    (
        id: "sense:test:one",
        lexeme_id: "lexeme:test:one",
        anchor: Some(
            Concept(
                "ENTITY",
            ),
        ),
        base_category: Atom(
            kind: NounPhrase,
            semantic_type: Concrete(
                EntityOf("CITY"),
            ),
            features: (
                values: {},
            ),
        ),
        meaning: (
            id: "meaning:test:one",
            expression: Entity(
                "PARIS",
            ),
            query_variables: {},
        ),
        features: (
            values: {},
        ),
        valency: [],
        priority: 0,
    ),
]"#,
    )
    .expect("write senses");
    std::fs::write(
        root.join("forms.ron"),
        r#"[
    (
        id: "form:test:one",
        lexeme_id: "lexeme:test:one",
        surface: "alpha",
        normalized: "alpha",
        features: (
            values: {},
        ),
    ),
]"#,
    )
    .expect("write forms");
    std::fs::write(
        root.join("paradigms.ron"),
        r#"[
    (
        id: "paradigm:test:one",
        language: "en",
        forms: [
            (
                id: "form:test:paradigm",
                lexeme_id: "lexeme:test:one",
                surface: "Alpha",
                normalized: "wrong",
                features: (
                    values: {},
                ),
            ),
        ],
    ),
]"#,
    )
    .expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &catalog())
        .expect_err("paradigm form normalized mismatch must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn paradigm_form_must_match_canonical_form() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lexflex-language-paradigm-canonical-{stamp}"));
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 1,
    package_id: "lexflex:language:test:paradigm-canonical",
    language: "en",
    lexemes: "lexemes.ron",
    senses: "senses.ron",
    forms: "forms.ron",
    paradigms: "paradigms.ron",
)"#,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("lexemes.ron"),
        r#"[
    (
        id: "lexeme:test:one",
        language: "en",
        lemma: "alpha",
    ),
]"#,
    )
    .expect("write lexemes");
    std::fs::write(
        root.join("senses.ron"),
        r#"[
    (
        id: "sense:test:one",
        lexeme_id: "lexeme:test:one",
        anchor: Some(
            Concept(
                "ENTITY",
            ),
        ),
        base_category: Atom(
            kind: NounPhrase,
            semantic_type: Concrete(
                EntityOf("CITY"),
            ),
            features: (
                values: {},
            ),
        ),
        meaning: (
            id: "meaning:test:one",
            expression: Entity(
                "PARIS",
            ),
            query_variables: {},
        ),
        features: (
            values: {},
        ),
        valency: [],
        priority: 0,
    ),
]"#,
    )
    .expect("write senses");
    std::fs::write(
        root.join("forms.ron"),
        r#"[
    (
        id: "form:test:one",
        lexeme_id: "lexeme:test:one",
        surface: "alpha",
        normalized: "alpha",
        features: (
            values: {},
        ),
    ),
]"#,
    )
    .expect("write forms");
    std::fs::write(
        root.join("paradigms.ron"),
        r#"[
    (
        id: "paradigm:test:one",
        language: "en",
        forms: [
            (
                id: "form:test:one",
                lexeme_id: "lexeme:test:one",
                surface: "alpha",
                normalized: "alpha",
                features: (
                    values: {
                        "number": "plural",
                    },
                ),
            ),
        ],
    ),
]"#,
    )
    .expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &catalog())
        .expect_err("paradigm canonical mismatch must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn unknown_entity_anchor_is_rejected() {
    let mut model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &catalog(),
        )
        .expect("load");

    let sense_id = model.senses.keys().next().cloned().expect("sense id");
    model.senses.get_mut(&sense_id).expect("sense").anchor =
        Some(SemanticAnchor::Entity(EntityId::new_unchecked("MISSING")));

    let err = LanguageModelValidator
        .validate(&model, &catalog())
        .expect_err("unknown entity anchor must fail");
    assert!(matches!(
        err,
        lexflex_language::LanguageValidationError::Issue(
            lexflex_language::LanguageValidationIssue::UnknownEntityAnchor { .. }
        )
    ));
}

#[test]
fn declarative_query_variable_is_rejected() {
    let mut model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &catalog(),
        )
        .expect("load");

    let sense_id = model.senses.keys().next().cloned().expect("sense id");
    model.senses.get_mut(&sense_id).expect("sense").meaning = MeaningTemplate::new(
        MeaningTemplateId::new("meaning:test:query"),
        LinguaExpression::QueryVariable(VariableId::new_unchecked("answer")),
        BTreeMap::new(),
    );

    let err = LanguageModelValidator
        .validate(&model, &catalog())
        .expect_err("declarative sense must not use query variables");
    assert!(matches!(
        err,
        lexflex_language::LanguageValidationError::Issue(_)
    ));
}

#[test]
fn unknown_valency_parameter_is_rejected() {
    let mut model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &catalog(),
        )
        .expect("load");

    let capital = model
        .senses
        .values_mut()
        .find(|sense| sense.id.as_str() == "sense:en:capital:city")
        .expect("capital sense");
    capital.valency = vec![ValencySlot {
        id: lexflex_language::ValencySlotId::new("valency:test:invalid"),
        parameter: lexflex_model::ParameterId::new_unchecked("missing"),
        argument_category: SyntacticCategory::noun_phrase(
            CategoryType::Concrete(lexflex_model::SemanticType::EntityOf(
                lexflex_model::ConceptId::new_unchecked("POLITY"),
            )),
            Default::default(),
        ),
        surface_relation: Some(lexflex_language::SurfaceRelationId::new("of-complement")),
        direction: lexflex_language::SlashDirection::Forward,
        application_rank: 0,
        required: true,
    }];

    let err = LanguageModelValidator
        .validate(&model, &catalog())
        .expect_err("unknown valency parameter must fail");
    assert!(matches!(
        err,
        lexflex_language::LanguageValidationError::Issue(
            lexflex_language::LanguageValidationIssue::UnknownValencyParameter { .. }
        )
    ));
}

#[test]
fn duplicate_valency_rank_is_rejected() {
    let mut model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &catalog(),
        )
        .expect("load");

    let capital = model
        .senses
        .values_mut()
        .find(|sense| sense.id.as_str() == "sense:en:capital:city")
        .expect("capital sense");
    capital.valency.push(ValencySlot {
        id: lexflex_language::ValencySlotId::new("valency:test:duplicate"),
        parameter: lexflex_model::ParameterId::new_unchecked("scope"),
        argument_category: SyntacticCategory::noun_phrase(
            CategoryType::Concrete(lexflex_model::SemanticType::EntityOf(
                lexflex_model::ConceptId::new_unchecked("POLITY"),
            )),
            Default::default(),
        ),
        surface_relation: Some(lexflex_language::SurfaceRelationId::new("of-complement")),
        direction: lexflex_language::SlashDirection::Forward,
        application_rank: 0,
        required: true,
    });

    let err = LanguageModelValidator
        .validate(&model, &catalog())
        .expect_err("duplicate rank must fail");
    assert!(matches!(
        err,
        lexflex_language::LanguageValidationError::Issue(
            lexflex_language::LanguageValidationIssue::DuplicateValencyApplicationRank { .. }
        )
    ));
}

#[test]
fn category_semantic_type_mismatch_is_rejected() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lexflex-language-category-mismatch-{stamp}"));
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 1,
    package_id: "lexflex:language:test:category-mismatch",
    language: "en",
    lexemes: "lexemes.ron",
    senses: "senses.ron",
    forms: "forms.ron",
    paradigms: "paradigms.ron",
)"#,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("lexemes.ron"),
        r#"[
    (
        id: "lexeme:test:one",
        language: "en",
        lemma: "alpha",
    ),
]"#,
    )
    .expect("write lexemes");
    std::fs::write(
        root.join("senses.ron"),
        r#"[
    (
        id: "sense:test:one",
        lexeme_id: "lexeme:test:one",
        anchor: Some(
            Concept(
                "MISSING",
            ),
        ),
        base_category: Atom(
            kind: NounPhrase,
            semantic_type: Concrete(
                EntityOf("CITY"),
            ),
            features: (
                values: {},
            ),
        ),
        meaning: (
            id: "meaning:test:one",
            expression: Entity(
                "PARIS",
            ),
            query_variables: {},
        ),
        features: (
            values: {},
        ),
        valency: [],
        priority: 0,
    ),
]"#,
    )
    .expect("write senses");
    std::fs::write(
        root.join("forms.ron"),
        r#"[
    (
        id: "form:test:one",
        lexeme_id: "lexeme:test:one",
        surface: "alpha",
        normalized: "alpha",
        features: (
            values: {},
        ),
    ),
]"#,
    )
    .expect("write forms");
    std::fs::write(root.join("paradigms.ron"), "[]").expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &catalog())
        .expect_err("category mismatch must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}
