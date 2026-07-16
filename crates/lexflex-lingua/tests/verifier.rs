#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{
    compiler::CompileError, LinguaDeclaration, LinguaExpression, LinguaProgram, ProgramId,
};
use lexflex_model::EntityId;

#[test]
fn duplicate_concept_is_rejected() {
    let declaration = support::model::capital_declaration();
    let program = LinguaProgram {
        id: ProgramId::new("test:duplicate-concept"),
        declarations: vec![
            LinguaDeclaration::Concept(declaration.clone()),
            LinguaDeclaration::Concept(declaration),
        ],
        entry: LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
    };

    let error = support::test_compiler()
        .compile(&program)
        .expect_err("duplicate concept");
    assert!(
        matches!(error, CompileError::Diagnostic(message) if message.message.contains("duplicate concept id"))
    );
}

#[test]
fn recursive_concept_is_rejected() {
    let program = LinguaProgram {
        id: ProgramId::new("test:recursive"),
        declarations: vec![LinguaDeclaration::Concept(
            support::model::recursive_capital_declaration(),
        )],
        entry: LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
    };

    let error = support::test_compiler()
        .compile(&program)
        .expect_err("recursive concept");
    assert!(
        matches!(error, CompileError::Diagnostic(message) if message.message.contains("recursive concept"))
    );
}

#[test]
fn too_deep_expression_is_rejected() {
    let mut expression = LinguaExpression::Entity(EntityId::new_unchecked("PARIS"));
    for _ in 0..300 {
        expression = LinguaExpression::Not(Box::new(expression));
    }
    let program = LinguaProgram {
        id: ProgramId::new("test:too-deep"),
        declarations: Vec::new(),
        entry: expression,
    };

    let error = support::test_compiler()
        .compile(&program)
        .expect_err("too deep");
    assert!(
        matches!(error, CompileError::Diagnostic(message) if message.message.contains("expression limits exceeded"))
    );
}

#[test]
fn free_local_symbol_is_rejected_by_compiler() {
    let program = LinguaProgram {
        id: ProgramId::new("test:free-local"),
        declarations: Vec::new(),
        entry: LinguaExpression::Variable(lexflex_lingua::SymbolName::new("missing")),
    };

    let error = support::test_compiler()
        .compile(&program)
        .expect_err("free local");
    assert!(
        matches!(error, CompileError::Diagnostic(message) if message.message.contains("unknown symbol"))
    );
}
