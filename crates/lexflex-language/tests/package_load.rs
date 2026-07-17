mod support;

use lexflex_language::{LanguagePackageLoader, SyntacticCategory};

#[test]
fn english_package_loads() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en");
    let model = LanguagePackageLoader
        .load(&root, &support::catalog())
        .expect("load");
    assert_eq!(model.lexemes.len(), 13);
    assert_eq!(model.senses.len(), 13);
    assert!(model.senses.values().any(
        |sense| matches!(sense.base_category, SyntacticCategory::Function { .. }),
    ));
    assert_eq!(
        model.form_index.lookup("paris"),
        &[lexflex_language::FormId::new_unchecked("form:en:Paris")]
    );
    assert_eq!(
        model
            .sense_index
            .lookup(&lexflex_language::LexemeId::new_unchecked(
                "lexeme:en:paris:proper"
            )),
        &[lexflex_language::LexicalSenseId::new_unchecked(
            "sense:en:paris:entity"
        )]
    );
    assert!(model
        .senses
        .values()
        .any(|sense| sense.id.as_str() == "sense:en:see:event"));
}
