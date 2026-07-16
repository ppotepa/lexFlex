#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{runtime::RuntimeError, ExecutionBudget, LinguaInterpreter};

#[test]
fn max_steps_stops_pathological_program() {
    let interpreter = LinguaInterpreter::new(ExecutionBudget {
        max_steps: 3,
        ..ExecutionBudget::default()
    });

    let compiled = support::test_compiler()
        .compile(&support::model::deeply_nested_not_program(50))
        .expect("compile");

    let error = interpreter.execute(&compiled).expect_err("budget");
    assert!(matches!(error, RuntimeError::BudgetExceeded));
}

#[test]
fn max_call_depth_stops_nested_calls() {
    let interpreter = LinguaInterpreter::new(ExecutionBudget {
        max_call_depth: 0,
        ..ExecutionBudget::default()
    });

    let compiled = support::test_compiler()
        .compile(&support::model::identity_program())
        .expect("compile");

    let error = interpreter.execute(&compiled).expect_err("call depth");
    assert!(matches!(error, RuntimeError::BudgetExceeded));
}

#[test]
fn trace_limit_is_enforced() {
    let interpreter = LinguaInterpreter::new(ExecutionBudget {
        max_trace_events: 1,
        ..ExecutionBudget::default()
    });

    let compiled = support::test_compiler()
        .compile(&support::model::capital_program())
        .expect("compile");

    let error = interpreter.execute(&compiled).expect_err("trace");
    assert!(matches!(error, RuntimeError::BudgetExceeded));
}
