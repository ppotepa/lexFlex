#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{
    compiler::CompiledConceptSemantics, runtime::RuntimeError, ExecutionPolicy, ExpansionMode,
    LinguaDeclaration, LinguaExpression, LinguaProgram, ProgramId,
};
use lexflex_model::{ConceptId, EntityId, ParameterId};
use std::collections::BTreeMap;

#[test]
fn direct_expansion_cycle_is_rejected() {
    let mut capital = support::model::capital_declaration();
    capital.self_parameter = None;

    let mut compiled = support::compiler::test_compiler()
        .compile(&LinguaProgram {
            id: ProgramId::new_unchecked("test:expansion-cycle"),
            declarations: vec![LinguaDeclaration::Concept(capital)],
            entry: LinguaExpression::ApplyConcept {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    ParameterId::new_unchecked("scope"),
                    LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            },
        })
        .expect("compile");

    compiled
        .concepts
        .get_mut(&ConceptId::new_unchecked("CAPITAL"))
        .expect("capital concept")
        .semantics = CompiledConceptSemantics::Defined {
        body: lexflex_lingua::compiler::ResolvedExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([(
                ParameterId::new_unchecked("scope"),
                lexflex_lingua::compiler::ResolvedExpression::Entity(EntityId::new_unchecked(
                    "FRANCE",
                )),
            )]),
        },
    };

    let error = lexflex_lingua::LinguaInterpreter::with_policy(
        Default::default(),
        ExecutionPolicy {
            expansion: ExpansionMode::ExpandTransparent,
        },
    )
    .execute(&compiled)
    .expect_err("cycle must fail");

    assert!(matches!(
        error,
        RuntimeError::ExpansionCycle { path }
            if path
                == vec![
                    ConceptId::new_unchecked("CAPITAL"),
                    ConceptId::new_unchecked("CAPITAL"),
                ]
    ));
}
