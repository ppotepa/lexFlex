use lexflex_lingua::{CompileContext, CompiledModelContext, LinguaCompiler, LinguaExpression};
use lexflex_model::ConceptCatalog;
use std::sync::Arc;

#[test]
fn empty_model_context_compiles_entry_against_verified_context() {
    let compiler = LinguaCompiler::try_new(Arc::new(ConceptCatalog::default())).expect("compiler");
    let first: CompiledModelContext = compiler
        .compile_model_declarations(&[])
        .expect("model context");
    let second = compiler
        .compile_model_declarations(&[])
        .expect("model context");
    assert_eq!(first.declaration_hash(), second.declaration_hash());
    compiler
        .compile_entry_with_context(
            &LinguaExpression::Value(true.into()),
            &first,
            &CompileContext::default(),
        )
        .expect("entry");
}

#[test]
fn model_context_binds_catalog_identity() {
    let first_catalog = Arc::new(ConceptCatalog::default());
    let second_catalog = Arc::new(ConceptCatalog::default());
    let first = LinguaCompiler::try_new(first_catalog).expect("compiler");
    let second = LinguaCompiler::try_new(second_catalog).expect("compiler");
    let context = first
        .compile_model_declarations(&[])
        .expect("model context");

    second
        .compile_entry_with_context(
            &LinguaExpression::Value(true.into()),
            &context,
            &CompileContext::default(),
        )
        .expect("equivalent catalogs should share identity");
}

#[test]
fn verified_entry_contains_entry_only_report() {
    let compiler = LinguaCompiler::try_new(Arc::new(ConceptCatalog::default())).expect("compiler");
    let model = Arc::new(compiler.compile_model_declarations(&[]).expect("model"));
    let entry = compiler
        .compile_verified_entry(
            &LinguaExpression::Value(true.into()),
            model,
            &CompileContext::default(),
        )
        .expect("verified entry");
    assert_eq!(entry.verification().declaration_count, 0);
    assert_eq!(
        entry.entry_type(),
        &lexflex_model::SemanticType::Value(lexflex_model::ValueType::Boolean)
    );
}
