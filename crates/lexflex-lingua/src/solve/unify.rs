use crate::solve::bound_scope::BoundVariableScope;
use crate::solve::context::UnificationContext;
use crate::solve::unify_expression::unify_inner;
use crate::solve::{Substitution, UnifyError};
use lexflex_model::SemanticExpression;

pub fn unify(
    pattern: &SemanticExpression,
    candidate: &SemanticExpression,
    context: &UnificationContext,
    substitution: &mut Substitution,
) -> Result<(), UnifyError> {
    let mut bound = BoundVariableScope::default();
    unify_inner(pattern, candidate, context, substitution, 0, &mut bound)
}
