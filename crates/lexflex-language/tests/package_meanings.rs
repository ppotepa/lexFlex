mod support;

use lexflex_language::{
    CategoryType, LanguageModelValidator, LanguagePackageLoader, MeaningTemplate,
    MeaningTemplateId, SemanticAnchor,
};
use lexflex_lingua::{LinguaExpression, SymbolName};
use lexflex_model::{ConceptId, EntityId, SemanticType, VariableId};
use std::collections::BTreeMap;

#[test]
fn unknown_entity_anchor_is_rejected() {
    let mut model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &support::catalog(),
        )
        .expect("load");

    let sense_id = model.senses.keys().next().cloned().expect("sense id");
    model.senses.get_mut(&sense_id).expect("sense").anchor =
        Some(SemanticAnchor::Entity(EntityId::new_unchecked("MISSING")));

    let err = LanguageModelValidator
        .validate(&model, &support::catalog())
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
            &support::catalog(),
        )
        .expect("load");

    let sense_id = model.senses.keys().next().cloned().expect("sense id");
    model.senses.get_mut(&sense_id).expect("sense").meaning = MeaningTemplate::new(
        MeaningTemplateId::new_unchecked("meaning:test:query"),
        LinguaExpression::QueryVariable(VariableId::new_unchecked("answer")),
        BTreeMap::new(),
    );

    let err = LanguageModelValidator
        .validate(&model, &support::catalog())
        .expect_err("declarative sense must not use query variables");
    assert!(matches!(
        err,
        lexflex_language::LanguageValidationError::Issue(_)
    ));
}

#[test]
fn free_local_symbol_in_meaning_is_rejected() {
    let mut model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &support::catalog(),
        )
        .expect("load");

    let sense_id = model.senses.keys().next().cloned().expect("sense id");
    model.senses.get_mut(&sense_id).expect("sense").meaning = MeaningTemplate::new(
        MeaningTemplateId::new_unchecked("meaning:test:free-local"),
        LinguaExpression::Variable(SymbolName::new_unchecked("free_symbol")),
        BTreeMap::new(),
    );

    let err = LanguageModelValidator
        .validate(&model, &support::catalog())
        .expect_err("free local symbol must fail");
    assert!(matches!(
        err,
        lexflex_language::LanguageValidationError::Issue(_)
    ));
}

#[test]
fn query_variable_category_type_with_unknown_concept_is_rejected() {
    let mut model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &support::catalog(),
        )
        .expect("load");

    let sense_id = model.senses.keys().next().cloned().expect("sense id");
    model.senses.get_mut(&sense_id).expect("sense").meaning = MeaningTemplate::new(
        MeaningTemplateId::new_unchecked("meaning:test:query-type"),
        LinguaExpression::QueryVariable(VariableId::new_unchecked("answer")),
        BTreeMap::from([(
            VariableId::new_unchecked("answer"),
            CategoryType::Concrete(SemanticType::EntityOf(ConceptId::new_unchecked(
                "MISSING_CONCEPT",
            ))),
        )]),
    );

    let err = LanguageModelValidator
        .validate(&model, &support::catalog())
        .expect_err("invalid query variable category type must fail");
    assert!(matches!(
        err,
        lexflex_language::LanguageValidationError::Issue(_)
    ));
}
