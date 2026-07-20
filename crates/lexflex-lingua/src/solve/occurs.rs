use lexflex_model::{SemanticExpression, VariableId};

pub fn occurs(variable: &VariableId, expression: &SemanticExpression) -> bool {
    occurs_free(variable, expression, &mut Vec::new())
}

pub fn occurs_free(
    variable: &VariableId,
    expression: &SemanticExpression,
    bound: &mut Vec<VariableId>,
) -> bool {
    match expression {
        SemanticExpression::Variable(candidate) => {
            candidate == variable && !bound.contains(candidate)
        }
        SemanticExpression::Apply { bindings, .. } => bindings
            .values()
            .any(|value| occurs_free(variable, value, bound)),
        SemanticExpression::Satisfies { subject, predicate } => {
            occurs_free(variable, subject, bound) || occurs_free(variable, predicate, bound)
        }
        SemanticExpression::Equals { left, right } => {
            occurs_free(variable, left, bound) || occurs_free(variable, right, bound)
        }
        SemanticExpression::And(items) | SemanticExpression::Or(items) => {
            items.iter().any(|item| occurs_free(variable, item, bound))
        }
        SemanticExpression::Not(inner) => occurs_free(variable, inner, bound),
        SemanticExpression::Exists {
            variable: bound_variable,
            body,
            ..
        }
        | SemanticExpression::ForAll {
            variable: bound_variable,
            body,
            ..
        } => {
            bound.push(bound_variable.clone());
            let found = occurs_free(variable, body, bound);
            bound.pop();
            found
        }
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => {
            occurs_free(variable, expression, bound)
                || qualifiers
                    .values()
                    .any(|value| occurs_free(variable, value, bound))
        }
        SemanticExpression::Concept(_)
        | SemanticExpression::Entity(_)
        | SemanticExpression::Value(_) => false,
    }
}
