use crate::{ConceptId, ParameterId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ConceptKind {
    EntityType,
    RoleType,
    AttributeType,
    EventType,
    StateType,
    RelationType,
    ValueFunction,
    Predicate,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ValueType {
    Integer,
    Decimal,
    Boolean,
    Text,
    Date,
    DateTime,
    Duration,
    Quantity { dimension: ConceptId },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FunctionType {
    pub parameters: BTreeMap<ParameterId, SemanticType>,
    pub result: Box<SemanticType>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SemanticType {
    Boolean,
    Concept,
    ConceptOf(ConceptKind),
    Entity,
    EntityOf(ConceptId),
    Value(ValueType),
    Predicate(Box<SemanticType>),
    Set(Box<SemanticType>),
    Optional(Box<SemanticType>),
    Record(BTreeMap<String, SemanticType>),
    Function(FunctionType),
}
