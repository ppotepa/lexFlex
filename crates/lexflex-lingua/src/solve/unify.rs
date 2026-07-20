use crate::solve::bound_scope::BoundVariableScope;
use crate::solve::context::UnificationContext;
use crate::solve::unify_expression::unify_inner;
use crate::solve::{Substitution, UnifyError, UnifyOutcome};
use lexflex_model::SemanticExpression;

pub fn unify(
    pattern: &SemanticExpression,
    candidate: &SemanticExpression,
    context: &UnificationContext,
    substitution: &mut Substitution,
) -> Result<UnifyOutcome, UnifyError> {
    let mut bound = BoundVariableScope::default();
    let mut candidate_substitution = substitution.clone();
    let outcome = unify_inner(
        pattern,
        candidate,
        context,
        &mut candidate_substitution,
        0,
        &mut bound,
    )?;

    if matches!(outcome, UnifyOutcome::Matched) {
        *substitution = candidate_substitution;
    }

    Ok(outcome)
}
