mod support;

use lexflex::document::compilation::DocumentCompiler;
use lexflex::document::translation::DocumentTranslationValidator;
use lexflex::document::translation::{BestEffortDocumentTranslator, DocumentTranslationOptions, TranslationFallbackPolicy};
use support::{segmented, AlwaysSuccessAnalyzer, EmptyGenerator, PrefixGenerator};

#[test]
fn valid_translation_passes_validation() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document).unwrap();
    let translation = BestEffortDocumentTranslator::new(PrefixGenerator { prefix: "EN:".into() })
        .translate(&compilation, lexflex::core::interlingua::LanguageId::new("en"))
        .unwrap();
    assert!(DocumentTranslationValidator::validate(&compilation, &translation).is_ok());
}

#[test]
fn fallback_translation_passes_validation() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document).unwrap();
    let translation = BestEffortDocumentTranslator::new(EmptyGenerator)
        .translate(&compilation, lexflex::core::interlingua::LanguageId::new("en"))
        .unwrap();
    assert!(DocumentTranslationValidator::validate(&compilation, &translation).is_ok());
}

#[test]
fn blank_placeholder_is_rejected_by_options() {
    let options = DocumentTranslationOptions {
        fallback_policy: TranslationFallbackPolicy::Placeholder { template: "   ".into() },
        trim_generated_output: true,
        reject_empty_generated_output: true,
        preserve_source_whitespace: true,
    };
    assert!(options.validate().is_err());
}
