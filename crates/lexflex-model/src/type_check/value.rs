use crate::{SemanticType, SemanticValue, ValueType};

pub(crate) fn semantic_value_type(value: &SemanticValue) -> SemanticType {
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
