use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::meaning::{count_lingua_nodes, BooleanOperator, MeaningInstance};
use lexflex_model::ParameterId;
use std::collections::BTreeMap;

pub(crate) fn apply_meaning(
    function: &MeaningInstance,
    semantic_parameter: &ParameterId,
    argument: &MeaningInstance,
    max_semantic_nodes: usize,
) -> Result<MeaningInstance, ParseError> {
    let expression = lexflex_lingua::LinguaExpression::Call {
        callee: Box::new(function.expression.clone()),
        arguments: BTreeMap::from([(semantic_parameter.clone(), argument.expression.clone())]),
    };

    let mut query_variables = function.query_variables.clone();
    for (variable, incoming) in &argument.query_variables {
        if let Some(existing) = query_variables.get(variable) {
            if existing != incoming {
                return Err(ParseError::ConflictingQueryCategoryType {
                    existing: existing.clone(),
                    incoming: incoming.clone(),
                });
            }
        } else {
            query_variables.insert(variable.clone(), incoming.clone());
        }
    }
    let semantic_nodes = count_lingua_nodes(&expression);
    if semantic_nodes > max_semantic_nodes {
        return Err(ParseError::BudgetExceeded(
            ParseBudgetLimit::SemanticNodeLimit,
        ));
    }

    let scope_violation = match (function.boolean_operator, argument.boolean_operator) {
        (Some(BooleanOperator::Not), Some(BooleanOperator::And | BooleanOperator::Or))
        | (Some(BooleanOperator::And), Some(BooleanOperator::Or)) => 1,
        _ => 0,
    };

    Ok(MeaningInstance {
        expression,
        query_variables,
        semantic_nodes,
        boolean_operator: function.boolean_operator.or(argument.boolean_operator),
        boolean_scope_violations: function.boolean_scope_violations
            + argument.boolean_scope_violations
            + scope_violation,
    })
}
