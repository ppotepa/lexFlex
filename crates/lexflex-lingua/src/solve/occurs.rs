use lexflex_model::{SemanticExpression, VariableId};

pub fn occurs(variable: &VariableId, expression: &SemanticExpression) -> bool {
    match expression {
        SemanticExpression::Variable(candidate) => candidate == variable,
        SemanticExpression::Apply { bindings, .. } => {
            bindings.values().any(|value| occurs(variable, value))
        }
        SemanticExpression::Satisfies { subject, predicate } => {
            occurs(variable, subject) || occurs(variable, predicate)
        }
        SemanticExpression::Equals { left, right } => {
            occurs(variable, left) || occurs(variable, right)
        }
        SemanticExpression::And(items) | SemanticExpression::Or(items) => {
            items.iter().any(|item| occurs(variable, item))
        }
        SemanticExpression::Not(inner) => occurs(variable, inner),
        SemanticExpression::Exists {
            variable: bound,
            body,
        }
        | SemanticExpression::ForAll {
            variable: bound,
            body,
        } => {
            if bound == variable {
                false
            } else {
                occurs(variable, body)
            }
        }
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => {
            occurs(variable, expression) || qualifiers.values().any(|value| occurs(variable, value))
        }
        SemanticExpression::Concept(_)
        | SemanticExpression::Entity(_)
        | SemanticExpression::Value(_) => false,
    }
}
