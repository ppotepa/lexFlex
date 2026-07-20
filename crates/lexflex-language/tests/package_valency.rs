mod support;

use lexflex_language::{
    CategoryType, LanguageModelValidator, LanguagePackageLoader, SyntacticCategory, ValencySlot,
};

#[test]
fn unknown_valency_parameter_is_rejected() {
    let mut model = LanguagePackageLoader
        .load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/languages/en"),
            &support::catalog(),
        )
        .expect("load");

    let capital = model
        .senses
        .values_mut()
        .find(|sense| sense.id.as_str() == "sense:en:capital:city")
        .expect("capital sense");
    capital.valency = vec![ValencySlot {
        id: lexflex_language::ValencySlotId::new_unchecked("valency:test:invalid"),
        parameter: lexflex_model::ParameterId::new_unchecked("missing"),
        argument_category: SyntacticCategory::noun_phrase(
            CategoryType::Concrete(lexflex_model::SemanticType::EntityOf(
                lexflex_model::ConceptId::new_unchecked("POLITY"),
            )),
            Default::default(),
        ),
        surface_relation: Some(lexflex_language::SurfaceRelationId::new_unchecked(
            "of-complement",
        )),
        direction: lexflex_language::SlashDirection::Forward,
        application_rank: 0,
        required: true,
    }];

    let err = LanguageModelValidator
        .validate(&model, &support::catalog())
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
            &support::catalog(),
        )
        .expect("load");

    let capital = model
        .senses
        .values_mut()
        .find(|sense| sense.id.as_str() == "sense:en:capital:city")
        .expect("capital sense");
    capital.valency.push(ValencySlot {
        id: lexflex_language::ValencySlotId::new_unchecked("valency:test:duplicate"),
        parameter: lexflex_model::ParameterId::new_unchecked("scope"),
        argument_category: SyntacticCategory::noun_phrase(
            CategoryType::Concrete(lexflex_model::SemanticType::EntityOf(
                lexflex_model::ConceptId::new_unchecked("POLITY"),
            )),
            Default::default(),
        ),
        surface_relation: Some(lexflex_language::SurfaceRelationId::new_unchecked(
            "of-complement",
        )),
        direction: lexflex_language::SlashDirection::Forward,
        application_rank: 0,
        required: true,
    });

    let err = LanguageModelValidator
        .validate(&model, &support::catalog())
        .expect_err("duplicate rank must fail");
    assert!(matches!(
        err,
        lexflex_language::LanguageValidationError::Issue(
            lexflex_language::LanguageValidationIssue::DuplicateValencyApplicationRank { .. }
        )
    ));
}
