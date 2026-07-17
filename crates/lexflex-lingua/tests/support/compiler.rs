use lexflex_lingua::{ExecutionPolicy, LinguaCompiler, LinguaInterpreter};

pub fn test_compiler() -> LinguaCompiler {
    LinguaCompiler::new(std::sync::Arc::new(super::model::kernel_catalog()))
}

pub fn compile_and_execute(
    program: lexflex_lingua::LinguaProgram,
) -> Result<lexflex_lingua::ExecutionResult, lexflex_lingua::runtime::RuntimeError> {
    let compiled = test_compiler().compile(&program).expect("compile");
    LinguaInterpreter::default().execute(&compiled).map(
        |result| {
            result.execution
        },
    )
}

pub fn compile_and_execute_with_policy(
    program: lexflex_lingua::LinguaProgram,
    policy: ExecutionPolicy,
) -> Result<lexflex_lingua::ExecutionResult, lexflex_lingua::runtime::RuntimeError> {
    let compiled = test_compiler().compile(&program).expect("compile");
    LinguaInterpreter::with_policy(Default::default(), policy)
        .execute(&compiled)
        .map(|result| result.execution)
}
