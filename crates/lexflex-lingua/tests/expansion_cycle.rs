#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{LinguaDeclaration, LinguaExpression, LinguaProgram, ProgramId};
use lexflex_model::{ConceptId, EntityId, ParameterId};
use std::collections::BTreeMap;

#[test]
fn recursive_concept_definition_is_rejected_before_execution() {
    let mut capital = support::model::capital_declaration();
    capital.self_parameter = None;
    capital.semantics = lexflex_lingua::ConceptSemantics::Defined {
        body: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([(
                ParameterId::new_unchecked("scope"),
                LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
            )]),
        },
    };

    let error = support::compiler::test_compiler()
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
        .expect_err("recursive definitions must be rejected");

    assert!(error.to_string().contains("recursive concept definitions"));
}
