use lexflex_lingua::{LinguaCompiler, LinguaExpression};
use lexflex_model::{canonical_hash, ConceptCatalog, SemanticValue};
use std::sync::Arc;

#[test]
fn model_context_hash_is_stable_for_same_inputs() {
    let compiler = LinguaCompiler::try_new(Arc::new(ConceptCatalog::default())).expect("compiler");
    let first = compiler.compile_model_declarations(&[]).expect("model");
    let second = compiler.compile_model_declarations(&[]).expect("model");

    assert_eq!(first.catalog_hash(), second.catalog_hash());
    assert_eq!(first.declaration_hash(), second.declaration_hash());
    assert_eq!(first.context_hash(), second.context_hash());
}

#[test]
fn catalog_identity_mismatch_is_rejected() {
    let first_catalog = Arc::new(ConceptCatalog::default());
    let second_catalog = Arc::new(ConceptCatalog {
        parents: std::collections::BTreeMap::from([(
            lexflex_model::ConceptId::new_unchecked("CHANGED"),
            std::collections::BTreeSet::new(),
        )]),
        ..ConceptCatalog::default()
    });
    let first = LinguaCompiler::try_new(first_catalog).expect("compiler");
    let second = LinguaCompiler::try_new(second_catalog).expect("compiler");
    let model = first.compile_model_declarations(&[]).expect("model");

    let error = second
        .compile_verified_entry(
            &LinguaExpression::Value(SemanticValue::from(true)),
            Arc::new(model),
            &lexflex_lingua::CompileContext::default(),
        )
        .expect_err("different catalogs must not share a model");

    assert!(matches!(
        error,
        lexflex_lingua::compiler::CompileError::ModelContextMismatch { .. }
    ));
}

#[test]
fn declaration_identity_is_part_of_context_hash() {
    let compiler = LinguaCompiler::try_new(Arc::new(ConceptCatalog::default())).expect("compiler");
    let empty = compiler.compile_model_declarations(&[]).expect("model");
    let declaration_hash = canonical_hash(&[lexflex_lingua::LinguaDeclaration::Function(
        lexflex_lingua::FunctionDeclaration {
            declaration_id: lexflex_lingua::DeclarationId::new_unchecked("function:test"),
            function_id: lexflex_lingua::FunctionId::new_unchecked("TEST"),
            name: lexflex_lingua::SymbolName::new_unchecked("test"),
            parameters: Vec::new(),
            result_type: lexflex_model::SemanticType::Value(lexflex_model::ValueType::Boolean),
            body: LinguaExpression::Value(SemanticValue::from(true)),
        },
    )])
    .expect("hash");

    assert_ne!(empty.declaration_hash(), &declaration_hash);
}
