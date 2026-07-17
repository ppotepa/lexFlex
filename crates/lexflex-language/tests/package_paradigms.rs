mod support;

use lexflex_language::{LanguageLoadError, LanguagePackageLoader};

#[test]
fn paradigm_unknown_form_is_rejected() {
    let root = support::temp_root("lexflex-language-paradigm-form");
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
    ).expect("write manifest");
    std::fs::write(
        root.join("lexemes.ron"),
        r#"[
    (
        id: "lexeme:test:one",
        language: "en",
        lemma: "alpha",
        normalized_lemma: "alpha",
    ),
]"#,
    ).expect("write lexemes");
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
    ).expect("write senses");
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
    ).expect("write forms");
    std::fs::write(
        root.join("paradigms.ron"),
        r#"[
    (
        id: "paradigm:test:one",
        language: "en",
        form_ids: [
            "form:test:paradigm",
        ],
    ),
]"#,
    ).expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &support::catalog())
        .expect_err("unknown paradigm form must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn duplicate_paradigm_form_id_is_rejected() {
    let root = support::temp_root("lexflex-language-paradigm-duplicate");
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
    ).expect("write manifest");
    std::fs::write(
        root.join("lexemes.ron"),
        r#"[
    (
        id: "lexeme:test:one",
        language: "en",
        lemma: "alpha",
        normalized_lemma: "alpha",
    ),
]"#,
    ).expect("write lexemes");
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
    ).expect("write senses");
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
    ).expect("write forms");
    std::fs::write(
        root.join("paradigms.ron"),
        r#"[
    (
        id: "paradigm:test:one",
        language: "en",
        form_ids: [
            "form:test:one",
            "form:test:one",
        ],
    ),
]"#,
    ).expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &support::catalog())
        .expect_err("duplicate paradigm form id must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}
