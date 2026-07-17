use crate::types::SemanticType;
use lexflex_model::{SemanticExpression, VariableId, WorldId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidencePolicy {
    Required,
    Optional,
    Ignore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinguaGoal {
    pub expression: SemanticExpression,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub evidence_policy: EvidencePolicy,
    pub world: Option<WorldId>,
    pub limit: Option<usize>,
}
