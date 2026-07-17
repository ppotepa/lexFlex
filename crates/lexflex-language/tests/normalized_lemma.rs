mod support;

use lexflex_language::{LanguageLoadError, LanguagePackageLoader};

#[test]
fn invalid_normalized_lemma_is_rejected() {
    let root = support::temp_root("lexflex-language-normalized-lemma");
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 1,
    package_id: "lexflex:language:test:normalized",
    language: "en",
    lexemes: "lexemes.ron",
    senses: "senses.ron",
    forms: "forms.ron",
    paradigms: "paradigms.ron",
)"#,
    ).expect("manifest");
    std::fs::write(
        root.join("lexemes.ron"),
        r#"[
    (
        id: "lexeme:test:one",
        language: "en",
        lemma: "Alpha",
        normalized_lemma: "wrong",
    ),
]"#,
    ).expect("lexemes");
    std::fs::write(root.join("senses.ron"), "[]").expect("senses");
    std::fs::write(root.join("forms.ron"), "[]").expect("forms");
    std::fs::write(root.join("paradigms.ron"), "[]").expect("paradigms");

    let err = LanguagePackageLoader
        .load(&root, &support::catalog())
        .expect_err("normalized lemma must fail");
    assert!(matches!(err, LanguageLoadError::Validation(_)));
    let _ = std::fs::remove_dir_all(&root);
}
