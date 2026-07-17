use crate::solve::context::UnificationContext;
use crate::solve::occurs::occurs;
use crate::solve::type_inference::{infer_expression_type, variable_context, SolveTypeError};
use crate::solve::{Substitution, UnifyError};
use lexflex_model::{SemanticExpression, TypeRelation, VariableId};

pub(crate) fn bind_variable(
    variable: &VariableId,
    candidate: &SemanticExpression,
    context: &UnificationContext,
    substitution: &mut Substitution,
) -> Result<(), UnifyError> {
    if occurs(variable, candidate) {
        return Err(UnifyError::OccursCheck {
            variable: variable.clone(),
            candidate: candidate.clone(),
        });
    }

    let expected = variable_context(context, variable)
        .map_err(|_| UnifyError::UnknownVariableType(variable.clone()))?;
    let actual =
        infer_expression_type(candidate, context.catalog.as_ref(), &context.variable_types)
            .map_err(|error| match error {
                SolveTypeError::Concept(concept) => UnifyError::ValueMismatch {
                    pattern: SemanticExpression::Concept(concept),
                    candidate: candidate.clone(),
                },
                SolveTypeError::Entity(entity) => UnifyError::ValueMismatch {
                    pattern: SemanticExpression::Entity(entity),
                    candidate: candidate.clone(),
                },
                SolveTypeError::Variable(variable) => UnifyError::UnknownVariableType(variable),
            })?;

    if !TypeRelation::new(context.catalog.as_ref()).accepts(&expected, &actual) {
        return Err(UnifyError::VariableTypeMismatch {
            variable: variable.clone(),
            expected,
            actual,
        });
    }

    substitution.bind(variable.clone(), candidate.clone())
}
