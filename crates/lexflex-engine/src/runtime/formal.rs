use lexflex_lingua::LinguaExpression;
use lexflex_model::SemanticExpression;

pub(crate) fn evaluate_text_expression(
    expression: &LinguaExpression,
) -> Result<SemanticExpression, String> {
    use lexflex_lingua::LinguaExpression as L;

    Ok(match expression {
        L::Concept(concept) => SemanticExpression::Concept(concept.clone()),
        L::Entity(entity) => SemanticExpression::Entity(entity.clone()),
        L::Value(value) => SemanticExpression::Value(value.clone()),
        L::Variable(name) => {
            SemanticExpression::Variable(lexflex_model::VariableId::new_unchecked(name.as_str()))
        }
        L::QueryVariable(variable) => SemanticExpression::Variable(variable.clone()),
        L::ApplyConcept { concept, bindings } => SemanticExpression::Apply {
            concept: concept.clone(),
            bindings: bindings
                .iter()
                .map(|(parameter, value)| Ok((parameter.clone(), evaluate_text_expression(value)?)))
                .collect::<Result<_, String>>()?,
        },
        L::Satisfies { subject, concept } => SemanticExpression::Satisfies {
            subject: Box::new(evaluate_text_expression(subject)?),
            predicate: Box::new(evaluate_text_expression(concept)?),
        },
        L::Equals { left, right } => SemanticExpression::Equals {
            left: Box::new(evaluate_text_expression(left)?),
            right: Box::new(evaluate_text_expression(right)?),
        },
        L::And(items) => SemanticExpression::And(
            items
                .iter()
                .map(evaluate_text_expression)
                .collect::<Result<_, String>>()?,
        ),
        L::Or(items) => SemanticExpression::Or(
            items
                .iter()
                .map(evaluate_text_expression)
                .collect::<Result<_, String>>()?,
        ),
        L::Not(item) => SemanticExpression::Not(Box::new(evaluate_text_expression(item)?)),
        L::Exists { variable, body, .. } => SemanticExpression::Exists {
            variable: variable.clone(),
            body: Box::new(evaluate_text_expression(body)?),
        },
        L::ForAll { variable, body, .. } => SemanticExpression::ForAll {
            variable: variable.clone(),
            body: Box::new(evaluate_text_expression(body)?),
        },
        L::Function(_) | L::Lambda { .. } | L::Call { .. } | L::Let { .. } => {
            return Err(format!(
                "unsupported residual lexical expression: {expression:?}"
            ));
        }
    })
}
