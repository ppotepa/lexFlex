use lexflex_model::{SemanticExpression, VariableId};
use std::collections::BTreeSet;

pub(crate) fn collect_free_variables(expression: &SemanticExpression) -> BTreeSet<VariableId> {
    let mut output = BTreeSet::new();
    collect(expression, &mut Vec::new(), &mut output);
    output
}

fn collect(
    expression: &SemanticExpression,
    bound: &mut Vec<VariableId>,
    output: &mut BTreeSet<VariableId>,
) {
    match expression {
        SemanticExpression::Variable(variable) => {
            if !bound.iter().rev().any(|current| current == variable) {
                output.insert(variable.clone());
            }
        }
        SemanticExpression::Apply { bindings, .. } => {
            for value in bindings.values() {
                collect(value, bound, output);
            }
        }
        SemanticExpression::Satisfies { subject, predicate }
        | SemanticExpression::Equals {
            left: subject,
            right: predicate,
        } => {
            collect(subject, bound, output);
            collect(predicate, bound, output);
        }
        SemanticExpression::And(items) | SemanticExpression::Or(items) => {
            for item in items {
                collect(item, bound, output);
            }
        }
        SemanticExpression::Not(inner) => {
            collect(inner, bound, output);
        }
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => {
            collect(expression, bound, output);
            for qualifier in qualifiers.values() {
                collect(qualifier, bound, output);
            }
        }
        SemanticExpression::Exists { variable, body, .. }
        | SemanticExpression::ForAll { variable, body, .. } => {
            bound.push(variable.clone());
            collect(body, bound, output);
            bound.pop();
        }
        SemanticExpression::Concept(_)
        | SemanticExpression::Entity(_)
        | SemanticExpression::Value(_) => {}
    }
}
