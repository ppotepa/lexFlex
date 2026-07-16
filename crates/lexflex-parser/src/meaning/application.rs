use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::meaning::{count_lingua_nodes, MeaningInstance};
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
                return Err(ParseError::ConflictingQueryVariableType {
                    variable: variable.clone(),
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

    Ok(MeaningInstance {
        expression,
        query_variables,
        semantic_nodes,
    })
}
