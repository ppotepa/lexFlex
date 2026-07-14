mod support;

use lexflex::document::service::DocumentService;
use lexflex::document::translation::{DocumentTranslationOptions, TranslationFallbackPolicy};
use lexflex::api::LexFlexAPI;

#[test]
fn service_compile_and_translate_work() {
    let api = LexFlexAPI::builder().build().unwrap();
    let service = DocumentService::new(&api);
    let compilation = service.compile("Ala ma kota.", "pl").unwrap();
    let translation = service.translate_best_effort("Ala ma kota.", "pl", "en").unwrap();
    assert_eq!(compilation.document.sentences().len(), 1);
    assert!(!translation.output.is_empty());
}

#[test]
fn service_options_validation_rejects_blank_placeholder() {
    let api = LexFlexAPI::builder().build().unwrap();
    let service = DocumentService::new(&api);
    let options = DocumentTranslationOptions {
        fallback_policy: TranslationFallbackPolicy::Placeholder { template: " ".into() },
        trim_generated_output: true,
        reject_empty_generated_output: true,
        preserve_source_whitespace: true,
    };
    let result = service.translate_best_effort_with_options("Ala ma kota.", "pl", "en", &options);
    assert!(result.is_err());
}
