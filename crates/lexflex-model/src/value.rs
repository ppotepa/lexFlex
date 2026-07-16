use crate::ConceptId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DecimalValue {
    pub canonical: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DateValue {
    pub iso8601: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct QuantityValue {
    pub amount: DecimalValue,
    pub unit: ConceptId,
    pub dimension: ConceptId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SemanticValue {
    Integer(i64),
    Decimal(DecimalValue),
    Text(String),
    Boolean(bool),
    Date(DateValue),
    Quantity(QuantityValue),
}

impl From<i64> for SemanticValue {
    fn from(value: i64) -> Self {
        SemanticValue::Integer(value)
    }
}

impl From<bool> for SemanticValue {
    fn from(value: bool) -> Self {
        SemanticValue::Boolean(value)
    }
}

impl From<&str> for SemanticValue {
    fn from(value: &str) -> Self {
        SemanticValue::Text(value.to_owned())
    }
}

impl From<String> for SemanticValue {
    fn from(value: String) -> Self {
        SemanticValue::Text(value)
    }
}

impl From<QuantityValue> for SemanticValue {
    fn from(value: QuantityValue) -> Self {
        SemanticValue::Quantity(value)
    }
}
