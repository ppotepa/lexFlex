mod support;

use lexflex_language::{LanguageLoadError, LanguagePackageLoader};

#[test]
fn lexeme_language_mismatch_is_rejected() {
    let root = support::temp_root("lexflex-language-lang-mismatch");
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
        normalized_lemma: "bonjour",
    ),
]"#,
    )
    .expect("write lexemes");
    std::fs::write(root.join("senses.ron"), "[]").expect("write senses");
    std::fs::write(root.join("forms.ron"), "[]").expect("write forms");
    std::fs::write(root.join("paradigms.ron"), "[]").expect("write paradigms");

    let err = LanguagePackageLoader
        .load(&root, &support::catalog())
        .expect_err("language mismatch must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn feature_conflict_between_sense_and_form_is_rejected() {
    let root = support::temp_root("lexflex-language-feature-conflict");
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
        normalized_lemma: "alpha",
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
        .load(&root, &support::catalog())
        .expect_err("feature conflict must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn form_normalized_mismatch_is_rejected() {
    let root = support::temp_root("lexflex-language-form-normalized");
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
        normalized_lemma: "alpha",
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
        .load(&root, &support::catalog())
        .expect_err("form normalized mismatch must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn category_semantic_type_mismatch_is_rejected() {
    let root = support::temp_root("lexflex-language-category-mismatch");
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
        normalized_lemma: "alpha",
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
        .load(&root, &support::catalog())
        .expect_err("category mismatch must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}
