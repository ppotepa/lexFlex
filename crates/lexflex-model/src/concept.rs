use crate::{ConceptId, ConceptKind, ParameterId, SemanticType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptParameterSchema {
    pub id: ParameterId,
    pub value_type: SemanticType,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptSchema {
    pub id: ConceptId,
    pub kind: ConceptKind,
    pub parameters: BTreeMap<ParameterId, ConceptParameterSchema>,
    pub result_type: SemanticType,
}
