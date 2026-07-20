mod support;

use lexflex_language::{LanguagePackageLoader, SyntacticCategory};
use std::collections::BTreeSet;

#[test]
fn english_package_loads() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en");
    let model = LanguagePackageLoader
        .load(&root, &support::catalog())
        .expect("load");
    let required_lexemes = [
        "lexeme:en:paris:proper",
        "lexeme:en:france:proper",
        "lexeme:en:capital:noun",
        "lexeme:en:see:verb",
        "lexeme:en:what:question",
        "lexeme:en:who:question",
        "lexeme:en:thomas:proper",
        "lexeme:en:tommy:proper",
        "lexeme:en:isabelle:proper",
        "lexeme:en:izzy:proper",
    ];
    let loaded_lexemes: BTreeSet<_> = model.lexemes.keys().map(|id| id.as_str()).collect();
    for lexeme in required_lexemes {
        assert!(
            loaded_lexemes.contains(lexeme),
            "missing required lexeme: {lexeme}"
        );
    }
    let required_senses = [
        "sense:en:paris:entity",
        "sense:en:france:entity",
        "sense:en:capital:city",
        "sense:en:see:event",
        "sense:en:what:question",
        "sense:en:who:question",
        "sense:en:thomas:entity",
        "sense:en:tommy:entity",
        "sense:en:isabelle:entity",
        "sense:en:izzy:entity",
    ];
    let loaded_senses: BTreeSet<_> = model.senses.keys().map(|id| id.as_str()).collect();
    for sense in required_senses {
        assert!(
            loaded_senses.contains(sense),
            "missing required sense: {sense}"
        );
    }
    assert!(model.lexemes.len() >= required_lexemes.len());
    assert!(model.senses.len() >= required_senses.len());
    assert!(model
        .senses
        .values()
        .any(|sense| matches!(sense.base_category, SyntacticCategory::Function { .. }),));
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
