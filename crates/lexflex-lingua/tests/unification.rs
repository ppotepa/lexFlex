#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{unify, Substitution, UnificationContext, UnificationMode};
use lexflex_model::{
    ConceptId, EntityId, ParameterId, SemanticExpression, SemanticType, VariableId,
};
use std::collections::BTreeMap;
use std::sync::Arc;

fn context(variable_types: BTreeMap<VariableId, SemanticType>) -> UnificationContext {
    UnificationContext {
        catalog: Arc::new(support::model::kernel_catalog()),
        variable_types,
        mode: UnificationMode::Pattern,
        max_depth: 128,
    }
}

#[test]
fn variable_binding_succeeds() {
    let mut substitution = Substitution::default();
    let variable = VariableId::new_unchecked("city");
    let pattern = SemanticExpression::Variable(variable.clone());
    let candidate = SemanticExpression::Entity(EntityId::new_unchecked("PARIS"));
    unify(
        &pattern,
        &candidate,
        &context(BTreeMap::from([(
            variable.clone(),
            SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        )])),
        &mut substitution,
    )
    .expect("unify");
    assert_eq!(substitution.get(&variable), Some(&candidate));
}

#[test]
fn conflicting_binding_fails() {
    let mut substitution = Substitution::default();
    let variable = VariableId::new_unchecked("city");
    substitution
        .bind(
            variable.clone(),
            SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
        )
        .expect("first bind");
    let result = substitution.bind(
        variable,
        SemanticExpression::Entity(EntityId::new_unchecked("WARSAW")),
    );
    assert!(result.is_err());
}

#[test]
fn apply_unifies_by_parameter() {
    let mut substitution = Substitution::default();
    let pattern = SemanticExpression::Apply {
        concept: ConceptId::new_unchecked("CAPITAL"),
        bindings: BTreeMap::from([(
            ParameterId::new_unchecked("scope"),
            SemanticExpression::Variable(VariableId::new_unchecked("scope")),
        )]),
    };
    let candidate = SemanticExpression::Apply {
        concept: ConceptId::new_unchecked("CAPITAL"),
        bindings: BTreeMap::from([(
            ParameterId::new_unchecked("scope"),
            SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
        )]),
    };
    unify(
        &pattern,
        &candidate,
        &context(BTreeMap::from([(
            VariableId::new_unchecked("scope"),
            SemanticType::EntityOf(ConceptId::new_unchecked("POLITY")),
        )])),
        &mut substitution,
    )
    .expect("unify");
    assert_eq!(
        substitution.get(&VariableId::new_unchecked("scope")),
        Some(&SemanticExpression::Entity(EntityId::new_unchecked(
            "FRANCE"
        )))
    );
}

#[test]
fn satisfies_unifies_structurally() {
    let mut substitution = Substitution::default();
    let pattern = SemanticExpression::Satisfies {
        subject: Box::new(SemanticExpression::Variable(VariableId::new_unchecked(
            "city",
        ))),
        predicate: Box::new(SemanticExpression::Apply {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([(
                ParameterId::new_unchecked("scope"),
                SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
            )]),
        }),
    };
    let candidate = SemanticExpression::Satisfies {
        subject: Box::new(SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))),
        predicate: Box::new(SemanticExpression::Apply {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([(
                ParameterId::new_unchecked("scope"),
                SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
            )]),
        }),
    };
    unify(
        &pattern,
        &candidate,
        &context(BTreeMap::from([(
            VariableId::new_unchecked("city"),
            SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        )])),
        &mut substitution,
    )
    .expect("unify");
    assert_eq!(
        substitution.get(&VariableId::new_unchecked("city")),
        Some(&SemanticExpression::Entity(EntityId::new_unchecked(
            "PARIS"
        )))
    );
}

#[test]
fn no_match_is_reported() {
    let mut substitution = Substitution::default();
    let result = unify(
        &SemanticExpression::Entity(EntityId::new_unchecked("PARIS")),
        &SemanticExpression::Entity(EntityId::new_unchecked("WARSAW")),
        &context(BTreeMap::new()),
        &mut substitution,
    );
    assert!(result.is_err());
}
