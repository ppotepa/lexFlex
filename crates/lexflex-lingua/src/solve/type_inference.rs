use crate::solve::context::UnificationContext;
use lexflex_model::{
    ConceptCatalog, ConceptId, SemanticExpression, SemanticType, SemanticValue, TypeRelation,
    ValueType, VariableId,
};
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum SolveTypeError {
    #[error("unknown concept: {0}")]
    Concept(ConceptId),
    #[error("unknown entity: {0}")]
    Entity(lexflex_model::EntityId),
    #[error("unknown variable: {0}")]
    Variable(VariableId),
}

pub fn infer_expression_type(
    expression: &SemanticExpression,
    catalog: &ConceptCatalog,
    variables: &BTreeMap<VariableId, SemanticType>,
) -> Result<SemanticType, SolveTypeError> {
    match expression {
        SemanticExpression::Concept(concept_id) => {
            let schema = catalog
                .concept(concept_id)
                .ok_or_else(|| SolveTypeError::Concept(concept_id.clone()))?;
            Ok(SemanticType::ConceptOf(schema.kind))
        }
        SemanticExpression::Entity(entity_id) => {
            let entity = catalog
                .entity(entity_id)
                .ok_or_else(|| SolveTypeError::Entity(entity_id.clone()))?;
            Ok(SemanticType::EntityOf(entity.primary_type.clone()))
        }
        SemanticExpression::Value(value) => Ok(value_type(value)),
        SemanticExpression::Variable(variable) => variables
            .get(variable)
            .cloned()
            .ok_or_else(|| SolveTypeError::Variable(variable.clone())),
        SemanticExpression::Apply { concept, .. } => Ok(catalog
            .concept(concept)
            .ok_or_else(|| SolveTypeError::Concept(concept.clone()))?
            .result_type
            .clone()),
        SemanticExpression::Satisfies { .. }
        | SemanticExpression::Equals { .. }
        | SemanticExpression::And(_)
        | SemanticExpression::Or(_)
        | SemanticExpression::Not(_)
        | SemanticExpression::Exists { .. }
        | SemanticExpression::ForAll { .. } => Ok(SemanticType::Boolean),
        SemanticExpression::Qualified { expression, .. } => {
            infer_expression_type(expression, catalog, variables)
        }
    }
}

pub fn semantic_types_compatible(
    actual: &SemanticType,
    expected: &SemanticType,
    catalog: &ConceptCatalog,
) -> bool {
    TypeRelation::new(catalog).accepts(expected, actual)
}

pub fn variable_context(
    context: &UnificationContext,
    variable: &VariableId,
) -> Result<SemanticType, SolveTypeError> {
    context
        .variable_types
        .get(variable)
        .cloned()
        .ok_or_else(|| SolveTypeError::Variable(variable.clone()))
}

fn value_type(value: &SemanticValue) -> SemanticType {
    match value {
        SemanticValue::Integer(_) => SemanticType::Value(ValueType::Integer),
        SemanticValue::Decimal(_) => SemanticType::Value(ValueType::Decimal),
        SemanticValue::Text(_) => SemanticType::Value(ValueType::Text),
        SemanticValue::Boolean(_) => SemanticType::Value(ValueType::Boolean),
        SemanticValue::Date(_) => SemanticType::Value(ValueType::Date),
        SemanticValue::Quantity(quantity) => SemanticType::Value(ValueType::Quantity {
            dimension: quantity.dimension.clone(),
        }),
    }
}
