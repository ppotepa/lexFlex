use crate::solve::free_variables::collect_free_variables;
use crate::solve::goal::LinguaGoal;
use lexflex_model::{
    validate_semantic_type_references, ConceptCatalog, ExpressionTypeChecker,
    ExpressionTypeEnvironment, ExpressionTypeError, SemanticType, SemanticTypeReferenceError,
    VariableId,
};
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
    #[error("invalid type for query variable {variable}: {source}")]
    InvalidVariableType {
        variable: VariableId,
        source: SemanticTypeReferenceError,
    },
    #[error("projection variable is not free in expression: {0}")]
    ProjectionVariableNotFree(VariableId),
    #[error("goal expression type error: {0}")]
    Type(ExpressionTypeError),
    #[error("goal expression must be Boolean, found {0:?}")]
    NonBooleanExpression(SemanticType),
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

    let free = collect_free_variables(&goal.expression);

    let mut projection_seen = BTreeSet::new();
    for variable in &goal.projection {
        if !projection_seen.insert(variable.clone()) {
            return Err(GoalValidationError::DuplicateProjection(variable.clone()));
        }
        if !goal.variables.contains_key(variable) {
            return Err(GoalValidationError::ProjectionVariableNotDeclared(
                variable.clone(),
            ));
        }
        if !free.contains(variable) {
            return Err(GoalValidationError::ProjectionVariableNotFree(
                variable.clone(),
            ));
        }
    }

    for variable in &free {
        if !goal.variables.contains_key(variable) {
            return Err(GoalValidationError::ExpressionVariableNotDeclared(
                variable.clone(),
            ));
        }
    }
    for (variable, value_type) in &goal.variables {
        validate_semantic_type_references(value_type, catalog).map_err(|source| {
            GoalValidationError::InvalidVariableType {
                variable: variable.clone(),
                source,
            }
        })?;
        if !free.contains(variable) {
            return Err(GoalValidationError::UnusedVariable(variable.clone()));
        }
    }

    let checker = ExpressionTypeChecker::new(catalog);
    let mut env = ExpressionTypeEnvironment::with_free_variables(catalog, goal.variables.clone());
    let expression_type = checker
        .infer(&goal.expression, &mut env)
        .map_err(GoalValidationError::Type)?;
    if expression_type != SemanticType::Boolean {
        return Err(GoalValidationError::NonBooleanExpression(expression_type));
    }
    Ok(())
}
