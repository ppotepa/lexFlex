#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{LambdaParameter, LinguaExpression, LinguaProgram, ProgramId, SemanticType};
use lexflex_model::{EntityId, ParameterId, SemanticExpression};
use std::collections::BTreeMap;

#[test]
fn identity_executes() {
    let program = support::model::identity_program();
    let result = support::compile_and_execute(program).expect("execute");
    assert_eq!(
        result.value,
        SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))
    );
}

#[test]
fn nested_closure_captures_outer_value() {
    let outer = ParameterId::new_unchecked("outer");
    let inner = ParameterId::new_unchecked("inner");
    let outer_name = lexflex_lingua::SymbolName::new_unchecked("outer");
    let inner_name = lexflex_lingua::SymbolName::new_unchecked("inner");

    let program = LinguaProgram {
        id: ProgramId::new_unchecked("test:nested-closure"),
        declarations: Vec::new(),
        entry: LinguaExpression::Call {
            callee: Box::new(LinguaExpression::Call {
                callee: Box::new(LinguaExpression::Lambda {
                    parameters: vec![
                        LambdaParameter {
                            name: outer_name.clone(),
                            parameter_id: outer.clone(),
                            value_type: SemanticType::Entity,
                        },
                    ],
                    body: Box::new(LinguaExpression::Lambda {
                        parameters: vec![
                            LambdaParameter {
                                name: inner_name,
                                parameter_id: inner.clone(),
                                value_type: SemanticType::Entity,
                            },
                        ],
                        body: Box::new(LinguaExpression::Variable(outer_name)),
                    }),
                }),
                arguments: BTreeMap::from(
                    [
                        (
                            outer,
                            LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
                        ),
                    ],
                ),
            }),
            arguments: BTreeMap::from(
                [
                    (
                        inner,
                        LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
                    ),
                ],
            ),
        },
    };

    let result = support::compile_and_execute(program).expect("execute");
    assert_eq!(
        result.value,
        SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))
    );
}

#[test]
fn lexical_shadowing_prefers_inner_binding() {
    let program = support::model::let_program();
    let result = support::compile_and_execute(program).expect("execute");
    assert_eq!(
        result.value,
        SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))
    );
}
