#[path = "support/mod.rs"]
mod support;

use lexflex_model::{ConceptId, EntityId, ParameterId, SemanticExpression};
use std::collections::BTreeMap;

#[test]
fn capital_scope_application_executes() {
    let program = support::model::capital_program();
    let result = support::compile_and_execute(program).expect("execute");
    assert_eq!(
        result.value,
        SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        }
    );
}

#[test]
fn capital_program_stays_canonical() {
    let result = support::compile_and_execute(support::model::capital_program()).expect("execute");
    assert_eq!(
        result.value,
        SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    ParameterId::new_unchecked("scope"),
                    SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        }
    );
}
