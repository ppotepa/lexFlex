mod support;

use lexflex_lingua::{unify, Substitution, UnificationContext, UnificationMode};
use lexflex_model::{ConceptId, EntityId, ParameterId, SemanticExpression, SemanticType};
use std::collections::BTreeMap;
use std::sync::Arc;
use support::model::kernel_catalog;

#[test]
fn alpha_equivalent_exists_unifies() {
    let catalog = Arc::new(kernel_catalog());
    let pattern = SemanticExpression::Exists {
        variable: lexflex_model::VariableId::new_unchecked("x"),
        value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        body: Box::new(SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(
                lexflex_model::VariableId::new_unchecked("x"),
            )),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from(
                    [
                        (
                            ParameterId::new_unchecked("scope"),
                            SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                        ),
                    ],
                ),
            }),
        }),
    };
    let candidate = SemanticExpression::Exists {
        variable: lexflex_model::VariableId::new_unchecked("y"),
        value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        body: Box::new(SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(
                lexflex_model::VariableId::new_unchecked("y"),
            )),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from(
                    [
                        (
                            ParameterId::new_unchecked("scope"),
                            SemanticExpression::Entity(EntityId::new_unchecked("FRANCE")),
                        ),
                    ],
                ),
            }),
        }),
    };

    let context = UnificationContext {
        catalog,
        variable_types: BTreeMap::new(),
        mode: UnificationMode::Exact,
        max_depth: 128,
    };
    let mut substitution = Substitution::default();
    assert!(unify(&pattern, &candidate, &context, &mut substitution).is_ok());
}

#[test]
fn different_quantifier_kinds_do_not_unify() {
    let catalog = Arc::new(kernel_catalog());
    let pattern = SemanticExpression::Exists {
        variable: lexflex_model::VariableId::new_unchecked("x"),
        value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        body: Box::new(SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(
                lexflex_model::VariableId::new_unchecked("x"),
            )),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CITY"),
                bindings: BTreeMap::new(),
            }),
        }),
    };
    let candidate = SemanticExpression::ForAll {
        variable: lexflex_model::VariableId::new_unchecked("y"),
        value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        body: Box::new(SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(
                lexflex_model::VariableId::new_unchecked("y"),
            )),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("CITY"),
                bindings: BTreeMap::new(),
            }),
        }),
    };

    let context = UnificationContext {
        catalog,
        variable_types: BTreeMap::new(),
        mode: UnificationMode::Exact,
        max_depth: 128,
    };
    let mut substitution = Substitution::default();
    assert!(unify(&pattern, &candidate, &context, &mut substitution).is_err());
}
