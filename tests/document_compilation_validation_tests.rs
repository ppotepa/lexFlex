mod support;

use lexflex::document::compilation::{DocumentCompilationValidator, DocumentCompiler};
use support::{segmented, AlwaysSuccessAnalyzer, PanicAnalyzer, PartialAnalyzer, UnresolvedAnalyzer};

#[test]
fn valid_compilation_passes_validation() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer).compile(document).unwrap();
    assert!(DocumentCompilationValidator::validate(&compilation).is_ok());
}

#[test]
fn compilation_with_panic_result_still_validates() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(PanicAnalyzer).compile(document).unwrap();
    assert!(DocumentCompilationValidator::validate(&compilation).is_ok());
}

#[test]
fn compilation_with_partial_result_validates() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(PartialAnalyzer).compile(document).unwrap();
    assert!(DocumentCompilationValidator::validate(&compilation).is_ok());
}

#[test]
fn compilation_with_unresolved_result_validates() {
    let document = segmented("Ala ma kota.");
    let compilation = DocumentCompiler::new(UnresolvedAnalyzer).compile(document).unwrap();
    assert!(DocumentCompilationValidator::validate(&compilation).is_ok());
}

