#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{
    ConceptSemantics, ExecutionPolicy, ExpansionMode, ExpansionPolicy, LinguaDeclaration,
    LinguaExpression, LinguaProgram, ProgramId,
};
use lexflex_model::{ConceptId, SemanticExpression};
use std::collections::BTreeMap;

#[test]
fn transparent_definition_expands_into_its_body() {
    let mut capital = support::model::capital_declaration();
    capital.semantics = ConceptSemantics::Defined {
        body: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CITY"),
            bindings: BTreeMap::new(),
        },
    };
    capital.expansion = ExpansionPolicy::Transparent;

    let program = LinguaProgram {
        id: ProgramId::new("test:transparent-definition"),
        declarations: vec![LinguaDeclaration::Concept(capital)],
        entry: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([(
                lexflex_model::ParameterId::new_unchecked("scope"),
                LinguaExpression::Entity(lexflex_model::EntityId::new_unchecked("FRANCE")),
            )]),
        },
    };

    let result = support::compile_and_execute_with_policy(
        program,
        ExecutionPolicy {
            expansion: ExpansionMode::ExpandTransparent,
        },
    )
    .expect("execute");

    assert_eq!(
        result.value,
        SemanticExpression::Apply {
            concept: ConceptId::new_unchecked("CITY"),
            bindings: BTreeMap::new(),
        }
    );
}

#[test]
fn on_demand_definition_expands_only_in_expand_all_defined() {
    let mut capital = support::model::capital_declaration();
    capital.semantics = ConceptSemantics::Defined {
        body: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CITY"),
            bindings: BTreeMap::new(),
        },
    };
    capital.expansion = ExpansionPolicy::OnDemand;

    let program = LinguaProgram {
        id: ProgramId::new("test:on-demand-definition"),
        declarations: vec![LinguaDeclaration::Concept(capital)],
        entry: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([(
                lexflex_model::ParameterId::new_unchecked("scope"),
                LinguaExpression::Entity(lexflex_model::EntityId::new_unchecked("FRANCE")),
            )]),
        },
    };

    let transparent = support::compile_and_execute_with_policy(
        program.clone(),
        ExecutionPolicy {
            expansion: ExpansionMode::ExpandTransparent,
        },
    )
    .expect("execute");

    assert!(matches!(
        transparent.value,
        SemanticExpression::Apply { concept, .. } if concept == ConceptId::new_unchecked("CAPITAL")
    ));

    let expanded = support::compile_and_execute_with_policy(
        program,
        ExecutionPolicy {
            expansion: ExpansionMode::ExpandAllDefined,
        },
    )
    .expect("execute");

    assert_eq!(
        expanded.value,
        SemanticExpression::Apply {
            concept: ConceptId::new_unchecked("CITY"),
            bindings: BTreeMap::new(),
        }
    );
}
