mod support;

use lexflex_language::{LanguagePackageLoader, SlashDirection, SyntacticCategory};

#[test]
fn english_language_contains_compiled_function_category() {
    let model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &support::catalog(),
        )
        .expect("load");

    assert!(model.compiled_senses.values().any(|sense| {
        matches!(
        sense.category,
        SyntacticCategory::Function {
            direction: SlashDirection::Forward | SlashDirection::Backward,
            ..
        }
    )
    }));
}
