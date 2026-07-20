use crate::solve::bound_scope::BoundVariableScope;
use crate::solve::context::{UnificationContext, UnificationMode};
use crate::solve::unify_variable::bind_variable;
use crate::solve::{Substitution, UnifyError, UnifyMismatch, UnifyOutcome};
use lexflex_model::{ParameterId, SemanticExpression};

pub(crate) fn unify_inner(
    pattern: &SemanticExpression,
    candidate: &SemanticExpression,
    context: &UnificationContext,
    substitution: &mut Substitution,
    depth: usize,
    bound: &mut BoundVariableScope,
) -> Result<UnifyOutcome, UnifyError> {
    if depth > context.max_depth {
        return Err(UnifyError::DepthLimitExceeded {
            depth: context.max_depth,
        });
    }
    match (pattern, candidate) {
        (SemanticExpression::Variable(variable), SemanticExpression::Variable(other))
            if bound.right_for_left(variable).is_some()
                || bound.left_for_right(other).is_some() =>
        {
            if bound.right_for_left(variable) == Some(other) {
                Ok(UnifyOutcome::Matched)
            } else {
                Ok(UnifyOutcome::Mismatch(UnifyMismatch::Value {
                    pattern: pattern.clone(),
                    candidate: candidate.clone(),
                }))
            }
        }
        (SemanticExpression::Variable(variable), value) => {
            bind_variable(variable, value, context, substitution)
        }
        (SemanticExpression::Concept(left), SemanticExpression::Concept(right))
            if left == right =>
        {
            Ok(UnifyOutcome::Matched)
        }
        (SemanticExpression::Entity(left), SemanticExpression::Entity(right)) if left == right => {
            Ok(UnifyOutcome::Matched)
        }
        (SemanticExpression::Value(left), SemanticExpression::Value(right)) if left == right => {
            Ok(UnifyOutcome::Matched)
        }
        (
            SemanticExpression::Apply {
                concept: left_concept,
                bindings: left_bindings,
            },
            SemanticExpression::Apply {
                concept: right_concept,
                bindings: right_bindings,
            },
        ) if left_concept == right_concept => {
            if matches!(context.mode, UnificationMode::Exact)
                && left_bindings.len() != right_bindings.len()
            {
                return Ok(UnifyOutcome::Mismatch(UnifyMismatch::Shape));
            }
            if left_bindings.len() > right_bindings.len() {
                return Ok(UnifyOutcome::Mismatch(UnifyMismatch::Shape));
            }
            for (parameter, left_value) in left_bindings {
                let Some(right_value) = right_bindings.get(parameter) else {
                    return Ok(UnifyOutcome::Mismatch(UnifyMismatch::MissingBinding(
                        parameter.clone(),
                    )));
                };
                let outcome = unify_inner(
                    left_value,
                    right_value,
                    context,
                    substitution,
                    depth + 1,
                    bound,
                )?;
                if !matches!(outcome, UnifyOutcome::Matched) {
                    return Ok(outcome);
                }
            }
            Ok(UnifyOutcome::Matched)
        }
        (
            SemanticExpression::Satisfies {
                subject: left_subject,
                predicate: left_predicate,
            },
            SemanticExpression::Satisfies {
                subject: right_subject,
                predicate: right_predicate,
            },
        ) => {
            let outcome = unify_inner(
                left_subject,
                right_subject,
                context,
                substitution,
                depth + 1,
                bound,
            )?;
            if !matches!(outcome, UnifyOutcome::Matched) {
                return Ok(outcome);
            }
            unify_inner(
                left_predicate,
                right_predicate,
                context,
                substitution,
                depth + 1,
                bound,
            )
        }
        (
            SemanticExpression::Equals {
                left: left_a,
                right: left_b,
            },
            SemanticExpression::Equals {
                left: right_a,
                right: right_b,
            },
        ) => {
            let outcome = unify_inner(left_a, right_a, context, substitution, depth + 1, bound)?;
            if !matches!(outcome, UnifyOutcome::Matched) {
                return Ok(outcome);
            }
            unify_inner(left_b, right_b, context, substitution, depth + 1, bound)
        }
        (SemanticExpression::And(left), SemanticExpression::And(right))
        | (SemanticExpression::Or(left), SemanticExpression::Or(right))
            if left.len() == right.len() =>
        {
            for (left_item, right_item) in left.iter().zip(right) {
                let outcome = unify_inner(
                    left_item,
                    right_item,
                    context,
                    substitution,
                    depth + 1,
                    bound,
                )?;
                if !matches!(outcome, UnifyOutcome::Matched) {
                    return Ok(outcome);
                }
            }
            Ok(UnifyOutcome::Matched)
        }
        (SemanticExpression::Not(left), SemanticExpression::Not(right)) => {
            unify_inner(left, right, context, substitution, depth + 1, bound)
        }
        (
            SemanticExpression::Exists {
                variable: left_variable,
                body: left_body,
                value_type: left_type,
            },
            SemanticExpression::Exists {
                variable: right_variable,
                body: right_body,
                value_type: right_type,
            },
        )
        | (
            SemanticExpression::ForAll {
                variable: left_variable,
                body: left_body,
                value_type: left_type,
            },
            SemanticExpression::ForAll {
                variable: right_variable,
                body: right_body,
                value_type: right_type,
            },
        ) => {
            if left_type != right_type {
                return Ok(UnifyOutcome::Mismatch(UnifyMismatch::Value {
                    pattern: pattern.clone(),
                    candidate: candidate.clone(),
                }));
            }
            bound.push(left_variable.clone(), right_variable.clone());
            let result = unify_inner(
                left_body,
                right_body,
                context,
                substitution,
                depth + 1,
                bound,
            );
            bound.pop()?;
            result
        }
        (
            SemanticExpression::Qualified {
                expression: left_expression,
                qualifiers: left_qualifiers,
            },
            SemanticExpression::Qualified {
                expression: right_expression,
                qualifiers: right_qualifiers,
            },
        ) if left_qualifiers.len() <= right_qualifiers.len() => {
            let outcome = unify_inner(
                left_expression,
                right_expression,
                context,
                substitution,
                depth + 1,
                bound,
            )?;
            if !matches!(outcome, UnifyOutcome::Matched) {
                return Ok(outcome);
            }
            for (qualifier, left_value) in left_qualifiers {
                let Some(right_value) = right_qualifiers.get(qualifier) else {
                    return Ok(UnifyOutcome::Mismatch(UnifyMismatch::MissingBinding(
                        ParameterId::new_unchecked(qualifier.as_str()),
                    )));
                };
                let outcome = unify_inner(
                    left_value,
                    right_value,
                    context,
                    substitution,
                    depth + 1,
                    bound,
                )?;
                if !matches!(outcome, UnifyOutcome::Matched) {
                    return Ok(outcome);
                }
            }
            Ok(UnifyOutcome::Matched)
        }
        _ => Ok(UnifyOutcome::Mismatch(UnifyMismatch::Value {
            pattern: pattern.clone(),
            candidate: candidate.clone(),
        })),
    }
}
