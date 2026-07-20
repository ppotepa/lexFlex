mod support;

use lexflex_lingua::{unify, Substitution, UnificationContext, UnificationMode};
use lexflex_model::{ConceptId, SemanticExpression, SemanticType, VariableId};
use std::collections::BTreeMap;
use std::sync::Arc;
use support::model::kernel_catalog;

#[test]
fn nested_same_ids_preserve_scope_during_unification() {
    let expression = SemanticExpression::Exists {
        variable: VariableId::new_unchecked("x"),
        value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        body: Box::new(SemanticExpression::ForAll {
            variable: VariableId::new_unchecked("x"),
            value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
            body: Box::new(SemanticExpression::Satisfies {
                subject: Box::new(SemanticExpression::Variable(VariableId::new_unchecked("x"))),
                predicate: Box::new(SemanticExpression::Apply {
                    concept: ConceptId::new_unchecked("CITY"),
                    bindings: BTreeMap::new(),
                }),
            }),
        }),
    };

    let context = UnificationContext {
        catalog: Arc::new(kernel_catalog()),
        variable_types: BTreeMap::new(),
        mode: UnificationMode::Exact,
        max_depth: 128,
    };
    let mut substitution = Substitution::default();
    assert!(unify(&expression, &expression, &context, &mut substitution).is_ok());
}
