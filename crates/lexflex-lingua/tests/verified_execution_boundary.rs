use lexflex_lingua::{
    CompileContext, ExecutionPolicy, LinguaCompiler, LinguaExpression, LinguaInterpreter,
    SemanticType,
};
use lexflex_model::{ConceptCatalog, SemanticValue, ValueType};
use std::sync::Arc;

#[test]
fn verified_standalone_program_executes_through_public_api() {
    let compiler = LinguaCompiler::try_new(Arc::new(ConceptCatalog::default())).expect("compiler");
    let program = compiler
        .compile(&lexflex_lingua::LinguaProgram {
            id: lexflex_lingua::ProgramId::new_unchecked("test:verified-standalone"),
            declarations: Vec::new(),
            entry: LinguaExpression::Value(SemanticValue::from(true)),
        })
        .expect("verified program");

    let result = LinguaInterpreter::default()
        .execute_program(&program, ExecutionPolicy::default())
        .expect("verified executable");

    assert_eq!(result.entry_type, SemanticType::Value(ValueType::Boolean));
}

#[test]
fn verified_entry_executes_against_shared_model() {
    let compiler = LinguaCompiler::try_new(Arc::new(ConceptCatalog::default())).expect("compiler");
    let model = Arc::new(
        compiler
            .compile_model_declarations(&[])
            .expect("verified model"),
    );
    let entry = compiler
        .compile_verified_entry(
            &LinguaExpression::Value(SemanticValue::from(true)),
            model.clone(),
            &CompileContext::default(),
        )
        .expect("verified entry");

    let result = LinguaInterpreter::default()
        .execute_entry(&entry, ExecutionPolicy::default())
        .expect("verified entry executable");

    assert_eq!(result.entry_type, SemanticType::Value(ValueType::Boolean));
    assert!(Arc::ptr_eq(&model, entry.model()));
}
