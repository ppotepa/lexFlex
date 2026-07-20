mod support;

use lexflex_language::{LanguageLoadError, LanguagePackageLoader};

#[test]
fn unsafe_package_paths_are_rejected() {
    let root = support::temp_root("lexflex-language");
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
            .load(&root, &support::catalog())
            .expect_err("unsafe path must fail");
        assert!(matches!(err, LanguageLoadError::UnsafePath { .. }));
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn unsupported_manifest_schema_is_rejected() {
    let root = support::temp_root("lexflex-language-schema");
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
        .load(&root, &support::catalog())
        .expect_err("unsupported schema must fail");
    assert!(matches!(err, LanguageLoadError::UnsupportedSchema { .. }));
    let _ = std::fs::remove_dir_all(&root);
}
