mod support;

use lexflex_language::{LanguageModelValidator, LanguagePackageLoader, MeaningTemplate,
                       MeaningTemplateId};
use lexflex_lingua::{LinguaExpression, SymbolName};
use std::collections::BTreeMap;

#[test]
fn free_local_symbol_is_rejected_by_meaning_validator() {
    let mut model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &support::catalog(),
        )
        .expect("load");
    let sense_id = model.senses.keys().next().cloned().expect("sense");
    model.senses.get_mut(&sense_id).expect("sense").meaning =
        MeaningTemplate::new(
            MeaningTemplateId::new_unchecked("meaning:test:free"),
            LinguaExpression::Variable(SymbolName::new_unchecked("free_symbol")),
            BTreeMap::new(),
        );
    assert!(
        LanguageModelValidator
            .validate(&model, &support::catalog())
            .is_err()
    );
}
