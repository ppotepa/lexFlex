mod support;

use support::compiler::compile_and_execute_with_policy;
use support::model::capital_program;

#[test]
fn defined_expansion_executes_with_stable_step_count() {
    let result = compile_and_execute_with_policy(
        capital_program(),
        lexflex_lingua::ExecutionPolicy {
            expansion: lexflex_lingua::ExpansionMode::PreserveApplications,
        },
    ).expect("execute");
    assert!(result.steps > 0);
}
