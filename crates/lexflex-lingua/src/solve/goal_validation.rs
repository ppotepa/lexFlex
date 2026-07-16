use crate::solve::goal::LinguaGoal;
use crate::solve::type_inference::infer_expression_type;
use lexflex_model::{ConceptCatalog, SemanticExpression, SemanticType, VariableId};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GoalValidationError {
    #[error("goal limit must be greater than zero")]
    ZeroLimit,
    #[error("goal projection is empty")]
    EmptyProjection,
    #[error("duplicate projection variable {0}")]
    DuplicateProjection(VariableId),
    #[error("projection variable is not declared: {0}")]
    ProjectionVariableNotDeclared(VariableId),
    #[error("free expression variable is not declared: {0}")]
    ExpressionVariableNotDeclared(VariableId),
    #[error("declared query variable is unused: {0}")]
    UnusedVariable(VariableId),
    #[error("goal expression must be Boolean, found {0:?}")]
    NonBooleanExpression(SemanticType),
    #[error("projection variable is bound by a quantifier: {0}")]
    ProjectionVariableBound(VariableId),
}

pub fn validate_goal(
    goal: &LinguaGoal,
    catalog: &ConceptCatalog,
) -> Result<(), GoalValidationError> {
    if goal.limit == Some(0) {
        return Err(GoalValidationError::ZeroLimit);
    }
    if goal.projection.is_empty() {
        return Err(GoalValidationError::EmptyProjection);
    }

    let mut projection_seen = BTreeSet::new();
    let mut bound_projection = BTreeSet::new();
    collect_bound_projection_conflicts(&goal.expression, &mut bound_projection);

    for variable in &goal.projection {
        if !projection_seen.insert(variable.clone()) {
            return Err(GoalValidationError::DuplicateProjection(variable.clone()));
        }
        if !goal.variables.contains_key(variable) {
            return Err(GoalValidationError::ProjectionVariableNotDeclared(
                variable.clone(),
            ));
        }
        if bound_projection.contains(variable) {
            return Err(GoalValidationError::ProjectionVariableBound(
                variable.clone(),
            ));
        }
    }

    let free = collect_free_variables(&goal.expression);
    for variable in &free {
        if !goal.variables.contains_key(variable) {
            return Err(GoalValidationError::ExpressionVariableNotDeclared(
                variable.clone(),
            ));
        }
    }
    for variable in goal.variables.keys() {
        if !free.contains(variable) {
            return Err(GoalValidationError::UnusedVariable(variable.clone()));
        }
    }

    let expression_type = infer_expression_type(&goal.expression, catalog, &goal.variables)
        .map_err(|error| match error {
            crate::solve::type_inference::SolveTypeError::Variable(variable) => {
                GoalValidationError::ExpressionVariableNotDeclared(variable)
            }
            crate::solve::type_inference::SolveTypeError::Concept(_)
            | crate::solve::type_inference::SolveTypeError::Entity(_) => {
                GoalValidationError::NonBooleanExpression(SemanticType::Concept)
            }
        })?;
    if expression_type != SemanticType::Boolean {
        return Err(GoalValidationError::NonBooleanExpression(expression_type));
    }
    Ok(())
}

fn collect_free_variables(expression: &SemanticExpression) -> BTreeSet<VariableId> {
    fn walk(
        expression: &SemanticExpression,
        bound: &mut Vec<VariableId>,
        output: &mut BTreeSet<VariableId>,
    ) {
        match expression {
            SemanticExpression::Variable(variable) => {
                if !bound.contains(variable) {
                    output.insert(variable.clone());
                }
            }
            SemanticExpression::Apply { bindings, .. } => {
                for value in bindings.values() {
                    walk(value, bound, output);
                }
            }
            SemanticExpression::Satisfies { subject, predicate } => {
                walk(subject, bound, output);
                walk(predicate, bound, output);
            }
            SemanticExpression::Equals { left, right } => {
                walk(left, bound, output);
                walk(right, bound, output);
            }
            SemanticExpression::And(items) | SemanticExpression::Or(items) => {
                for item in items {
                    walk(item, bound, output);
                }
            }
            SemanticExpression::Not(inner) => walk(inner, bound, output),
            SemanticExpression::Exists { variable, body }
            | SemanticExpression::ForAll { variable, body } => {
                bound.push(variable.clone());
                walk(body, bound, output);
                bound.pop();
            }
            SemanticExpression::Qualified {
                expression,
                qualifiers,
            } => {
                walk(expression, bound, output);
                for value in qualifiers.values() {
                    walk(value, bound, output);
                }
            }
            SemanticExpression::Concept(_)
            | SemanticExpression::Entity(_)
            | SemanticExpression::Value(_) => {}
        }
    }

    let mut bound = Vec::new();
    let mut output = BTreeSet::new();
    walk(expression, &mut bound, &mut output);
    output
}

fn collect_bound_projection_conflicts(
    expression: &SemanticExpression,
    output: &mut BTreeSet<VariableId>,
) {
    match expression {
        SemanticExpression::Exists { variable, body }
        | SemanticExpression::ForAll { variable, body } => {
            output.insert(variable.clone());
            collect_bound_projection_conflicts(body, output);
        }
        SemanticExpression::Apply { bindings, .. } => {
            for value in bindings.values() {
                collect_bound_projection_conflicts(value, output);
            }
        }
        SemanticExpression::Satisfies { subject, predicate } => {
            collect_bound_projection_conflicts(subject, output);
            collect_bound_projection_conflicts(predicate, output);
        }
        SemanticExpression::Equals { left, right } => {
            collect_bound_projection_conflicts(left, output);
            collect_bound_projection_conflicts(right, output);
        }
        SemanticExpression::And(items) | SemanticExpression::Or(items) => {
            for item in items {
                collect_bound_projection_conflicts(item, output);
            }
        }
        SemanticExpression::Not(inner) => collect_bound_projection_conflicts(inner, output),
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => {
            collect_bound_projection_conflicts(expression, output);
            for value in qualifiers.values() {
                collect_bound_projection_conflicts(value, output);
            }
        }
        SemanticExpression::Concept(_)
        | SemanticExpression::Entity(_)
        | SemanticExpression::Value(_)
        | SemanticExpression::Variable(_) => {}
    }
}
