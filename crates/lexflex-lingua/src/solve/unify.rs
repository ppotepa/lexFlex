use crate::solve::context::{UnificationContext, UnificationMode};
use crate::solve::occurs::occurs;
use crate::solve::type_inference::{
    infer_expression_type, semantic_types_compatible, variable_context, SolveTypeError,
};
use crate::solve::Substitution;
use lexflex_model::{SemanticExpression, SemanticType, VariableId};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UnifyError {
    #[error("shape mismatch")]
    ShapeMismatch,
    #[error("missing binding {0}")]
    MissingBinding(lexflex_model::ParameterId),
    #[error("conflicting binding for {variable}")]
    ConflictingBinding {
        variable: VariableId,
        existing: SemanticExpression,
        incoming: SemanticExpression,
    },
    #[error("value mismatch")]
    ValueMismatch {
        pattern: SemanticExpression,
        candidate: SemanticExpression,
    },
    #[error("occurs check failed for {variable}")]
    OccursCheck {
        variable: VariableId,
        candidate: SemanticExpression,
    },
    #[error("unknown variable type {0}")]
    UnknownVariableType(VariableId),
    #[error("type mismatch for {variable}: expected {expected:?}, found {actual:?}")]
    VariableTypeMismatch {
        variable: VariableId,
        expected: SemanticType,
        actual: SemanticType,
    },
}

pub fn unify(
    pattern: &SemanticExpression,
    candidate: &SemanticExpression,
    context: &UnificationContext,
    substitution: &mut Substitution,
) -> Result<(), UnifyError> {
    unify_inner(pattern, candidate, context, substitution, 0)
}

fn unify_inner(
    pattern: &SemanticExpression,
    candidate: &SemanticExpression,
    context: &UnificationContext,
    substitution: &mut Substitution,
    depth: usize,
) -> Result<(), UnifyError> {
    if depth > context.max_depth {
        return Err(UnifyError::ValueMismatch {
            pattern: pattern.clone(),
            candidate: candidate.clone(),
        });
    }
    match (pattern, candidate) {
        (SemanticExpression::Variable(variable), value) => {
            bind_variable(variable, value, context, substitution)
        }
        (SemanticExpression::Concept(left), SemanticExpression::Concept(right))
            if left == right =>
        {
            Ok(())
        }
        (SemanticExpression::Entity(left), SemanticExpression::Entity(right)) if left == right => {
            Ok(())
        }
        (SemanticExpression::Value(left), SemanticExpression::Value(right)) if left == right => {
            Ok(())
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
                return Err(UnifyError::ShapeMismatch);
            }
            if left_bindings.len() > right_bindings.len() {
                return Err(UnifyError::ShapeMismatch);
            }
            for (parameter, left_value) in left_bindings {
                let right_value = right_bindings
                    .get(parameter)
                    .ok_or_else(|| UnifyError::MissingBinding(parameter.clone()))?;
                unify_inner(left_value, right_value, context, substitution, depth + 1)?;
            }
            Ok(())
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
            unify_inner(
                left_subject,
                right_subject,
                context,
                substitution,
                depth + 1,
            )?;
            unify_inner(
                left_predicate,
                right_predicate,
                context,
                substitution,
                depth + 1,
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
            unify_inner(left_a, right_a, context, substitution, depth + 1)?;
            unify_inner(left_b, right_b, context, substitution, depth + 1)
        }
        (SemanticExpression::And(left), SemanticExpression::And(right))
        | (SemanticExpression::Or(left), SemanticExpression::Or(right))
            if left.len() == right.len() =>
        {
            for (left_item, right_item) in left.iter().zip(right) {
                unify_inner(left_item, right_item, context, substitution, depth + 1)?;
            }
            Ok(())
        }
        (SemanticExpression::Not(left), SemanticExpression::Not(right)) => {
            unify_inner(left, right, context, substitution, depth + 1)
        }
        (
            SemanticExpression::Exists {
                variable: left_variable,
                body: left_body,
            },
            SemanticExpression::Exists {
                variable: right_variable,
                body: right_body,
            },
        )
        | (
            SemanticExpression::ForAll {
                variable: left_variable,
                body: left_body,
            },
            SemanticExpression::ForAll {
                variable: right_variable,
                body: right_body,
            },
        ) if left_variable == right_variable => {
            unify_inner(left_body, right_body, context, substitution, depth + 1)
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
            unify_inner(
                left_expression,
                right_expression,
                context,
                substitution,
                depth + 1,
            )?;
            for (qualifier, left_value) in left_qualifiers {
                let right_value = right_qualifiers.get(qualifier).ok_or_else(|| {
                    UnifyError::MissingBinding(lexflex_model::ParameterId::new_unchecked(
                        qualifier.as_str(),
                    ))
                })?;
                unify_inner(left_value, right_value, context, substitution, depth + 1)?;
            }
            Ok(())
        }
        _ => Err(UnifyError::ValueMismatch {
            pattern: pattern.clone(),
            candidate: candidate.clone(),
        }),
    }
}

fn bind_variable(
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

    if !semantic_types_compatible(&actual, &expected, context.catalog.as_ref()) {
        return Err(UnifyError::VariableTypeMismatch {
            variable: variable.clone(),
            expected,
            actual,
        });
    }

    substitution.bind(variable.clone(), candidate.clone())
}
