use crate::solve::context::UnificationContext;
use crate::solve::occurs::occurs;
use crate::solve::{Substitution, UnifyError};
use lexflex_model::{
    ConceptCatalog, ExpressionTypeChecker, ExpressionTypeEnvironment, ExpressionTypeError,
    SemanticExpression, TypeRelation, VariableId,
};

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

    let expected = context
        .variable_types
        .get(variable)
        .cloned()
        .ok_or_else(|| UnifyError::UnknownVariableType(variable.clone()))?;

    let checker = ExpressionTypeChecker::new(context.catalog.as_ref());
    let mut env = ExpressionTypeEnvironment::with_free_variables(
        context.catalog.as_ref(),
        context.variable_types.clone(),
    );
    let actual = checker
        .infer(candidate, &mut env)
        .map_err(|error| match error {
            ExpressionTypeError::UnknownConcept(concept) => UnifyError::UnknownConcept(concept),
            ExpressionTypeError::UnknownEntity(entity) => UnifyError::UnknownEntity(entity),
            ExpressionTypeError::UnknownVariable(variable) => {
                UnifyError::UnknownVariableType(variable)
            }
            _ => UnifyError::UnknownVariableType(variable.clone()),
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