pub mod compiler;
pub mod model;

#[allow(unused_imports)]
pub use compiler::*;

#[test]
fn touch_support_symbols() {
    use model::*;

    let _ = compiler::test_compiler();
    let _ = compiler::compile_and_execute(capital_program());
    let _ = compiler::compile_and_execute_with_policy(
        identity_program(),
        lexflex_lingua::ExecutionPolicy::default(),
    );
    let _ = capital_declaration();
    let _ = city_declaration();
    let _ = population_declaration();
    let _ = recursive_capital_declaration();
    let _ = capital_program();
    let _ = identity_program();
    let _ = let_program();
    let _ = deeply_nested_not_program(1);
}
