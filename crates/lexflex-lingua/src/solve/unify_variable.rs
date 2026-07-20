use crate::solve::context::UnificationContext;
use crate::solve::occurs::occurs;
use crate::solve::{Substitution, UnifyError, UnifyMismatch, UnifyOutcome};
use lexflex_model::{
    ExpressionTypeChecker, ExpressionTypeEnvironment, ExpressionTypeError, SemanticExpression,
    TypeRelation, VariableId,
};

pub(crate) fn bind_variable(
    variable: &VariableId,
    candidate: &SemanticExpression,
    context: &UnificationContext,
    substitution: &mut Substitution,
) -> Result<UnifyOutcome, UnifyError> {
    if occurs(variable, candidate) {
        return Ok(UnifyOutcome::Mismatch(UnifyMismatch::Occurs {
            variable: variable.clone(),
            candidate: candidate.clone(),
        }));
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
            ExpressionTypeError::UnknownVariable(variable) => {
                UnifyError::UnknownVariableType(variable)
            }
            other => UnifyError::CandidateType(other),
        })?;

    if !TypeRelation::new(context.catalog.as_ref()).accepts(&expected, &actual) {
        return Ok(UnifyOutcome::Mismatch(UnifyMismatch::VariableType {
            variable: variable.clone(),
            expected,
            actual,
        }));
    }

    match substitution.bind(variable.clone(), candidate.clone()) {
        Ok(()) => Ok(UnifyOutcome::Matched),
        Err(mismatch) => Ok(UnifyOutcome::Mismatch(mismatch)),
    }
}
