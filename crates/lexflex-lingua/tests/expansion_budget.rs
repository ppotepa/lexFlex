#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{
    runtime::RuntimeError, ConceptSemantics, ExecutionBudget, ExecutionPolicy, ExpansionMode,
    ExpansionPolicy, LinguaDeclaration, LinguaExpression, LinguaInterpreter, LinguaProgram,
    ProgramId,
};
use lexflex_model::{ConceptId, EntityId, ParameterId, SemanticExpression};
use std::collections::BTreeMap;

fn transparent_defined_capital_program() -> LinguaProgram {
    let mut capital = support::model::capital_declaration();
    capital.self_parameter = None;
    capital.semantics = ConceptSemantics::Defined {
        body: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CITY"),
            bindings: BTreeMap::new(),
        },
    };
    capital.expansion = ExpansionPolicy::Transparent;

    LinguaProgram {
        id: ProgramId::new_unchecked("test:expansion-budget"),
        declarations: vec![LinguaDeclaration::Concept(capital)],
        entry: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([(
                ParameterId::new_unchecked("scope"),
                LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
            )]),
        },
    }
}

#[test]
fn expansion_depth_zero_is_rejected() {
    let compiled = support::compiler::test_compiler()
        .compile(&transparent_defined_capital_program())
        .expect("compile");

    let error = LinguaInterpreter::with_policy(
        ExecutionBudget {
            max_expansion_depth: 0,
            ..ExecutionBudget::default()
        },
        ExecutionPolicy {
            expansion: ExpansionMode::ExpandTransparent,
        },
    )
    .execute(&compiled)
    .expect_err("depth");

    assert!(matches!(
        error,
        RuntimeError::ExpansionDepthExceeded { max: 0 }
    ));
}

#[test]
fn expansion_count_zero_is_rejected() {
    let compiled = support::compiler::test_compiler()
        .compile(&transparent_defined_capital_program())
        .expect("compile");

    let error = LinguaInterpreter::with_policy(
        ExecutionBudget {
            max_expansions: 0,
            ..ExecutionBudget::default()
        },
        ExecutionPolicy {
            expansion: ExpansionMode::ExpandTransparent,
        },
    )
    .execute(&compiled)
    .expect_err("count");

    assert!(matches!(
        error,
        RuntimeError::ExpansionCountExceeded { max: 0 }
    ));
}

#[test]
fn exact_expansion_budget_boundary_is_allowed() {
    let compiled = support::compiler::test_compiler()
        .compile(&transparent_defined_capital_program())
        .expect("compile");

    let result = LinguaInterpreter::with_policy(
        ExecutionBudget {
            max_expansions: 1,
            max_expansion_depth: 1,
            ..ExecutionBudget::default()
        },
        ExecutionPolicy {
            expansion: ExpansionMode::ExpandTransparent,
        },
    )
    .execute(&compiled)
    .expect("boundary should pass");

    assert_eq!(
        result.execution.value,
        SemanticExpression::Apply {
            concept: ConceptId::new_unchecked("CITY"),
            bindings: BTreeMap::new(),
        }
    );
}
